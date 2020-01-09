extern crate rltk;
extern crate specs;
use super::{CombatStats, Name, Player, Position, Renderable, Viewshed};
use crate::color::{black, yellow};
use specs::prelude::*;

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
