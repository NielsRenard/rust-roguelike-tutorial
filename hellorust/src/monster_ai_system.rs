extern crate specs;
use super::{Monster, Point, Viewshed};
use specs::prelude::*;
use std::time::SystemTime;
extern crate rltk;

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        ReadExpect<'a, Point>,
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_pos, viewshed, monster) = data;

        for (viewshed, _monster) in (&viewshed, &monster).join() {
            // TODO don't understand why book dereferences player_pos here
            if viewshed.visible_tiles.contains(&*player_pos) {
                let thought = "Monster winks";
                rltk::console::log(format!("{:?}{}", SystemTime::now(), thought));
                // for WASM logs to browser console
            }
        }
    }
}
