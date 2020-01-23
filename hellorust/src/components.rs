use rltk::RGB;
use serde::{Deserialize, Serialize};
use specs::error::NoError;
use specs::prelude::*;
use specs::saveload::{ConvertSaveload, Marker};

/*
 `Component`. You can replace this with writing code specifying
 Specs storage if you prefer, but the macro is much easier!

 `ConvertSaveload` adds Serialize and Deserialize, but with extra
 conversion for any Entity classes it encounters.

 `Clone` is saying
 "this structure can be copied in memory from one point to another."
 This is necessary for the inner-workings of Serde, and also allows
 you to attach .clone() to the end of any reference to a component -
 and get another, perfect copy of it. In most cases, clone is really
 fast (and occasionally the compiler can make it do nothing at
 all!)
*/

/*
When you have a component with no data, the ConvertSaveload macro
doesn't work! Fortunately, these don't require any additional
conversion - so you can fall back to the default Serde syntax. Check Player for
a non-data ("tag") class.
*/

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Player {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Monster {}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Name {
    pub name: String,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToMelee {
    pub target: Entity,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct SufferDamage {
    pub amount: i32,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub strength: i32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct BlocksTile {}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

/// What you see from a specific place.
#[derive(Component, ConvertSaveload, Clone)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
    pub dirty: bool,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Renderable {
    pub glyph: u8,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Item {}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct InBackpack {
    pub owner: Entity,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToPickupItem {
    pub collected_by: Entity,
    pub item: Entity,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToDropItem {
    pub item: Entity,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToUseItem {
    pub item: Entity,
    pub target: Option<rltk::Point>,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Consumable {}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct ProvidesHealing {
    pub heal_amount: i32,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Ranged {
    pub range: i32,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct InflictsDamage {
    pub damage: i32,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct AreaOfEffect {
    pub radius: i32,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Confusion {
    pub turns: i32,
}

// "Serialization helper code. We need to implement ConvertSaveload for each type that contains an
// Entity."

pub struct SerializeMe;
