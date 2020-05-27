extern crate specs;
use super::{
    EntityMoved, EntryTrigger, Hidden, InflictsDamage, Map, Name, ParticleBuilder, Position,
    SingleActivation, SufferDamage,
};
use crate::color::*;
use crate::gamelog::GameLog;
use specs::prelude::*;

pub struct TriggerSystem {}

impl<'a> System<'a> for TriggerSystem {
    type SystemData = (
        ReadExpect<'a, Map>,
        WriteStorage<'a, EntityMoved>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, EntryTrigger>,
        ReadStorage<'a, InflictsDamage>,
        WriteExpect<'a, ParticleBuilder>,
        WriteStorage<'a, SufferDamage>,
        WriteStorage<'a, Hidden>,
        ReadStorage<'a, Name>,
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, SingleActivation>,
    );
    // Iterate the entities that moved and their final position
    fn run(&mut self, data: Self::SystemData) {
        let (
            map,
            mut entity_moved,
            position,
            entry_trigger,
            inflicts_damage,
            mut particle_builder,
            mut inflict_damage,
            mut hidden,
            names,
            entities,
            mut log,
            single_activation,
        ) = data;

        let mut remove_entities: Vec<Entity> = Vec::new();
        // Iterate the entities that moved and their final position
        for (entity, mut _entity_moved, position) in
            (&entities, &mut entity_moved, &position).join()
        {
            let idx = map.xy_idx(position.x, position.y);
            for entity_id in map.tile_content[idx].iter() {
                if entity != *entity_id {
                    // Do not bother to check yourself for being a trap!
                    let maybe_trigger = entry_trigger.get(*entity_id);
                    match maybe_trigger {
                        None => {}
                        Some(_trigger) => {
                            // We triggered it
                            let name = names.get(*entity_id);
                            if let Some(name) = name {
                                log.entries.push(format!("{} triggers!", &name.name));
                            }
                            hidden.remove(*entity_id);
                            let damage = inflicts_damage.get(*entity_id);
                            if let Some(damage) = damage {
                                particle_builder.request(
                                    position.x,
                                    position.y,
                                    orange(),
                                    black(),
                                    rltk::to_cp437('‼'),
                                    200.0,
                                );
                                SufferDamage::new_damage(&mut inflict_damage, entity, damage.damage)
                            }
                            let single = single_activation.get(*entity_id);
                            if let Some(_single) = single {
                                remove_entities.push(*entity_id);
                            }
                        }
                    }
                }
            }
            for trap in remove_entities.iter() {
                entities.delete(*trap).expect("Unable to delete trap");
            }
        }
        // this clear() is on the EntityMoved "storage" from systemdata
        entity_moved.clear();
    }
}
