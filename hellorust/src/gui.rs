extern crate rltk;
use rltk::{Console, Rltk, RGB};
extern crate specs;
use super::{CombatStats, Player};
use crate::gamelog::GameLog;
use specs::prelude::*;

// TODO: replace 43 and 79 with const's (maybe directly couple to the ones in map.rs)

pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    ctx.draw_box(
        0,
        43,
        79,
        6,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );
    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();
    let log = ecs.fetch::<GameLog>();

    //GameLog message printing
    let mut y = 44;
    for message in log.entries.iter() {
        if y < 49 {
            ctx.print(2, y, &message.to_string());
        }
        y += 1;
    }

    for (_player, stats) in (&players, &combat_stats).join() {
        let health = format!(" HP: {}/{} ", stats.hp, stats.max_hp);
        ctx.print_color(
            12,
            43,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            &health,
        );
        ctx.draw_bar_horizontal(
            28,
            43,
            51,
            stats.hp,
            stats.max_hp,
            RGB::named(rltk::RED),
            RGB::named(rltk::BLACK),
        )
    }
}
