rltk::add_wasm_support!();
use rltk::{Console, GameState, Rltk, RGB};
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

pub struct State {
    ecs: World,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        player_input(self, ctx);
        self.run_systems();
        //        let map = self.ecs.fetch::<Map>();
        draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}

impl State {
    fn run_systems(&mut self) {
        //        let mut lw = LeftWalker {};
        //        lw.run_now(&self.ecs);
        //        let mut vis = VisibilitySystem{};
        //        vis.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

fn main() {
    let context = Rltk::init_simple8x8(80, 50, "Hello Rust World", "resources");
    let mut gs = State { ecs: World::new() };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<LeftMover>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();

    // add a map to the world
    let map: Map = Map::new_map_rooms_and_corridors();
    // make sure the player doesn't get put inside wall
    let (player_x, player_y) = map.rooms[0].center();

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
