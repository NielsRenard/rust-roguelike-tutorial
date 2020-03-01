extern crate specs;
use super::{
    Confusion, Map, Monster, Name, ParticleBuilder, Point, Position, RunState, Viewshed,
    WantsToMelee,
};
use crate::color::*;
use specs::prelude::*;
extern crate rltk;

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    //    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, Point>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, RunState>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, WantsToMelee>,
        WriteStorage<'a, Confusion>,
        WriteExpect<'a, ParticleBuilder>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            player_pos,
            player_entity,
            runstate,
            entities,
            mut viewshed,
            monster,
            name,
            mut position,
            mut wants_to_melee,
            mut confused,
            mut particle_builder,
        ) = data;

        // only run system if the state is MonsterTurn
        if *runstate != RunState::MonsterTurn {
            return;
        }

        // "We also need to give the player a name;
        // we've explicitly included names in the AI's join, so we better be sure that the player has one!
        // Otherwise, the AI will ignore the player altogether." - Chapter 6

        for (entity, mut viewshed, _monster, _name, mut pos) in
            (&entities, &mut viewshed, &monster, &name, &mut position).join()
        {
            let mut can_act = true;
            let is_confused = confused.get_mut(entity);

            match is_confused {
                None => {}
                Some(i_am_confused) => {
                    i_am_confused.turns -= 1;
                    // count down until no longer confused
                    if i_am_confused.turns < 1 {
                        confused.remove(entity);
                    }
                    can_act = false;
                    particle_builder.request(
                        pos.x,
                        pos.y,
                        magenta(),
                        black(),
                        rltk::to_cp437('?'),
                        200.0,
                    )
                }
            }
            if can_act {
                let distance =
                    rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);
                if distance < 1.5 {
                    // Attack goes here
                    wants_to_melee
                        .insert(
                            entity,
                            WantsToMelee {
                                target: *player_entity,
                            },
                        )
                        .expect("Unable to insert attack");
                    return;
                } else if viewshed.visible_tiles.contains(&*player_pos) {
                    let path = rltk::a_star_search(
                        map.xy_idx(pos.x, pos.y) as i32,
                        map.xy_idx(player_pos.x, player_pos.y) as i32,
                        &mut *map,
                    );

                    if path.success && path.steps.len() > 1 {
                        let mut idx = map.xy_idx(pos.x, pos.y);
                        map.blocked_tiles[idx] = false;
                        pos.x = path.steps[1] as i32 % map.width;
                        pos.y = path.steps[1] as i32 / map.width;
                        idx = map.xy_idx(pos.x, pos.y);
                        map.blocked_tiles[idx] = true;
                        viewshed.dirty = true;
                    }
                }
            }
        }
    }
}
