extern crate specs;
use super::{BlocksTile, Map, Player, Position};
use specs::prelude::*;

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, BlocksTile>,
        ReadStorage<'a, Player>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, position, blockers, player, entities) = data;

        map.populate_blocked();
        map.clear_content_index();

        for (entity, position) in (&entities, &position).join() {
            let idx = map.xy_idx(position.x, position.y);

            // If they block, update the blocking list
            let _b: Option<&BlocksTile> = blockers.get(entity);
            if let Some(_p) = _b {
                map.blocked_tiles[idx] = true;
            }

            // Push the player into the tile_content stack. fixes
            // splash damage bug where player took no damage
            let _p: Option<&Player> = player.get(entity);
            if let Some(_player) = player.get(entity) {
                map.tile_content[idx].push(entity);
            }

            // Push the entity to the appropriate index slot. It's a Copy
            // type, so we don't need to clone it (we want to avoid moving it out of the ECS!)
            map.tile_content[idx].push(entity);
        }

        //        for (position, _blocks) in (&position, &blockers).join() {
        //            let idx = map.xy_idx(position.x, position.y);
        //            map.blocked_tiles[idx] = true;
        //        }
    }
}
