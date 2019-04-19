use crate::geography::{Geography, TilePoint};

use quicksilver::geom::Vector;

pub struct Human {
    pub current_path: Vec<TilePoint>,
    pub location: Vector,
    speed: f32,
}

impl Human {
    pub fn new(location: Vector) -> Human {
        Human {
            current_path: Vec::new(),
            location: location,
            speed: 0.1,
        }
    }

    pub fn set_goal(&mut self, goal: Vector, geography: &Geography) {
        let goal = TilePoint::from_vector(&goal);
        let start = TilePoint::from_vector(&self.location);
        self.current_path = geography.find_path(start, goal).unwrap_or(Vec::new());
    }

    pub fn update(&mut self) {
        if let Some(next_tile) = self.current_path.last() {
            let current_tile = TilePoint::from_vector(&self.location);
            if current_tile == *next_tile {
                self.current_path.pop();
            }
        }

        if let Some(next_tile) = self.current_path.last() {
            // Aim for middle of the tile
            let goal = Vector::new(next_tile.x as f32 + 0.5, next_tile.y as f32 + 0.5);
            let direct_path = goal - self.location;
            self.location += direct_path.with_len(self.speed);
        }
    }
}
