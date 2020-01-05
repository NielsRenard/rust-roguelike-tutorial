rltk::add_wasm_support!();
use rltk::{Console, GameState, RandomNumberGenerator, Rltk, RGB};
use specs::prelude::*;
#[macro_use]
extern crate specs_derive;
mod map;
pub use map::*;
mod components;
pub use components::*;
mod player;
use player::*;
mod rect;
pub use rect::Rect;
mod visibility_system;
use visibility_system::VisibilitySystem;
mod monster_ai_system;
use monster_ai_system::MonsterAI;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState { Waiting, Running }

pub struct State {
    pub ecs: World,
    pub runstate: RunState
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();



	if self.runstate == RunState::Running {
	    self.run_systems();
	    self.runstate = RunState::Waiting;
	} else {
	    self.runstate = player_input(self, ctx);
	}
	
        //        let map = self.ecs.fetch::<Map>();
        draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();
        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }
    }
}

impl State {
    fn run_systems(&mut self) {
        //        let mut lw = LeftWalker {};
        //        lw.run_now(&self.ecs);
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
	let mut mob = MonsterAI{};
	mob.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

fn main() {
    let context = Rltk::init_simple8x8(80, 50, "Hello Rust World", "resources");
    let mut gs = State { ecs: World::new(), runstate: RunState::Waiting };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<LeftMover>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    // add a map to the world
    let map: Map = Map::new_map_rooms_and_corridors();
    // make sure the player doesn't get put inside wall
    let (player_x, player_y) = map.rooms[0].center();

    // every room -except the first one- gets a monster
    let mut rng = RandomNumberGenerator::new();
    for room in map.rooms.iter().skip(1) {
        let (x, y) = room.center();
        let glyph: u8;
        let roll = rng.roll_dice(1, 2);
        match roll {
            1 => glyph = rltk::to_cp437('g'),
            _ => glyph = rltk::to_cp437('o'),
        }
        gs.ecs
            .create_entity()
            .with(Position { x, y })
            .with(Renderable {
                glyph: glyph,
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .with(Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
                dirty: true,
            })
            .with(Monster {})
            .build();
    }

    gs.ecs.insert(map);

    // make our 'guy'
    gs.ecs
        .create_entity()
        .with(Position {
            x: player_x,
            y: player_y,
        })
        .with(Renderable {
            glyph: rltk::to_cp437('☺'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .build();

    // make ten enemies
    //    for i in 0..10 {
    //        gs.ecs
    //            .create_entity()
    //            .with(Position { x: i * 7, y: 20 })
    //            .with(Renderable {
    //                glyph: rltk::to_cp437('☻'),
    //                fg: RGB::named(rltk::RED),
    //                bg: RGB::named(rltk::BLACK),
    //            })
    //            .with(LeftMover {})
    //            .build();
    //    }

    rltk::main_loop(context, gs);
}
