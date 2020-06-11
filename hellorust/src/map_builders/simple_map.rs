use super::{
    apply_horizontal_tunnel, apply_room_to_map, apply_vertical_tunnel, Map, MapBuilder, Position,
    Rect, TileType,
};
use rltk::RandomNumberGenerator;
pub struct SimpleMapBuilder {}

impl SimpleMapBuilder {
    fn rooms_and_corridors(map: &mut Map) -> Position {
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for _i in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            //(↓ e.g. max) x = [1..(80-10-1-1)] = [1..68]
            let x = rng.roll_dice(1, map.width as i32 - w - 1) - 1;
            let y = rng.roll_dice(1, map.height as i32 - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in map.rooms.iter() {
                if new_room.intersect(other_room) {
                    ok = false;
                }
            }
            if ok {
                apply_room_to_map(map, &new_room);

                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len() - 1].center();
                    // 50%
                    if rng.range(0, 2) == 1 {
                        apply_horizontal_tunnel(map, prev_x, new_x, prev_y);
                        apply_vertical_tunnel(map, prev_y, new_y, new_x);
                    } else {
                        apply_vertical_tunnel(map, prev_y, new_y, prev_x);
                        apply_horizontal_tunnel(map, prev_x, new_x, new_y);
                    }
                }
                map.rooms.push(new_room);
            }
        }
        let stairs_position = map.rooms[map.rooms.len() - 1].center();
        let stairs_idx = map.xy_idx(stairs_position.0, stairs_position.1);
        let start_position = map.rooms[0].center();
        map.tiles[stairs_idx] = TileType::DownStairs;
        Position {
            x: start_position.0,
            y: start_position.1,
        }
    }
}

impl MapBuilder for SimpleMapBuilder {
    fn build(new_depth: i32) -> (Map, Position) {
        let mut map = Map::new(new_depth);
        let pos = SimpleMapBuilder::rooms_and_corridors(&mut map);
        return (map, pos);
    }
}
