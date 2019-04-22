use crate::geography::{Geography, TilePoint};

use quicksilver::geom::Vector;
use rand::prelude::*;
use rand::distributions::{Normal, Distribution};

enum Activity {
    Idle,
    FindingFood,
    Eating,
    FindingBed,
    Sleeping,
} 

pub struct Human {
    current_path: Vec<TilePoint>,
    location: Vector,
    home: Vector,
    food_location: Vector,
    speed: f32,
    pub fatigue: f32,
    pub hunger: f32,
    state: Activity,
}

impl Human {
    pub fn new(location: Vector) -> Human {
        Human {
            current_path: Vec::new(),
            location: location,
            home: Vector::new(63.5, 22.5),
            food_location: Vector::new(39.5, 29.5),
            speed: 0.1,
            fatigue: 75.0,
            hunger: 80.0,
            state: Activity::Idle,
        }
    }

    pub fn location(&self) -> Vector {
        self.location
    }

    fn set_goal(&mut self, goal: Vector, geography: &Geography) {
        let goal = TilePoint::from_vector(&goal);
        let start = TilePoint::from_vector(&self.location);
        self.current_path = geography.find_path(start, goal).unwrap_or(Vec::new());
    }

    pub fn think(&mut self, geography: &Geography) {
        let mut rng = thread_rng();
        match self.state {
            Activity::Idle => if self.hunger > 80.0 {
                self.state = Activity::FindingFood;
                self.set_goal(self.food_location, geography);
            } else if self.fatigue > 80.0 {
                self.state = Activity::FindingBed;
                self.set_goal(self.home, geography);
            } else if self.current_path.is_empty() && rng.gen::<f32>() > 0.99 {
                let normal = Normal::new(0.0, 5.0);
                self.set_goal(self.location + Vector::new(normal.sample(&mut rng) as f32, normal.sample(&mut rng) as f32), geography);
            },
            Activity::FindingFood => if self.current_path.is_empty() {
                self.state = Activity::Eating;
            },
            Activity::FindingBed => if self.current_path.is_empty() {
                self.state = Activity::Sleeping;
            },
            Activity::Eating => if self.hunger <= 0.0 {
                self.state = Activity::Idle;
            },
            Activity::Sleeping => if self.fatigue <= 0.0 {
                self.state = Activity::Idle;
            },
        }
    }

    pub fn act(&mut self) {
        match self.state {
            Activity::Eating => self.hunger -= 1.0,
            Activity::Sleeping => {
                self.fatigue -= 0.05;
                self.hunger -= 0.025; // hunger slower while sleeping
                    
            },
            _ => (),
        }
        self.travel();
        self.fatigue += 0.01;
        self.hunger += 0.05;
    }

    fn travel(&mut self) {
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
