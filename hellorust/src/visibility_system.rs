extern crate specs;
use super::{Hidden, Map, Name, Player, Position, Viewshed};
use crate::gamelog::GameLog;
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
        WriteStorage<'a, Hidden>,
        WriteExpect<'a, rltk::RandomNumberGenerator>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, Name>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, entities, mut viewshed, pos, player, mut hidden, mut rng, mut log, name) =
            data;

        // set all tiles visible and revealed and return early
        if env::var("MAP_HACK").is_ok() {
            map.visible_tiles.iter_mut().for_each(|t| *t = true);
            map.revealed_tiles.iter_mut().for_each(|t| *t = true);
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
                            for entity in map.tile_content[idx].iter() {
                                let maybe_hidden = hidden.get(*entity);
                                if let Some(_maybe_hidden) = maybe_hidden {
                                    if rng.roll_dice(1, 24) == 1 {
                                        let maybe_name = name.get(*entity);
                                        if let Some(name) = maybe_name {
                                            log.entries
                                                .push(format!("You spotted a {}.", &name.name));
                                        }
                                        hidden.remove(*entity);
                                    }
                                }
                            }
                        }
                    }
                    None => (),
                }
            }
        }
    }
}
