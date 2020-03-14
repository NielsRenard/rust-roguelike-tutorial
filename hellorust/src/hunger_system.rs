extern crate specs;
use crate::components::SufferDamage;
use crate::components::{HungerClock, HungerState::*};
use crate::gamelog::GameLog;
use crate::RunState;
use specs::prelude::*;

pub struct HungerSystem {}
pub const STARVATION_DAMAGE: i32 = 1;

impl<'a> System<'a> for HungerSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, HungerClock>,
        ReadExpect<'a, Entity>, // The player
        ReadExpect<'a, RunState>,
        WriteStorage<'a, SufferDamage>,
        WriteExpect<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut hunger_clock, player_entity, runstate, mut inflict_damage, mut log) =
            data;

        for (entity, mut clock) in (&entities, &mut hunger_clock).join() {
            let mut proceed = false;

            match *runstate {
                RunState::PlayerTurn => {
                    if entity == *player_entity {
                        proceed = true;
                    }
                }
                RunState::MonsterTurn => {
                    if entity != *player_entity {
                        proceed = true;
                    }
                }
                _ => proceed = false,
            }
            if proceed {
                clock.duration -= 1;
                if clock.duration < 1 {
                    match clock.state {
                        WellFed => {
                            clock.state = Normal;
                            clock.duration = 100;
                            if entity == *player_entity {
                                log.entries
                                    .insert(0, "You no longer feel well fed.".to_string());
                            }
                        }
                        Normal => {
                            clock.state = Hungry;
                            clock.duration = 100;
                            if entity == *player_entity {
                                log.entries.insert(0, "You feel hungry.".to_string());
                            }
                        }
                        Hungry => {
                            clock.state = Starving;
                            clock.duration = 100;
                            if entity == *player_entity {
                                log.entries.insert(0, "You are starving.".to_string());
                            }
                        }
                        Starving => {
                            if entity == *player_entity {
                                log.entries.insert(
                                    0,
                                    format!(
                                        "You lose {} health from starvation.",
                                        STARVATION_DAMAGE
                                    ),
                                );
                                SufferDamage::new_damage(
                                    &mut inflict_damage,
                                    entity,
                                    STARVATION_DAMAGE,
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}
