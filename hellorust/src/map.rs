extern crate rltk;
use super::{Rect, Viewshed, Player};
use rltk::{Console, RandomNumberGenerator, Rltk, RGB, Algorithm2D, BaseMap, Point};
use std::cmp::{max, min};
extern crate specs;
use specs::prelude::*;

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
}

impl Map {
    /// returns which array index is at a given x/y position
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        // multiplies the y position by the map width, and adds x.
        // This guarantees one tile per location
        // and efficiently maps it in memory for left-to-right reading.
        (y as usize * self.width as usize) + x as usize
        // TODO: when player moves 'left' off of the map
        // thread 'main' panicked at 'attempt to add with overflow', src/map.rs:16:12
    }

    fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2)..=max(x1, x2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < 80 * 50 {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < 80 * 50 {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    fn apply_room_to_map(&mut self, room: &Rect) {
        for y in room.y1 + 1..=room.y2 {
            for x in room.x1 + 1..=room.x2 {
                let idx = self.xy_idx(x, y);
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    pub fn new_map_rooms_and_corridors() -> Map {
        let mut map = Map {
            tiles: vec![TileType::Wall; 80 * 50],
            rooms: Vec::new(),
            width: 80,
            height: 50,
        };

        //    let mut rooms: Vec<Rect> = Vec::new();
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for _i in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, 80 - w - 1) - 1;
            let y = rng.roll_dice(1, 50 - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in map.rooms.iter() {
                if new_room.intersect(other_room) {
                    ok = false;
                }
            }
            if ok {
                map.apply_room_to_map(&new_room);

                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len() - 1].center();
                    // 50%
                    if rng.range(0, 2) == 1 {
                        map.apply_horizontal_tunnel(prev_x, new_x, prev_y);
                        map.apply_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        map.apply_vertical_tunnel(prev_y, new_y, prev_x);
                        map.apply_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }
                map.rooms.push(new_room);
            }
        }
        map
    }
}

impl Algorithm2D for Map {
    fn in_bounds(&self, pos : Point) -> bool {
        pos.x > 0 && pos.x < self.width-1 && pos.y > 0 && pos.y < self.height-1
    }

    fn point2d_to_index(&self, pt: Point) -> i32 {
	(pt.y * self.width) + pt.x
    }

    fn index_to_point2d(&self, idx:i32) -> Point {
        Point{ x: idx % self.width, y: idx / self.width }
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx:i32) -> bool {
        self.tiles[idx as usize] == TileType::Wall
    }

    fn get_available_exits(&self, _idx:i32) -> Vec<(i32, f32)> {
        Vec::new()
    }

    fn get_pathing_distance(&self, idx1: i32, idx2: i32) -> f32{
	let p1 = Point::new(idx1 % self.width, idx1 / self.width);
        let p2 = Point::new(idx2 % self.width, idx2 / self.width);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}

pub fn draw_map(ecs: &World, ctx: &mut Rltk) {
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<Map>();


    for (_player, viewshed) in (&mut players, &mut viewsheds).join() {
	let mut y = 0;
	let mut x = 0;
	for tile in map.tiles.iter() {
            // Render a tile depending upon the tile type
            let pt = Point::new(x,y);
	    if viewshed.visible_tiles.contains(&pt) {
		match tile {
		    TileType::Floor => {
			ctx.set(
			    x,
			    y,
			    RGB::from_f32(0.5, 0.5, 0.5),
			    RGB::from_f32(0., 0., 0.),
			    rltk::to_cp437('.'),
			);
		    }
		    TileType::Wall => {
			ctx.set(
			    x,
			    y,
			    RGB::from_f32(0.0, 1.0, 0.0),
			    RGB::from_f32(0., 0., 0.),
			    rltk::to_cp437('#'),
			);
		    }
		}
	    }
	    
            // Move the coordinates
            x += 1;
            if x > 79 {
		x = 0;
		y += 1;
            }
	}
    }
}
