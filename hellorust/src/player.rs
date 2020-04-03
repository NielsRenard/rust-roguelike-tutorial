use super::{
    gamelog::GameLog, CombatStats, EntityMoved, HungerClock, HungerState, Item, Map, Monster,
    Player, Point, Position, RunState, State, TileType, Viewshed, WantsToMelee, WantsToPickupItem,
};
use rltk::{Rltk, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let entities = ecs.entities();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let _map = ecs.fetch::<Map>();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let mut entity_moved = ecs.write_storage::<EntityMoved>();

    let map = ecs.fetch::<Map>();

    for (entity, _player, pos, viewshed) in
        (&entities, &mut players, &mut positions, &mut viewsheds).join()
    {
        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

        //"Bump to attack (walking into the target)"
        for potential_target in map.tile_content[destination_idx].iter() {
            let target = combat_stats.get(*potential_target);
            match target {
                None => {}
                Some(_target) => {
                    // Attack it
                    wants_to_melee
                        .insert(
                            entity,
                            WantsToMelee {
                                target: *potential_target,
                            },
                        )
                        .expect("Add target failed");
                    return; // So we don't move after attacking
                }
            }
        }

        if !map.blocked_tiles[destination_idx] {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));
            entity_moved
                .insert(entity, EntityMoved {})
                .expect("Unable to insert marker");

            viewshed.dirty = true;

            //update player position resource
            let mut player_pos = ecs.write_resource::<Point>();
            player_pos.x = pos.x;
            player_pos.y = pos.y;
        }
    }
}

fn get_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    let mut target_item: Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if player_pos.x == position.x && player_pos.y == position.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => gamelog.entries.push(String::from("Nothing to pick up")),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup
                .insert(
                    *player_entity,
                    WantsToPickupItem {
                        collected_by: *player_entity,
                        item: item,
                    },
                )
                .expect("Unable to insert WantsToPickupItem");
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    // Player movement
    match ctx.key {
        None => return RunState::AwaitingInput, // Nothing happened, don't Tick yet.
        Some(key) => match key {
            VirtualKeyCode::Period => {
                if try_next_level(&mut gs.ecs) {
                    return RunState::NextLevel;
                }
            }
            VirtualKeyCode::O => return RunState::ShowDropItem,
            VirtualKeyCode::I => return RunState::ShowInventory,
            VirtualKeyCode::G => get_item(&mut gs.ecs),
            VirtualKeyCode::R => return RunState::ShowRemoveItem,
            // Skip turn
            VirtualKeyCode::Numpad5 => return skip_turn(&mut gs.ecs),
            VirtualKeyCode::Space => return skip_turn(&mut gs.ecs),
            // Cardinal movement
            VirtualKeyCode::Left | VirtualKeyCode::A | VirtualKeyCode::Numpad4 => {
                try_move_player(-1, 0, &mut gs.ecs)
            }
            VirtualKeyCode::Right | VirtualKeyCode::D | VirtualKeyCode::Numpad6 => {
                try_move_player(1, 0, &mut gs.ecs)
            }
            VirtualKeyCode::Up | VirtualKeyCode::W | VirtualKeyCode::Numpad8 => {
                try_move_player(0, -1, &mut gs.ecs)
            }
            VirtualKeyCode::Down | VirtualKeyCode::S | VirtualKeyCode::Numpad2 => {
                try_move_player(0, 1, &mut gs.ecs)
            }
            // Diagonal movement
            VirtualKeyCode::Q | VirtualKeyCode::Numpad7 => try_move_player(-1, -1, &mut gs.ecs),
            VirtualKeyCode::E | VirtualKeyCode::Numpad9 => try_move_player(1, -1, &mut gs.ecs),
            VirtualKeyCode::Z | VirtualKeyCode::Numpad1 => try_move_player(-1, 1, &mut gs.ecs),
            VirtualKeyCode::C | VirtualKeyCode::Numpad3 => try_move_player(1, 1, &mut gs.ecs),
            // Saving
            VirtualKeyCode::Escape => return RunState::SaveGame,
            // Nothing pressed, do nothing
            _ => return RunState::AwaitingInput,
        },
    }
    // If a button was pressed, the next Tick may occur.
    return RunState::PlayerTurn;
}

fn skip_turn(ecs: &mut World) -> RunState {
    let player_entity = ecs.fetch::<Entity>();
    let viewshed_components = ecs.read_storage::<Viewshed>();
    let monsters = ecs.read_storage::<Monster>();

    let worldmap_resource = ecs.fetch::<Map>();
    let mut can_heal = true;
    let viewshed = viewshed_components.get(*player_entity).unwrap();

    // Don't let resting heal when hungry
    let hunger_clocks = ecs.read_storage::<HungerClock>();
    let hunger_clock = hunger_clocks.get(*player_entity);
    if let Some(hunger_clock) = hunger_clock {
        match hunger_clock.state {
            HungerState::Hungry => can_heal = false,
            HungerState::Starving => can_heal = false,
            _ => {}
        }
    }

    for tile in viewshed.visible_tiles.iter() {
        let idx = worldmap_resource.xy_idx(tile.x, tile.y);
        for entity_id in worldmap_resource.tile_content[idx].iter() {
            let mob = monsters.get(*entity_id);
            // Check visible tiles for monsters, if present then can't heal
            match mob {
                None => {}
                Some(_) => {
                    can_heal = false;
                }
            }
        }
    }

    if can_heal {
        let mut health_components = ecs.write_storage::<CombatStats>();
        let player_health = health_components.get_mut(*player_entity).unwrap();
        // "makes the game really easy - but we're getting to that!"
        player_health.hp = i32::min(player_health.hp + 1, player_health.max_hp);
    }

    return RunState::PlayerTurn;
}

pub fn try_next_level(ecs: &mut World) -> bool {
    let player_pos = ecs.fetch::<Point>();
    let map = ecs.fetch::<Map>();
    let player_idx = map.xy_idx(player_pos.x, player_pos.y);
    if map.tiles[player_idx] == TileType::DownStairs {
        true
    } else {
        let mut gamelog = ecs.fetch_mut::<GameLog>();
        gamelog
            .entries
            .push("There is no way down from here.".to_string());
        false
    }
}
