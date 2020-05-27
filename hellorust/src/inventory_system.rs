extern crate specs;
use super::{
    gamelog::GameLog, AreaOfEffect, CombatStats, Confusion, Consumable, Destructable, Equippable,
    Equipped, HungerClock, HungerState, InBackpack, InflictsDamage, MagicMapper, Map, Name,
    ParticleBuilder, Position, ProvidesFood, ProvidesHealing, RunState, SufferDamage,
    WantsToDropItem, WantsToPickupItem, WantsToRemoveEquipment, WantsToUseItem,
};
use crate::color::*;
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
                gamelog.entries.push(format!(
                    "You pick up the {}.",
                    names.get(pickup.item).unwrap().name
                ));
            }
        }
        wants_pickup.clear();
    }
}

pub struct EquipmentRemoveSystem {}

impl<'a> System<'a> for EquipmentRemoveSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToRemoveEquipment>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut wants_to_remove, mut equipped, mut backpack) = data;

        for (entity, to_remove) in (&entities, &wants_to_remove).join() {
            equipped.remove(to_remove.item);
            backpack
                .insert(to_remove.item, InBackpack { owner: entity })
                .expect("Unable to insert backpack");
        }

        wants_to_remove.clear();
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
                gamelog.entries.push(format!(
                    "You drop the {}.",
                    names.get(to_drop.item).unwrap().name
                ));
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
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, InflictsDamage>,
        ReadStorage<'a, AreaOfEffect>,
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        WriteStorage<'a, Confusion>,
        ReadStorage<'a, Equippable>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
        WriteStorage<'a, Destructable>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, ProvidesFood>,
        WriteStorage<'a, HungerClock>,
        ReadStorage<'a, MagicMapper>,
        WriteExpect<'a, RunState>,
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
            mut confused,
            equippable,
            mut equipped,
            mut in_backpack,
            mut destructable,
            mut particle_builder,
            positions,
            provides_food,
            mut hunger_clocks,
            magic_mapper,
            mut runstate,
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
                                particle_builder.request(
                                    tile_idx.x,
                                    tile_idx.y,
                                    orange(),
                                    black(),
                                    rltk::to_cp437('░'),
                                    250.0,
                                );
                            }
                        }
                    }
                }
            }

            // Equipping (replacing active equipped item)
            let item_equippable = equippable.get(use_item.item);
            match item_equippable {
                None => {}
                Some(can_equip) => {
                    let target_slot = can_equip.slot;
                    let target = targets[0];

                    // Mark active equipped item as to_unequip
                    let mut to_unequip: Vec<Entity> = Vec::new();
                    for (item_entity, currently_equipped, name) in
                        (&entities, &equipped, &names).join()
                    {
                        if currently_equipped.owner == target
                            && currently_equipped.slot == target_slot
                        {
                            to_unequip.push(item_entity);
                            if target == *player_entity {
                                gamelog
                                    .entries
                                    .push(format!("You unequip the {}.", name.name));
                            }
                        }
                    }
                    // Actually unequip the item
                    for item in to_unequip.iter() {
                        equipped.remove(*item);
                        in_backpack
                            .insert(*item, InBackpack { owner: target })
                            .expect("Unable to insert backpack entry");
                    }

                    // Equip the item
                    equipped
                        .insert(
                            use_item.item,
                            Equipped {
                                owner: target,
                                slot: target_slot,
                            },
                        )
                        .expect("Unable to insert equipped component");
                    in_backpack.remove(use_item.item);
                    if target == *player_entity {
                        gamelog.entries.push(format!(
                            "You equip the {}.",
                            names.get(use_item.item).unwrap().name
                        ));
                    }
                }
            }

            // Edibles reset hungerclock
            let item_edible = provides_food.get(use_item.item);
            match item_edible {
                None => {}
                Some(_edible) => {
                    used_item = true;
                    let target = targets[0]; // the player
                    let hunger_clock = hunger_clocks.get_mut(target);
                    if let Some(hc) = hunger_clock {
                        hc.state = HungerState::WellFed;
                        hc.duration = 20;
                        gamelog.entries.push(format!(
                            "You eat the {}.",
                            names.get(use_item.item).unwrap().name
                        ));
                    }
                }
            }

            // If its a Magic Mapper
            let is_magic_mapper = magic_mapper.get(use_item.item);
            match is_magic_mapper {
                None => {}
                Some(_) => {
                    used_item = true;
                    gamelog.entries.push("All is revealed to you!".to_string());
                    *runstate = RunState::MagicMapReveal { row: 0 };
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
                                    gamelog.entries.push(format!(
                                        "You use the {}, healing {} hp.",
                                        names.get(use_item.item).unwrap().name,
                                        healer.heal_amount
                                    ));
                                }
                                let pos = positions.get(*player_entity);
                                if let Some(pos) = pos {
                                    particle_builder.request(
                                        pos.x,
                                        pos.y,
                                        green(),
                                        black(),
                                        rltk::to_cp437('♥'),
                                        200.0,
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
                    for target in targets.iter() {
                        match destructable.get(*target) {
                            None => {}
                            Some(_destructable) => {
                                destructable
                                    .insert(*target, Destructable { broken: true })
                                    .expect("Unable to insert");
                            }
                        }

                        SufferDamage::new_damage(&mut suffer_damage, *target, damage.damage);

                        if entity == *player_entity {
                            let target_name = names.get(*target).unwrap();
                            let item_name = names.get(use_item.item).unwrap();
                            gamelog.entries.push(format!(
                                "You use {} on {}, inflicting {} damage.",
                                item_name.name, target_name.name, damage.damage
                            ));
                        }
                        used_item = true;
                        let pos = positions.get(*target);
                        if let Some(pos) = pos {
                            particle_builder.request(
                                pos.x,
                                pos.y,
                                red(),
                                black(),
                                rltk::to_cp437('‼'),
                                200.0,
                            );
                        }
                    }
                }
            }

            // "Can it pass along confusion? Note the use of scopes to escape from the borrow checker!"
            let mut add_confusion = Vec::new();
            {
                let causes_confusion = confused.get(use_item.item);
                match causes_confusion {
                    None => {}
                    Some(confusion) => {
                        used_item = false;
                        for mob in targets.iter() {
                            add_confusion.push((*mob, confusion.turns));
                            if entity == *player_entity {
                                let mob_name = names.get(*mob).unwrap();
                                let item_name = names.get(use_item.item).unwrap();
                                gamelog.entries.push(format!(
                                    "You use {} on {}, confusing them.",
                                    item_name.name, mob_name.name
                                ));
                            }
                            let pos = positions.get(*mob);
                            if let Some(pos) = pos {
                                particle_builder.request(
                                    pos.x,
                                    pos.y,
                                    magenta(),
                                    black(),
                                    rltk::to_cp437('?'),
                                    1000.0,
                                );
                            }
                        }
                    }
                }
            }
            for mob in add_confusion.iter() {
                confused
                    .insert(mob.0, Confusion { turns: mob.1 })
                    .expect("Unable to insert status");
                used_item = true;
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
