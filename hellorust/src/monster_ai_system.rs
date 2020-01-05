extern crate specs;
use super::{Map, Monster, Name, Point, Position, Viewshed};
use specs::prelude::*;
extern crate rltk;

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    //    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, Point>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, player_pos, mut viewshed, monster, name, mut position) = data;

        // "We also need to give the player a name;
        // we've explicitly included names in the AI's join, so we better be sure that the player has one!
        // Otherwise, the AI will ignore the player altogether." - Chapter 6

        for (viewshed, _monster, name, mut pos) in
            (&mut viewshed, &monster, &name, &mut position).join()
        {
            let distance =
                rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);
            if distance < 1.5 {
                // Attack goes here
                rltk::console::log(&format!("{} shouts insults", name.name));
                return;
            }
            // TODO don't understand why book dereferences player_pos here
            else if viewshed.visible_tiles.contains(&*player_pos) {
                // (for WASM, this logs to browser console)
                rltk::console::log(format!("{:?} {}", name.name, "winks"));
                let path = rltk::a_star_search(
                    map.xy_idx(pos.x, pos.y) as i32,
                    map.xy_idx(player_pos.x, player_pos.y) as i32,
                    &mut *map,
                );

                if path.success && path.steps.len() > 1 {
                    pos.x = path.steps[1] % map.width;
                    pos.y = path.steps[1] / map.width;
                    viewshed.dirty = true;
                }
            }
        }
    }
}
