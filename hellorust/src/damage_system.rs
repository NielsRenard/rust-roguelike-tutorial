extern crate specs;
use super::{CombatStats, Name, Player, SufferDamage};
use crate::gamelog::GameLog;
use specs::prelude::*;

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut stats, mut damage) = data;

        for (mut stats, damage) in (&mut stats, &damage).join() {
            stats.hp -= damage.amount;
        }

        damage.clear();
    }
}
pub fn delete_the_dead(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();
    // Using a scope to make the borrow checker happy
    {
        let combat_stats = ecs.read_storage::<CombatStats>();
        let names = ecs.read_storage::<Name>();
        let players = ecs.read_storage::<Player>();
        let entities = ecs.entities();
        let mut log = ecs.write_resource::<GameLog>();

        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hp < 1 {
                let player = players.get(entity);
                match player {
                    Some(_) => log.entries.insert(0, String::from("You are dead")),
                    None => {
                        // TODO: not too stoked on this nested match expression
                        match names.get(entity) {
                            Some(victim_name) => log
                                .entries
                                .insert(0, format!("{} was slain", victim_name.name)),
                            None => {
                                // TODO: leaving this in for now but
                                // entities with no name probably shouldn't happen
                                log.entries.insert(0, format!("unknown entity was slain"))
                            }
                        }
                        dead.push(entity)
                    }
                }
            }
        }
    }

    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete");
    }
}
