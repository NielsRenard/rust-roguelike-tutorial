extern crate rltk;
use rltk::{Console, Rltk, RGB};
extern crate specs;
use super::{CombatStats, Player};
use specs::prelude::*;

pub struct GameLog {
    pub entries: Vec<String>,
}
