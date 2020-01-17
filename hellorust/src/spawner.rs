extern crate rltk;
extern crate specs;
use super::color::{black, cyan, magenta, red, yellow};
use super::map::MAP_WIDTH;
use super::{
    BlocksTile, CombatStats, Consumable, InflictsDamage, Item, Monster, Name, Player, Position,
    ProvidesHealing, RandomNumberGenerator, Ranged, Rect, Renderable, Viewshed,
};
use specs::prelude::*;

const MAX_MONSTERS: i32 = 4;
const MAX_ITEMS: i32 = 2;

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
        .build()
}

// Spawns a random item at a given location
pub fn random_item(ecs: &mut World, x: i32, y: i32) {
    let roll: i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 2);
    }
    match roll {
        1 => health_potion(ecs, x, y),
        _ => magic_missile_scroll(ecs, x, y),
    }
}

/// Spawns a random monster at a given location
pub fn random_monster(ecs: &mut World, x: i32, y: i32) {
    let roll: i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 2);
    }
    match roll {
        1 => {
            goblin(ecs, x, y);
        }
        _ => {
            orc(ecs, x, y);
        }
    }
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
        .build();
}

pub fn spawn_room(ecs: &mut World, room: &Rect) {
    let mut monster_spawn_points: Vec<usize> = Vec::new();
    let mut item_spawn_points: Vec<usize> = Vec::new();

    // "Scope to keep the borrow checker happy"
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_monsters = rng.roll_dice(1, MAX_MONSTERS + 2) - 3;
        let num_items = rng.roll_dice(1, MAX_ITEMS + 1); // + 2) - 3;

        for _i in 0..num_monsters {
            let mut added = false;
            // "keep trying to add random positions that aren't already occupied
            // until sufficient monsters have been created"
            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAP_WIDTH) + x;
                if !monster_spawn_points.contains(&idx) {
                    monster_spawn_points.push(idx);
                    added = true;
                }
            }
        }
        for _i in 0..num_items {
            let mut added = false;
            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAP_WIDTH) + x;
                if !item_spawn_points.contains(&idx) {
                    item_spawn_points.push(idx);
                    added = true;
                }
            }
        }
    }
    // "Actually spawn the monsters"
    for idx in monster_spawn_points.iter() {
        let x = *idx % MAP_WIDTH;
        let y = *idx / MAP_WIDTH;
        random_monster(ecs, x as i32, y as i32);
    }
    // "Actually spawn the items"
    for idx in item_spawn_points.iter() {
        let x = *idx % MAP_WIDTH;
        let y = *idx / MAP_WIDTH;
        random_item(ecs, x as i32, y as i32);
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
        .build();
}
