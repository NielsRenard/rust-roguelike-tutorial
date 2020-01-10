extern crate rltk;
use rltk::{Console, Rltk, VirtualKeyCode};
extern crate specs;
use super::{CombatStats, InBackpack, Map, Name, Player, Point, Position, State};
use crate::color::{black, grey, magenta, red, white, yellow};
use crate::gamelog::GameLog;
use specs::prelude::*;

// TODO: replace 43 and 79 with const's (maybe directly couple to the ones in map.rs)

pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    ctx.draw_box(0, 43, 79, 6, white(), black());
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
        ctx.print_color(12, 43, white(), black(), &health);
        ctx.draw_bar_horizontal(28, 43, 51, stats.hp, stats.max_hp, red(), black())
    }
    let mouse_pos = ctx.mouse_pos();
    ctx.set_bg(mouse_pos.0, mouse_pos.1, magenta());
    draw_tooltips(ecs, ctx);
}

fn draw_tooltips(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();

    let mouse_pos = ctx.mouse_pos();
    if mouse_pos.0 >= map.width || mouse_pos.1 >= map.height {
        return;
    }

    let mut tooltip: Vec<String> = Vec::new();
    for (name, position) in (&names, &positions).join() {
        if position.x == mouse_pos.0 && position.y == mouse_pos.1 {
            tooltip.push(name.name.to_string());
        }
    }

    if !tooltip.is_empty() {
        let mut width: i32 = 0;
        if !tooltip.is_empty() {
            for tip in tooltip.iter() {
                if width < tip.len() as i32 {
                    width = tip.len() as i32;
                }
            }
        }
        width += 3;
        //  if mouse is on the left, tooltip goes right, and vice versa
        if mouse_pos.0 > 40 {
            let arrow_pos = Point::new(mouse_pos.0 - 2, mouse_pos.1);
            let left_x = mouse_pos.0 - width;

            let mut y = mouse_pos.1;
            for tip in tooltip.iter() {
                ctx.print_color(left_x, y, white(), grey(), &tip.to_string());
                let padding = (width - tip.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(arrow_pos.x - i, y, white(), grey(), &" ".to_string());
                }
                y += 1;
            }
            ctx.print_color(arrow_pos.x, arrow_pos.y, white(), grey(), &"->".to_string());
        } else {
            let arrow_pos = Point::new(mouse_pos.0 + 1, mouse_pos.1);
            let left_x = mouse_pos.0 + 3;
            let mut y = mouse_pos.1;
            for s in tooltip.iter() {
                ctx.print_color(left_x + 1, y, white(), grey(), &s.to_string());
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(arrow_pos.x + 1 + i, y, white(), grey(), &" ".to_string());
                }
                y += 1;
            }
            ctx.print_color(arrow_pos.x, arrow_pos.y, white(), grey(), &"<-".to_string());
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected,
}

pub fn show_inventory(gs: &mut State, ctx: &mut Rltk) -> ItemMenuResult {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();

    // all the items in the player_entity's inventory
    let inventory = (&backpack, &names)
        .join()
        .filter(|item| item.0.owner == *player_entity);
    let count = inventory.count() as i32;

    let mut y = 25 - (count / 2);
    ctx.draw_box(15, y - 2, 31, count + 3, white(), black());
    let title = "Inventory";
    ctx.print_color(18, y - 2, yellow(), black(), title);
    let close_msg = "Esc to close";
    ctx.print_color(18, y + count + 1, yellow(), black(), close_msg);

    let mut j = 0;
    for (_backpack, name) in (&backpack, &names)
        .join()
        .filter(|item| item.0.owner == *player_entity)
    {
        ctx.set(17, y, white(), black(), rltk::to_cp437('('));
        ctx.set(18, y, yellow(), black(), 97 + j as u8);
        ctx.set(19, y, white(), black(), rltk::to_cp437(')'));

        ctx.print(21, y, &name.name.to_string());
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => ItemMenuResult::NoResponse,
        Some(key) => match key {
            VirtualKeyCode::Escape => ItemMenuResult::Cancel,
            _ => ItemMenuResult::NoResponse,
        },
    }
}
