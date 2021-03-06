rltk::add_wasm_support!();
use rltk::{GameState, Point, RandomNumberGenerator, Rltk, RltkBuilder};
use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};
#[macro_use]
extern crate specs_derive;
extern crate serde;
mod map;
pub use map::*;
mod color;
mod components;
mod gamelog;
mod gui;
mod inventory_system;
mod spawner;
pub use components::*;
use inventory_system::{
    EquipmentRemoveSystem, ItemCollectionSystem, ItemDropSystem, ItemUseSystem,
};
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
mod random_table;
mod saveload_system;
use random_table::RandomTable;
mod particle_system;
use particle_system::{ParticleBuilder, ParticleSpawnSystem};
mod hunger_system;
use hunger_system::HungerSystem;
mod trigger_system;
use trigger_system::TriggerSystem;
pub mod map_builders;
mod rex_assets;

const SHOW_MAPGEN_VISUALIZER: bool = true;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem,
    ShowTargeting {
        range: i32,
        item: Entity,
    },
    MainMenu {
        menu_selection: gui::MainMenuSelection,
    },
    SaveGame,
    NextLevel,
    MagicMapReveal {
        row: i32,
    },
    MapGeneration,
    ShowRemoveItem,
    GameOver,
}

pub struct State {
    pub ecs: World,
    mapgen_next_state: Option<RunState>,
    mapgen_history: Vec<Map>,
    mapgen_index: usize,
    mapgen_timer: f32,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        let mut new_runstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            new_runstate = *runstate;
        }
        ctx.cls();
        particle_system::cull_dead_particles(&mut self.ecs, ctx);

        match new_runstate {
            RunState::MainMenu { .. } => {}
            RunState::GameOver { .. } => {}
            _ => {
                draw_map(&self.ecs.fetch::<Map>(), ctx);
                {
                    let positions = self.ecs.read_storage::<Position>();
                    let renderables = self.ecs.read_storage::<Renderable>();
                    let hidden = self.ecs.read_storage::<Hidden>();
                    let map = self.ecs.fetch::<Map>();

                    // https://specs.amethyst.rs/docs/tutorials/11_advanced_component.html#sorting-entities-based-on-component-value
                    // sort the components by render order
                    // player and monster visible when standing on potion
                    let mut data = (&positions, &renderables, !&hidden)
                        .join()
                        .collect::<Vec<_>>();
                    data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));
                    for (pos, render, _hidden) in data.iter() {
                        let idx = map.xy_idx(pos.x, pos.y);
                        if map.visible_tiles[idx] {
                            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
                        }
                    }

                    gui::draw_ui(&self.ecs, ctx);
                }
            }
        }

        match new_runstate {
            RunState::MapGeneration => {
                if !SHOW_MAPGEN_VISUALIZER {
                    new_runstate = self.mapgen_next_state.unwrap();
                }
                ctx.cls();
                draw_map(&self.mapgen_history[self.mapgen_index], ctx);

                // "Add the frame duration to the mapgen_timer..."
                self.mapgen_timer += ctx.frame_time_ms;
                if self.mapgen_timer > 300.0 {
                    self.mapgen_timer = 0.0;
                    self.mapgen_index += 1;
                    // if the frame counter has reached the end of our history...
                    if self.mapgen_index >= self.mapgen_history.len() {
                        // ...transition to the next game state.
                        new_runstate = self.mapgen_next_state.unwrap();
                    }
                }
            }
            RunState::PreRun => {
                self.run_systems();
                self.ecs.maintain();
                new_runstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                new_runstate = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                self.ecs.maintain();
                match *self.ecs.fetch::<RunState>() {
                    RunState::MagicMapReveal { .. } => {
                        new_runstate = RunState::MagicMapReveal { row: 0 }
                    }
                    _ => new_runstate = RunState::MonsterTurn,
                }
            }
            RunState::MonsterTurn => {
                self.run_systems();
                self.ecs.maintain();
                new_runstate = RunState::AwaitingInput;
            }
            RunState::ShowInventory => {
                let result = gui::show_inventory(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let is_ranged = self.ecs.read_storage::<Ranged>();
                        let is_item_ranged = is_ranged.get(item_entity);

                        // "handle items that are ranged and induce a mode switch"
                        match is_item_ranged {
                            Some(ranged_item) => {
                                new_runstate = RunState::ShowTargeting {
                                    range: ranged_item.range,
                                    item: item_entity,
                                }
                            }
                            None => {
                                let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                                intent
                                    .insert(
                                        *self.ecs.fetch::<Entity>(),
                                        WantsToUseItem {
                                            item: item_entity,
                                            target: None,
                                        },
                                    )
                                    .expect("Unable to insert use item intent");
                                new_runstate = RunState::PlayerTurn;
                            }
                        }
                    }
                }
            }
            RunState::ShowDropItem => {
                let result = gui::show_drop_item_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToDropItem { item: item_entity },
                            )
                            .expect("Unable to insert drop item intent");
                        new_runstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::ShowTargeting { range, item } => {
                let result = gui::ranged_target(self, ctx, range);
                match result.0 {
                    gui::ItemMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToUseItem {
                                    item: item,
                                    target: result.1,
                                },
                            )
                            .expect("Unable to insert intent");
                        new_runstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::MainMenu { .. } => {
                let result = gui::main_menu(self, ctx);
                match result {
                    gui::MainMenuResult::NoSelection { selected } => {
                        new_runstate = RunState::MainMenu {
                            menu_selection: selected,
                        }
                    }
                    gui::MainMenuResult::Selected { selected } => match selected {
                        gui::MainMenuSelection::NewGame => {
                            saveload_system::delete_save();
                            new_runstate = RunState::PreRun
                        }
                        gui::MainMenuSelection::LoadGame => {
                            saveload_system::load_game(&mut self.ecs);
                            new_runstate = RunState::PreRun;
                            saveload_system::delete_save();
                        }
                        gui::MainMenuSelection::Quit => {
                            std::process::exit(0);
                        }
                    },
                }
            }
            RunState::SaveGame => {
                saveload_system::save_game(&mut self.ecs);
                new_runstate = RunState::MainMenu {
                    menu_selection: gui::MainMenuSelection::LoadGame,
                };
            }
            RunState::NextLevel => {
                self.goto_next_level();
                new_runstate = RunState::PreRun;
            }
            RunState::ShowRemoveItem => {
                let result = gui::remove_equipment_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToRemoveEquipment>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToRemoveEquipment { item: item_entity },
                            )
                            .expect("Unable to insert intent");
                        new_runstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::MagicMapReveal { row } => {
                let mut map = self.ecs.fetch_mut::<Map>();
                for x in 0..MAP_WIDTH {
                    let idx = map.xy_idx(x as i32, row);
                    map.revealed_tiles[idx] = true;
                }
                if row as usize == MAP_HEIGHT - 1 {
                    new_runstate = RunState::MonsterTurn;
                } else {
                    new_runstate = RunState::MagicMapReveal { row: row + 1 };
                }
            }
            RunState::GameOver => {
                let result = gui::game_over(ctx);
                match result {
                    gui::GameOverResult::QuitToMenu => {
                        self.game_over_cleanup();
                        new_runstate = RunState::MainMenu {
                            menu_selection: gui::MainMenuSelection::NewGame,
                        };
                    }
                    gui::GameOverResult::NoSelection => {}
                }
            }
        }

        // "if you declare and use a variable inside a scope, it is dropped on scope exit
        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = new_runstate;
        }

        damage_system::delete_the_dead(&mut self.ecs);
    }
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);
        let mut trigger_system = TriggerSystem {};
        trigger_system.run_now(&self.ecs);
        let mut mapindex = MapIndexingSystem {};
        mapindex.run_now(&self.ecs);
        let mut melee = MeleeCombatSystem {};
        melee.run_now(&self.ecs);
        let mut damage = DamageSystem {};
        damage.run_now(&self.ecs);
        let mut pickup = ItemCollectionSystem {};
        pickup.run_now(&self.ecs);
        let mut drop_items = ItemDropSystem {};
        drop_items.run_now(&self.ecs);
        let mut use_items = ItemUseSystem {};
        use_items.run_now(&self.ecs);
        let mut remove_equipment = EquipmentRemoveSystem {};
        remove_equipment.run_now(&self.ecs);
        let mut particle_system = ParticleSpawnSystem {};
        particle_system.run_now(&self.ecs);
        let mut hunger_system = HungerSystem {};
        hunger_system.run_now(&self.ecs);
        // "We've made the particle system depend upon likely particle
        // spawners. We'll have to be a little careful to avoid
        // accidentally making it concurrent with anything that might
        // add to it."
        self.ecs.maintain();
    }

    fn generate_world_map(&mut self, new_depth: i32) {
        self.mapgen_index = 0;
        self.mapgen_timer = 0.0;
        self.mapgen_history.clear();

        let mut builder = map_builders::random_builder(new_depth);
        builder.build_map();
        self.mapgen_history = builder.get_snapshot_history();

        let player_start;

        {
            let mut worldmap_resource = self.ecs.write_resource::<Map>();
            *worldmap_resource = builder.get_map();
            player_start = builder.get_starting_position();
        }

        // spawn baddies
        builder.spawn_entities(&mut self.ecs);

        // position our 'guy' and update resources
        let (player_x, player_y) = (player_start.x, player_start.y);
        let mut player_position = self.ecs.write_resource::<Point>();
        *player_position = Point::new(player_x, player_y);
        let mut position_components = self.ecs.write_storage::<Position>();
        let player_entity = self.ecs.fetch::<Entity>();
        let player_pos_comp = position_components.get_mut(*player_entity);
        if let Some(player_pos_comp) = player_pos_comp {
            player_pos_comp.x = player_x;
            player_pos_comp.y = player_y;
        }

        // Mark the player's visibility as dirty, systems will take care of it
        let mut viewshed_components = self.ecs.write_storage::<Viewshed>();
        let vs = viewshed_components.get_mut(*player_entity);
        if let Some(vs) = vs {
            vs.dirty = true;
        }
    }

    fn entities_to_remove_on_level_change(&mut self) -> Vec<Entity> {
        let entities = self.ecs.entities();
        let player = self.ecs.read_storage::<Player>();
        let backpack = self.ecs.read_storage::<InBackpack>();
        let equipped_items = self.ecs.read_storage::<Equipped>();
        let player_entity = self.ecs.fetch::<Entity>();

        let mut to_delete: Vec<Entity> = Vec::new();
        // TODO: do we need .join() here?
        for entity in entities.join() {
            let mut should_delete = true;

            // Don't delete the player
            let p = player.get(entity);
            if let Some(_p) = p {
                should_delete = false;
            }

            // Don't delete the player's backpack contents
            let bp = backpack.get(entity);
            if let Some(bp) = bp {
                if bp.owner == *player_entity {
                    should_delete = false;
                }
            }
            // Don't delete the player's equiped items
            let eq_item = equipped_items.get(entity);
            if let Some(item) = eq_item {
                if item.owner == *player_entity {
                    should_delete = false;
                }
            }

            if should_delete {
                to_delete.push(entity);
            }
        }
        to_delete
    }

    fn goto_next_level(&mut self) {
        // Delete entities that aren't the player or his/her equipment
        let to_delete = self.entities_to_remove_on_level_change();
        for target in to_delete {
            self.ecs
                .delete_entity(target)
                .expect("Unable to delete entity");
        }

        let current_depth;
        {
            let worldmap_resource = self.ecs.write_resource::<Map>();
            current_depth = worldmap_resource.depth;
        }
        self.generate_world_map(current_depth + 1);

        let player_entity = self.ecs.fetch::<Entity>();

        // Notify the player and give them some health
        let mut gamelog = self.ecs.fetch_mut::<gamelog::GameLog>();
        gamelog
            .entries
            .push("You descend to the next level, and take a moment to heal.".to_string());
        let mut player_health_store = self.ecs.write_storage::<CombatStats>();
        let player_health = player_health_store.get_mut(*player_entity);
        if let Some(player_health) = player_health {
            //if less than 50% health, set to 50%
            player_health.hp = i32::max(player_health.hp, player_health.max_hp / 2);
        }
    }

    fn game_over_cleanup(&mut self) {
        // Mark and delete all entities
        let mut to_delete = Vec::new();
        for e in self.ecs.entities().join() {
            to_delete.push(e);
        }
        for del in to_delete.iter() {
            self.ecs.delete_entity(*del).expect("Deletion failed");
        }

        {
            let player_entity = spawner::player(&mut self.ecs, 0, 0);
            let mut player_entity_writer = self.ecs.write_resource::<Entity>();
            *player_entity_writer = player_entity;
        }

        // Flush the gamelog
        self.ecs.insert(gamelog::GameLog { entries: vec![] });
        self.generate_world_map(1);
    }
}

fn main() -> rltk::BError {
    let context = RltkBuilder::simple80x50()
        .with_title("Hello Rust World")
        .with_fullscreen(true)
        .build()?;

    let main_menu_selection = if saveload_system::save_exists() {
        gui::MainMenuSelection::LoadGame
    } else {
        gui::MainMenuSelection::NewGame
    };

    let mut gs = State {
        ecs: World::new(),
        mapgen_next_state: Some(RunState::MainMenu {
            menu_selection: main_menu_selection,
        }),
        mapgen_history: Vec::new(),
        mapgen_index: 0,
        mapgen_timer: 0.0,
    };
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
    gs.ecs.register::<Item>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<WantsToUseItem>();
    gs.ecs.register::<WantsToDropItem>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<ProvidesHealing>();
    gs.ecs.register::<Ranged>();
    gs.ecs.register::<InflictsDamage>();
    gs.ecs.register::<AreaOfEffect>();
    gs.ecs.register::<Confusion>();
    gs.ecs.register::<SimpleMarker<SerializeMe>>();
    gs.ecs.register::<SerializationHelper>();
    gs.ecs.register::<Equippable>();
    gs.ecs.register::<Equipped>();
    gs.ecs.register::<MeleePowerBonus>();
    gs.ecs.register::<DefenseBonus>();
    gs.ecs.register::<WantsToRemoveEquipment>();
    gs.ecs.register::<Destructable>();
    gs.ecs.register::<ParticleLifetime>();
    gs.ecs.register::<HungerClock>();
    gs.ecs.register::<ProvidesFood>();
    gs.ecs.register::<MagicMapper>();
    gs.ecs.register::<Hidden>();
    gs.ecs.register::<EntryTrigger>();
    gs.ecs.register::<EntityMoved>();
    gs.ecs.register::<SingleActivation>();
    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());
    gs.ecs.insert(rex_assets::RexAssets::new());

    gs.ecs.insert(Map::new(1));
    gs.ecs.insert(Point::new(0, 0));
    gs.ecs.insert(RandomNumberGenerator::new());
    let player_entity = spawner::player(&mut gs.ecs, 0, 0);
    gs.ecs.insert(player_entity);

    gs.ecs.insert(particle_system::ParticleBuilder::new());
    gs.ecs.insert(gamelog::GameLog {
        entries: vec!["Good luck...".to_string()],
    });

    // initial RunState
    gs.ecs.insert(RunState::MapGeneration {});
    gs.generate_world_map(1);

    rltk::main_loop(context, gs)
}
