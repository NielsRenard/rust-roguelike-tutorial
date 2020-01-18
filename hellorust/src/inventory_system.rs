extern crate specs;
use super::{
    gamelog::GameLog, AreaOfEffect, CombatStats, Consumable, InBackpack, InflictsDamage, Map, Name,
    Position, ProvidesHealing, SufferDamage, WantsToDropItem, WantsToPickupItem, WantsToUseItem,
};
use specs::prelude::*;

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_pickup, mut positions, names, mut backpack) =
            data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack
                .insert(
                    pickup.item,
                    InBackpack {
                        owner: pickup.collected_by,
                    },
                )
                .expect("Unable to insert backpack entry");

            if pickup.collected_by == *player_entity {
                gamelog.entries.insert(
                    0,
                    format!("You pick up the {}.", names.get(pickup.item).unwrap().name),
                );
            }
        }
        wants_pickup.clear();
    }
}

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToDropItem>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            entities,
            mut wants_drop,
            names,
            mut positions,
            mut backpack,
        ) = data;

        for (entity, to_drop) in (&entities, &wants_drop).join() {
            let mut dropper_pos: Position = Position { x: 0, y: 0 };
            {
                // set the drop position to the position of the dropper (could be player could be monster)
                let dropped_pos = positions.get(entity).unwrap();
                dropper_pos.x = dropped_pos.x;
                dropper_pos.y = dropped_pos.y;
            }
            positions
                .insert(
                    to_drop.item,
                    Position {
                        x: dropper_pos.x,
                        y: dropper_pos.y,
                    },
                )
                .expect("Unable to insert drop item position");
            backpack.remove(to_drop.item);

            if entity == *player_entity {
                gamelog.entries.insert(
                    0,
                    format!("You drop the {}.", names.get(to_drop.item).unwrap().name),
                );
            }
        }

        wants_drop.clear();
    }
}

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        ReadExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, InflictsDamage>,
        ReadStorage<'a, AreaOfEffect>,
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            map,
            entities,
            mut wants_use,
            names,
            consumables,
            healing,
            inflicts_damage,
            aoe,
            mut combat_stats,
            mut suffer_damage,
        ) = data;

        for (entity, use_item) in (&entities, &wants_use).join() {
            let mut used_item = true;

            // Targeting
            let mut targets: Vec<Entity> = Vec::new();
            match use_item.target {
                None => {
                    targets.push(*player_entity);
                }
                Some(target) => {
                    let area_of_effect = aoe.get(use_item.item);
                    match area_of_effect {
                        None => {
                            //single target
                            let idx = map.xy_idx(target.x, target.y);
                            for mob in map.tile_content[idx].iter() {
                                targets.push(*mob);
                            }
                        }
                        Some(area_of_effect) => {
                            let mut blast_tiles =
                                rltk::field_of_view(target, area_of_effect.radius, &*map);
                            // remove out of bounds tiles
                            blast_tiles.retain(|p| {
                                p.x > 0 && p.x < map.width - 1 && p.y > 0 && p.y < map.height - 1
                            });
                            // loop over remaining tiles...
                            for tile_idx in blast_tiles.iter() {
                                let idx = map.xy_idx(tile_idx.x, tile_idx.y);
                                // ...loop over all entities on a tile
                                for mob in map.tile_content[idx].iter() {
                                    targets.push(*mob);
                                }
                            }
                        }
                    }
                }
            }

            // Healing
            let item_heals = healing.get(use_item.item);
            match item_heals {
                None => {}
                Some(healer) => {
                    for target in targets.iter() {
                        let stats = combat_stats.get_mut(*target);
                        match stats {
                            None => {}
                            Some(stats) => {
                                stats.hp = i32::min(stats.max_hp, stats.hp + healer.heal_amount);
                                if entity == *player_entity {
                                    gamelog.entries.insert(
                                        0,
                                        format!(
                                            "You use the {}, healing {} hp.",
                                            names.get(use_item.item).unwrap().name,
                                            healer.heal_amount
                                        ),
                                    );
                                }
                            }
                        }
                    }
                }
            }

            // Damaging
            let item_damages = inflicts_damage.get(use_item.item);
            match item_damages {
                None => {}
                Some(damage) => {
                    used_item = false;
                    for mob in targets.iter() {
                        suffer_damage
                            .insert(
                                *mob,
                                SufferDamage {
                                    amount: damage.damage,
                                },
                            )
                            .expect("Unable to insert");
                        if entity == *player_entity {
                            let mob_name = names.get(*mob).unwrap();
                            let item_name = names.get(use_item.item).unwrap();
                            gamelog.entries.insert(
                                0,
                                format!(
                                    "You use {} on {}, inflicting {} damage.",
                                    item_name.name, mob_name.name, damage.damage
                                ),
                            );
                        }
                        used_item = true;
                    }
                }
            }
            // delete consumed items
            let consumable = consumables.get(use_item.item);
            if used_item {
                match consumable {
                    None => {}
                    Some(_) => {
                        entities.delete(use_item.item).expect("Delete failed");
                    }
                }
            }
        } // big for-loop ends here: (&entities, &wants_use).join

        wants_use.clear();
    }
}
