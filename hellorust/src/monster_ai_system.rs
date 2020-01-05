extern crate specs;
use super::{Monster, Name, Point, Viewshed};
use specs::prelude::*;
extern crate rltk;

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        ReadExpect<'a, Point>,
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Name>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_pos, viewshed, monster, name) = data;

        // "We also need to give the player a name;
        // we've explicitly included names in the AI's join, so we better be sure that the player has one!
        // Otherwise, the AI will ignore the player altogether." - Chapter 6

        for (viewshed, _monster, name) in (&viewshed, &monster, &name).join() {
            // TODO don't understand why book dereferences player_pos here
            if viewshed.visible_tiles.contains(&*player_pos) {
                rltk::console::log(format!("{:?} {}", name.name, "winks"));
                // for WASM, this logs to browser console
            }
        }
    }
}
