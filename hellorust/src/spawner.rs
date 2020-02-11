extern crate rltk;
extern crate specs;
use super::color::*;
use super::map::MAP_WIDTH;
use super::{
    AreaOfEffect, BlocksTile, CombatStats, Confusion, Consumable, InflictsDamage, Item, Monster,
    Name, Player, Position, ProvidesHealing, RandomNumberGenerator, RandomTable, Ranged, Rect,
    Renderable, SerializeMe, Viewshed,
};
use specs::prelude::*;
use specs::saveload::{MarkedBuilder, SimpleMarker};
use std::collections::HashMap;

const MAX_MONSTERS: i32 = 4;

/// Spawns the player and returns its entity object.
pub fn player(ecs: &mut World, player_x: i32, player_y: i32) -> Entity {
    ecs.create_entity()
        .with(Position {
            x: player_x,
            y: player_y,
        })
        .with(Renderable {
            glyph: rltk::to_cp437('☺'),
            fg: yellow(),
            bg: black(),
            render_order: 0,
        })
        .with(Player {})
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Name {
            name: "Player".to_string(),
        })
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            strength: 5,
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}

fn room_table(map_depth: i32) -> RandomTable {
    RandomTable::new()
        .add("Confusion Scroll", 2 + map_depth)
        .add("Fireball Scroll", 2 + map_depth)
        .add("Goblin", 10)
        .add("Health Potion", 7)
        .add("Magic Missile Scroll", 4)
        .add("Orc", 1 + map_depth)
}

pub fn orc(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, rltk::to_cp437('o'), "Orc");
}
pub fn goblin(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, rltk::to_cp437('g'), "Goblin");
}

pub fn monster<S: ToString>(ecs: &mut World, x: i32, y: i32, glyph: u8, name: S) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: glyph,
            fg: red(),
            bg: black(),
            render_order: 1,
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Monster {})
        .with(Name {
            name: name.to_string(),
        })
        .with(BlocksTile {})
        .with(CombatStats {
            max_hp: 16,
            hp: 16,
            defense: 1,
            strength: 4,
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

pub fn spawn_room(ecs: &mut World, room: &Rect, map_depth: i32) {
    let spawn_table = room_table(map_depth);
    let mut spawn_points: HashMap<usize, String> = HashMap::new();

    // "Scope to keep the borrow checker happy"
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_spawns = rng.roll_dice(1, MAX_MONSTERS + 3) + (map_depth - 1) - 3;

        for _i in 0..num_spawns {
            let mut added = false;
            let mut tries = 0;
            // "keep trying to add random positions that aren't already occupied
            // until sufficient monsters have been created"
            while !added && tries < 20 {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAP_WIDTH) + x;
                if !spawn_points.contains_key(&idx) {
                    spawn_points.insert(idx, spawn_table.roll(&mut rng));
                    added = true;
                }
            }
        }
    }
    // "Actually spawn the monsters"
    for spawn in spawn_points.iter() {
        let x = (*spawn.0 % MAP_WIDTH) as i32;
        let y = (*spawn.0 / MAP_WIDTH) as i32;

        match spawn.1.as_ref() {
            "Confusion Scroll" => confusion_scroll(ecs, x, y),
            "Fireball Scroll" => fireball_scroll(ecs, x, y),
            "Goblin" => goblin(ecs, x, y),
            "Health Potion" => health_potion(ecs, x, y),
            "Magic Missile Scroll" => magic_missile_scroll(ecs, x, y),
            "Orc" => orc(ecs, x, y),
            _ => {}
        }
    }
}

pub fn health_potion(ecs: &mut World, x: i32, y: i32) {
    let glyph = rltk::to_cp437('¡');
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: glyph,
            fg: magenta(),
            bg: black(),
            render_order: 2,
        })
        .with(Name {
            name: "Health Potion".to_string(),
        })
        .with(Item {})
        .with(ProvidesHealing { heal_amount: 8 })
        .with(Consumable {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

pub fn magic_missile_scroll(ecs: &mut World, x: i32, y: i32) {
    let glyph = rltk::to_cp437(')');
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: glyph,
            fg: cyan(),
            bg: black(),
            render_order: 2,
        })
        .with(Name {
            name: "Magic Missile Scroll".to_string(),
        })
        .with(Item {})
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 8 })
        .with(Consumable {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

pub fn fireball_scroll(ecs: &mut World, x: i32, y: i32) {
    let glyph = rltk::to_cp437(')');
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: glyph,
            fg: orange(),
            bg: black(),
            render_order: 2,
        })
        .with(Name {
            name: "Fireball scroll".to_string(),
        })
        .with(Item {})
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 20 })
        .with(AreaOfEffect { radius: 3 })
        .with(Consumable {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

pub fn confusion_scroll(ecs: &mut World, x: i32, y: i32) {
    let glyph = rltk::to_cp437(')');
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: glyph,
            fg: pink(),
            bg: black(),
            render_order: 2,
        })
        .with(Name {
            name: "Confusion scroll".to_string(),
        })
        .with(Item {})
        .with(Ranged { range: 6 })
        .with(Confusion { turns: 4 })
        .with(Consumable {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
