extern crate specs;
use super::Map;
use specs::prelude::*;

pub fn save_game(ecs: &mut World) {
    let data = serde_json::to_string(&*ecs.fetch::<Map>()).unwrap();
    println!("{}", data);
}
