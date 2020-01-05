extern crate specs;
use super::{Monster, Position, Viewshed};
use specs::prelude::*;
use std::time::{SystemTime};
extern crate rltk;

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Monster>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (viewshed, pos, monster) = data;

        for (_viewshed, _pos, _monster) in (&viewshed, &pos, &monster).join() {
	    let now = SystemTime::now();
	    let thought = "Monster considers their own existence";
	    let message = format!("{:?}{}", now, thought);
	    // println! to console, or when in WASM -> log to browser console
	    rltk::console::log(message);
        }
    }
}
