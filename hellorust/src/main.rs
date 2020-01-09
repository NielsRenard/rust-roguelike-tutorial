rltk::add_wasm_support!();
use rltk::{Console, GameState, Point, RandomNumberGenerator, Rltk};
use specs::prelude::*;
#[macro_use]
extern crate specs_derive;
mod map;
pub use map::*;
mod color;
use color::{black, red};
mod components;
mod gamelog;
mod gui;
mod spawner;
pub use components::*;
mod player;
use player::*;
mod rect;
pub use rect::Rect;
mod visibility_system;
use visibility_system::VisibilitySystem;
mod monster_ai_system;
use monster_ai_system::MonsterAI;
mod map_indexing_system;
use map_indexing_system::MapIndexingSystem;
mod melee_combat_system;
use melee_combat_system::MeleeCombatSystem;
mod damage_system;
use damage_system::DamageSystem;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
}

pub struct State {
    pub ecs: World,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        let mut new_runstate;

        {
            let runstate = self.ecs.fetch::<RunState>();
            new_runstate = *runstate;
        }

        match new_runstate {
            RunState::PreRun => {
                self.run_systems();
                new_runstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                new_runstate = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                new_runstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                new_runstate = RunState::AwaitingInput;
            }
        }

        // "if you declare and use a variable inside a scope, it is dropped on scope exit
        // (you can also manually drop things)"
        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = new_runstate;
        }

        damage_system::delete_the_dead(&mut self.ecs);
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
        gui::draw_ui(&self.ecs, ctx);
    }
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);
        let mut mapindex = MapIndexingSystem {};
        mapindex.run_now(&self.ecs);
        let mut melee = MeleeCombatSystem {};
        melee.run_now(&self.ecs);
        let mut damage = DamageSystem {};
        damage.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

fn main() {
    let context = Rltk::init_simple8x8(80, 50, "Hello Rust World", "resources");
    //    context.with_post_scanlines(true); // ↑ requires changing context to mut
    let mut gs = State { ecs: World::new() };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<SufferDamage>();

    // add a map to the world
    let map: Map = Map::new_map_rooms_and_corridors();
    // make sure the player doesn't get put inside wall
    let (player_x, player_y) = map.rooms[0].center();

    // make our 'guy'
    let player_entity = spawner::player(&mut gs.ecs, player_x, player_y);

    // every room -except the first one- gets a monster
    let mut rng = RandomNumberGenerator::new();
    for (i, room) in map.rooms.iter().skip(1).enumerate() {
        let (x, y) = room.center();

        let glyph: u8;
        let roll = rng.roll_dice(1, 2);
        let name: String;
        match roll {
            1 => {
                glyph = rltk::to_cp437('g');
                name = "Goblin".to_string();
            }
            _ => {
                glyph = rltk::to_cp437('o');
                name = "Orc".to_string();
            }
        }

        gs.ecs
            .create_entity()
            .with(Position { x, y })
            .with(Renderable {
                glyph: glyph,
                fg: red(),
                bg: black(),
            })
            .with(Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
                dirty: true,
            })
            .with(Monster {})
            .with(Name {
                name: format!("{} #{}", &name, i),
            })
            .with(BlocksTile {})
            .with(CombatStats {
                max_hp: 16,
                hp: 16,
                defense: 1,
                strength: 4,
            })
            .build();
    }

    gs.ecs.insert(RandomNumberGenerator::new());
    gs.ecs.insert(map);
    gs.ecs.insert(Point::new(player_x, player_y));
    println!("player initial position (x:{},y:{})", player_x, player_y);
    gs.ecs.insert(player_entity);
    gs.ecs.insert(gamelog::GameLog {
        entries: vec!["Good luck...".to_string()],
    });
    gs.ecs.insert(RunState::PreRun);

    rltk::main_loop(context, gs);
}
