extern crate specs;
use super::{Map, Player, Position, Viewshed};
use specs::prelude::*;
extern crate rltk;
use rltk::{field_of_view, Point};
use std::env;

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Player>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let map_hack = env::var("MAP_HACK").is_ok();
        let (mut map, entities, mut viewshed, pos, player) = data;

        if map_hack {
            // set all tiles visible and revealed and return early
            for t in map.visible_tiles.iter_mut() {
                *t = true
            }
            for t in map.revealed_tiles.iter_mut() {
                *t = true
            }
            return ();
        }
        for (ent, viewshed, pos) in (&entities, &mut viewshed, &pos).join() {
            if viewshed.dirty {
                viewshed.visible_tiles.clear();
                viewshed.visible_tiles =
                    field_of_view(Point::new(pos.x, pos.y), viewshed.range, &*map);
                viewshed
                    .visible_tiles
                    .retain(|p| p.x > 0 && p.x < map.width - 1 && p.y > 0 && p.y < map.height - 1);

                // If this is the player, reveal what they can see
                let _p: Option<&Player> = player.get(ent);
                match _p {
                    Some(_p) => {
                        for t in map.visible_tiles.iter_mut() {
                            *t = false;
                        }

                        for tile in viewshed.visible_tiles.iter() {
                            let idx = map.xy_idx(tile.x, tile.y);
                            map.revealed_tiles[idx] = true;
                            map.visible_tiles[idx] = true;
                        }
                    }
                    None => (),
                }
            }
        }
    }
}
