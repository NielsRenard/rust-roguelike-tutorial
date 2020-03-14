// https://bfnightly.bracketproductions.com/rustbook/chapter_11.html
extern crate specs;
use super::{Map, SerializeMe, MAP_COUNT};
use crate::components::*;
use specs::error::NoError;
use specs::prelude::*;
use specs::saveload::{
    DeserializeComponents, MarkedBuilder, SerializeComponents, SimpleMarker, SimpleMarkerAllocator,
};
use std::fs::File;
use std::path::Path;

pub fn save_exists() -> bool {
    return Path::new("savegame.json").exists();
}

pub fn delete_save() {
    if save_exists() {
        std::fs::remove_file("./savegame.json").expect("Unable to delete file");
    }
}

// https://doc.rust-lang.org/book/ch19-06-macros.html
macro_rules! serialize_individually {
    ($ecs:expr, $ser:expr, $data:expr, $( $type:ty),*) => {
        $(
        SerializeComponents::<NoError, SimpleMarker<SerializeMe>>::serialize(
            &( $ecs.read_storage::<$type>(), ),
            &$data.0,
            &$data.1,
            &mut $ser,
        )
        .unwrap();
        )*
    };
}

//This is pretty much the same as the serialize_individually macro - but reverses the process
macro_rules! deserialize_individually {
    ($ecs:expr, $de:expr, $data:expr, $( $type:ty),*) => {
        $(
        DeserializeComponents::<NoError, _>::deserialize(
            &mut ( &mut $ecs.write_storage::<$type>(), ),
            &mut $data.0, // entities
            &mut $data.1, // marker
            &mut $data.2, // allocater
            &mut $de,
        )
        .unwrap();
        )*
    };
}

// "only compile this for web assembly". We've kept the function
// signature the same, but added a _ before _ecs - telling the
// compiler that we intend not to use that variable. Then we keep the
// function empty.
#[cfg(target_arch = "wasm32")]
pub fn save_game(_ecs: &mut World) {
    // stub function for wasm because wasm is sandboxed.
    // future RLTK will use browser LocalStorage for savegames
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_game(ecs: &mut World) {
    // Create helper
    let mapcopy = ecs.get_mut::<Map>().unwrap().clone();
    let savehelper = ecs
        .create_entity()
        .with(SerializationHelper { map: mapcopy })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    // Actually serialize
    {
        let data = (
            ecs.entities(),
            ecs.read_storage::<SimpleMarker<SerializeMe>>(),
        );

        let writer = File::create("./savegame.json").unwrap();
        let mut serializer = serde_json::Serializer::new(writer);
        serialize_individually!(
            ecs,
            serializer,
            data,
            Position,
            Renderable,
            Player,
            Viewshed,
            Monster,
            Name,
            BlocksTile,
            CombatStats,
            SufferDamage,
            WantsToMelee,
            Item,
            Consumable,
            Ranged,
            InflictsDamage,
            AreaOfEffect,
            Confusion,
            ProvidesHealing,
            InBackpack,
            WantsToPickupItem,
            WantsToUseItem,
            WantsToDropItem,
            SerializationHelper,
            Equippable,
            Equipped,
            MeleePowerBonus,
            DefenseBonus,
            WantsToRemoveEquipment,
            Destructable,
            ParticleLifetime,
            HungerClock,
            ProvidesFood,
            MagicMapper
        );
    }

    // Clean up
    ecs.delete_entity(savehelper).expect("Crash on cleanup");
}

pub fn load_game(ecs: &mut World) {
    {
        // Delete everything
        let mut to_delete = Vec::new();
        for e in ecs.entities().join() {
            to_delete.push(e);
        }
        for del in to_delete.iter() {
            ecs.delete_entity(*del).expect("Deletion failed");
        }
    }

    let data = std::fs::read_to_string("./savegame.json").unwrap();
    let mut de = serde_json::Deserializer::from_str(&data);

    {
        //Then we build the tuple for the macro, which requires mutable access
        // to the entities store, write access to the marker store, and an
        // allocator (from Specs).
        let mut d = (
            &mut ecs.entities(),
            &mut ecs.write_storage::<SimpleMarker<SerializeMe>>(),
            &mut ecs.write_resource::<SimpleMarkerAllocator<SerializeMe>>(),
        );

        //Now we pass that to the macro, which calls the
        // de-serializer for each type in turn. Since we saved in the same
        // order, it will pick up everything.
        deserialize_individually!(
            ecs,
            de,
            d,
            Position,
            Renderable,
            Player,
            Viewshed,
            Monster,
            Name,
            BlocksTile,
            CombatStats,
            SufferDamage,
            WantsToMelee,
            Item,
            Consumable,
            Ranged,
            InflictsDamage,
            AreaOfEffect,
            Confusion,
            ProvidesHealing,
            InBackpack,
            WantsToPickupItem,
            WantsToUseItem,
            WantsToDropItem,
            SerializationHelper,
            Equippable,
            Equipped,
            MeleePowerBonus,
            DefenseBonus,
            WantsToRemoveEquipment,
            Destructable,
            ParticleLifetime,
            HungerClock,
            ProvidesFood,
            MagicMapper
        );
    }

    // Now we go into another block, to avoid borrow conflicts with
    // the previous code and the entity deletion.
    let mut deleteme: Option<Entity> = None;
    {
        // We first iterate all entities with a SerializationHelper type. If we
        // find it, we get access to the resource storing the map - and replace
        // it. Since we aren't serializing tile_content, we replace it with an
        // empty set of vectors.
        let entities = ecs.entities();
        let helper = ecs.read_storage::<SerializationHelper>();
        let player = ecs.read_storage::<Player>();
        let position = ecs.read_storage::<Position>();
        for (e, h) in (&entities, &helper).join() {
            let mut worldmap = ecs.write_resource::<Map>();
            *worldmap = h.map.clone();
            worldmap.tile_content = vec![Vec::new(); MAP_COUNT];
            deleteme = Some(e);
        }
        // Then we find the player, by iterating entities with a
        // Player type and a Position type. We store the world
        // resources for the player entity and his/her position.
        for (e, _p, pos) in (&entities, &player, &position).join() {
            let mut ppos = ecs.write_resource::<rltk::Point>();
            *ppos = rltk::Point::new(pos.x, pos.y);
            let mut player_resource = ecs.write_resource::<Entity>();
            *player_resource = e;
        }
    }
    ecs.delete_entity(deleteme.unwrap())
        .expect("Unable to delete helper");
}
