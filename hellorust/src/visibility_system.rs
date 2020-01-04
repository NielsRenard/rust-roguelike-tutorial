extern crate specs;
use super::{Position, Viewshed, Map};
use specs::prelude::*;
extern crate rltk;
use rltk::{field_of_view, Point};

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (ReadExpect<'a, Map>, WriteStorage<'a, Viewshed>, WriteStorage<'a, Position>);

    fn run(&mut self, data: Self::SystemData) {
	let (map, mut viewshed, pos) = data;
	
        for (viewshed, pos) in (&mut viewshed, &pos).join() {
	    viewshed.visible_tiles.clear();
	    // &* is syntax to dereference, then get a reference →→→→→→→→→→→→→→→→→→→→→→→→→→→→↓
	    viewshed.visible_tiles = field_of_view(Point::new(pos.x, pos.y), viewshed.range, &*map);
            viewshed.visible_tiles.retain(|p : &Point| p.x > 0 && p.x < map.width-1 && p.y > 0 && p.y < map.height-1 );
	    // This will now run every frame (which is overkill, more on that later)
	    // and store a list of visible tiles.	    
	}
    }
}
