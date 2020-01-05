extern crate specs;
use super::{Map, Monster, Position, Viewshed};
use specs::prelude::*;
extern crate rltk;
use rltk::{console, field_of_view, Point};

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Monster>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (viewshed, pos, monster) = data;

        for (viewshed, pos, _monster) in (&viewshed, &pos, &monster).join() {
	    // println! to console, or when in WASM -> log to browser console
	    rltk::console::log("Monster considers their own existence");
        }
    }
}
