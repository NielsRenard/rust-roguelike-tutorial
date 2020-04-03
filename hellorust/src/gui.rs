extern crate rltk;
use crate::components::{HungerClock, HungerState::*};
use rltk::{Console, Rltk, VirtualKeyCode};
extern crate specs;
use super::rex_assets::RexAssets;
use super::{
    CombatStats, Equipped, InBackpack, Map, Name, Player, Point, Position, RunState, State,
    Viewshed,
};
use crate::color::*;
use crate::gamelog::GameLog;
use crate::saveload_system::save_exists;
use specs::prelude::*;

// TODO: replace 43 and 79 with const's (maybe directly couple to the ones in map.rs)

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuSelection {
    NewGame,
    LoadGame,
    Quit,
}

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuResult {
    NoSelection { selected: MainMenuSelection },
    Selected { selected: MainMenuSelection },
}

#[derive(PartialEq, Copy, Clone)]
pub enum GameOverResult {
    NoSelection,
    QuitToMenu,
}

pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    ctx.draw_box(0, 43, 79, 6, white(), black());

    let map = ecs.fetch::<Map>();
    let depth = format!("Depth: {}", map.depth);
    ctx.print_color(2, 43, yellow(), black(), &depth);

    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();
    let hunger_clocks = ecs.read_storage::<HungerClock>();
    let log = ecs.fetch::<GameLog>();

    //GameLog message printing
    let mut y = 44;
    for message in log.entries.iter().rev() {
        if y < 49 {
            ctx.print(2, y, &message.to_string());
        }
        y += 1;
    }

    for (_player, stats, hunger) in (&players, &combat_stats, &hunger_clocks).join() {
        let health = format!(" HP: {}/{} ", stats.hp, stats.max_hp);
        ctx.print_color(12, 43, white(), black(), &health);
        ctx.draw_bar_horizontal(28, 43, 51, stats.hp, stats.max_hp, red(), black());

        let (hunger_text, text_color) = match hunger.state {
            WellFed => ("Well fed", green()),
            Normal => ("Normal", yellow()),
            Hungry => ("Hungry", orange()),
            Starving => ("Starving", red()),
        };
        ctx.print_color(71, 42, text_color, black(), hunger_text);
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

pub fn show_inventory(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    // all the items in the player_entity's inventory
    let inventory = (&backpack, &names)
        .join()
        .filter(|item| item.0.owner == *player_entity);
    let count = inventory.count() as i32;

    let mut equippable: Vec<Entity> = Vec::new();
    let mut y = 25 - (count / 2);
    ctx.draw_box(15, y - 2, 31, count + 3, white(), black());
    let title = "Inventory";
    ctx.print_color(18, y - 2, yellow(), black(), title);
    let close_msg = "Esc to close";
    ctx.print_color(18, y + count + 1, yellow(), black(), close_msg);

    let mut j = 0;
    for (entity, _backpack, name) in (&entities, &backpack, &names)
        .join()
        .filter(|item| item.1.owner == *player_entity)
    {
        ctx.set(17, y, white(), black(), rltk::to_cp437('('));
        ctx.set(18, y, yellow(), black(), 97 + j as u8); //ASCII code 97 = a
        ctx.set(19, y, white(), black(), rltk::to_cp437(')'));

        ctx.print(21, y, &name.name.to_string());
        equippable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count {
                    return (
                        ItemMenuResult::Selected,
                        Some(equippable[selection as usize]),
                    );
                }
                (ItemMenuResult::NoResponse, None)
            }
        },
    }
}

pub fn show_drop_item_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    // all the items in the player_entity's inventory
    let inventory = (&backpack, &names)
        .join()
        .filter(|item| item.0.owner == *player_entity);
    let count = inventory.count() as i32;

    let mut equippable: Vec<Entity> = Vec::new();
    let mut y = 25 - (count / 2);
    ctx.draw_box(15, y - 2, 31, count + 3, white(), black());
    let title = "Drop which item?";
    ctx.print_color(18, y - 2, red(), black(), title);
    let close_msg = "Esc to close";
    ctx.print_color(18, y + count + 1, yellow(), black(), close_msg);

    let mut j = 0;
    for (entity, _backpack, name) in (&entities, &backpack, &names)
        .join()
        .filter(|item| item.1.owner == *player_entity)
    {
        ctx.set(17, y, white(), black(), rltk::to_cp437('('));
        ctx.set(18, y, yellow(), black(), 97 + j as u8); //ASCII code 97 = a
        ctx.set(19, y, white(), black(), rltk::to_cp437(')'));

        ctx.print(21, y, &name.name.to_string());
        equippable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count {
                    return (
                        ItemMenuResult::Selected,
                        Some(equippable[selection as usize]),
                    );
                }
                (ItemMenuResult::NoResponse, None)
            }
        },
    }
}

pub fn ranged_target(
    gs: &mut State,
    ctx: &mut Rltk,
    range: i32,
) -> (ItemMenuResult, Option<Point>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let player_pos = gs.ecs.fetch::<Point>();
    let viewsheds = gs.ecs.read_storage::<Viewshed>();

    ctx.print_color(5, 0, yellow(), black(), "Select Target:");

    // "Highlight available target cells"
    let mut available_cells = Vec::new();
    let visible_to_player = viewsheds.get(*player_entity);

    match visible_to_player {
        Some(visible) => {
            // "We have a viewshed"
            for idx in visible.visible_tiles.iter() {
                let distance = rltk::DistanceAlg::Pythagoras.distance2d(*player_pos, *idx);
                if distance <= range as f32 {
                    ctx.set_bg(idx.x, idx.y, blue());
                    available_cells.push(idx);
                }
            }
        }
        None => return (ItemMenuResult::Cancel, None),
    }

    // Draw mouse cursor
    let mouse_pos = ctx.mouse_pos();
    let mut valid_target = false;
    for idx in available_cells.iter() {
        if idx.x == mouse_pos.0 && idx.y == mouse_pos.1 {
            valid_target = true;
        }
    }

    if valid_target {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, cyan());
        if ctx.left_click {
            return (
                ItemMenuResult::Selected,
                Some(Point::new(mouse_pos.0, mouse_pos.1)),
            );
        }
    } else {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, red());
        if ctx.left_click {
            return (ItemMenuResult::Cancel, None);
        }
    }

    (ItemMenuResult::NoResponse, None)
}

pub fn main_menu(gs: &mut State, ctx: &mut Rltk) -> MainMenuResult {
    let runstate = gs.ecs.fetch::<RunState>();
    ctx.print_color_centered(15, yellow(), black(), "Hello Rust World");
    let assets = gs.ecs.fetch::<RexAssets>();
    ctx.render_xp_sprite(&assets.menu, 0, 0);

    if let RunState::MainMenu {
        menu_selection: selection,
    } = *runstate
    {
        if selection == MainMenuSelection::NewGame {
            ctx.print_color_centered(20, magenta(), black(), "New Game");
        } else {
            ctx.print_color_centered(20, white(), black(), "New Game");
        }

        if selection == MainMenuSelection::LoadGame {
            ctx.print_color_centered(22, magenta(), black(), "Continue Game");
        } else {
            if save_exists() {
                ctx.print_color_centered(22, white(), black(), "Continue Game");
            } else {
                ctx.print_color_centered(22, grey(), black(), "Continue Game");
            }
        }

        if selection == MainMenuSelection::Quit {
            ctx.print_color_centered(24, magenta(), black(), "Quit");
        } else {
            ctx.print_color_centered(24, white(), black(), "Quit");
        }

        match ctx.key {
            None => {
                return MainMenuResult::NoSelection {
                    selected: selection,
                }
            }
            Some(key) => match key {
                VirtualKeyCode::Escape => {
                    return MainMenuResult::NoSelection {
                        selected: MainMenuSelection::Quit,
                    }
                }
                VirtualKeyCode::Up | VirtualKeyCode::W => {
                    let newselection;
                    match selection {
                        MainMenuSelection::NewGame => newselection = MainMenuSelection::Quit,
                        MainMenuSelection::LoadGame => newselection = MainMenuSelection::NewGame,
                        MainMenuSelection::Quit => {
                            newselection = if save_exists() {
                                MainMenuSelection::LoadGame
                            } else {
                                MainMenuSelection::NewGame
                            }
                        }
                    }
                    return MainMenuResult::NoSelection {
                        selected: newselection,
                    };
                }
                VirtualKeyCode::Down | VirtualKeyCode::S => {
                    let newselection;
                    match selection {
                        MainMenuSelection::NewGame => {
                            newselection = if save_exists() {
                                MainMenuSelection::LoadGame
                            } else {
                                MainMenuSelection::Quit
                            }
                        }
                        MainMenuSelection::LoadGame => newselection = MainMenuSelection::Quit,
                        MainMenuSelection::Quit => newselection = MainMenuSelection::NewGame,
                    }
                    return MainMenuResult::NoSelection {
                        selected: newselection,
                    };
                }
                VirtualKeyCode::Return | VirtualKeyCode::Space => {
                    return MainMenuResult::Selected {
                        selected: selection,
                    }
                }
                _ => {
                    return MainMenuResult::NoSelection {
                        selected: selection,
                    }
                }
            },
        }
    }

    MainMenuResult::NoSelection {
        selected: MainMenuSelection::NewGame,
    }
}

pub fn remove_equipment_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<Equipped>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names)
        .join()
        .filter(|item| item.0.owner == *player_entity);
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y - 2, 31, (count + 3) as i32, white(), black());
    ctx.print_color(18, y - 2, yellow(), black(), "Remove Which Item?");
    ctx.print_color(
        18,
        y + count as i32 + 1,
        yellow(),
        black(),
        "Escape to cancel",
    );

    let mut equippable: Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, _pack, name) in (&entities, &backpack, &names)
        .join()
        .filter(|item| item.1.owner == *player_entity)
    {
        ctx.set(17, y, white(), black(), rltk::to_cp437('('));
        ctx.set(18, y, yellow(), black(), 97 + j as u8);
        ctx.set(19, y, white(), black(), rltk::to_cp437(')'));

        ctx.print(21, y, &name.name.to_string());
        equippable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return (
                        ItemMenuResult::Selected,
                        Some(equippable[selection as usize]),
                    );
                }
                (ItemMenuResult::NoResponse, None)
            }
        },
    }
}

pub fn game_over(ctx: &mut Rltk) -> GameOverResult {
    ctx.print_color_centered(15, yellow(), black(), "GAME OVER");
    ctx.print_color_centered(
        40,
        white(),
        black(),
        "Let us go out this evening for pleasure.",
    );
    ctx.print_color_centered(41, white(), black(), "The night is still young.");

    ctx.print_color_centered(
        43,
        magenta(),
        black(),
        "Press any key to return to the Main Menu.",
    );
    match ctx.key {
        None => GameOverResult::NoSelection,
        Some(_) => GameOverResult::QuitToMenu,
    }
}
