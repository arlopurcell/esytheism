use crate::geography::{Geography, TilePoint};
use crate::world::{World, Container, TICKS_PER_HOUR};
use crate::item::{Item, Inventory, ItemMessage};

use std::sync::mpsc::Sender;
use std::cmp::Ordering;

use quicksilver::geom::Vector;
use rand::prelude::*;
use rand::distributions::{Normal, Distribution};

const FATIGUE_PER_TICK: f32 = 1.0 / ( // below is ticks per unit, so invert for units per tick
    (TICKS_PER_HOUR as f32)
    * 18.0 // hours non-sleep / day
    * (1.0 / 100.0) // day / units sleep
);

const HUNGER_PER_TICK: f32 = 1.0 / ( // below is ticks per unit, so invert for units per tick
    (TICKS_PER_HOUR as f32)
    * 8.0 // hours between meals
    * (1.0 / 100.0) // day / units sleep
);

const SLEEP_PER_TICK: f32 = 1.0 / ( // below is ticks per unit, so invert for units per tick
    (TICKS_PER_HOUR as f32)
    * 6.0 // hours sleep / day
    * (1.0 / 100.0) // day / units sleep
) + FATIGUE_PER_TICK; // make up for fatigue added even while sleeping

enum Activity {
    Idle,
    Eating,
    Sleeping,
}

pub struct Human {
    pub location: Vector,
    pub inventory: Inventory,
    pub fatigue: f32,
    pub hunger: f32,
    speed: f32,
}

pub struct Mind {
    current_path: Vec<TilePoint>,
    state: Activity,

    home: Vector,
    adjacent_container: Option<Sender<ItemMessage>>, 
}

#[derive(PartialEq)]
struct MinFloat(f32);

impl Eq for MinFloat {}

impl PartialOrd for MinFloat {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.0.partial_cmp(&self.0)
    }
}

impl Ord for MinFloat {
    fn cmp(&self, other: &MinFloat) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Human {
    pub fn new(location: Vector) -> Human {
        Human {
            location: location,
            inventory: Inventory::new(),
            speed: 0.1,
            fatigue: 80.0,
            hunger: 0.0,
        }
    }
}


impl Mind {
    pub fn new() -> Mind {
        Mind {
            current_path: Vec::new(),
            state: Activity::Idle,

            home: Vector::new(63.5, 22.5),
            adjacent_container: None,
        }
    }

    fn set_goal(&mut self, human: &Human, goal: Vector, geography: &Geography) {
        let goal = TilePoint::from_vector(&goal);
        let start = TilePoint::from_vector(&human.location);
        self.current_path = geography.find_path(start, goal).unwrap_or(Vec::new());
    }

    pub fn think(&mut self, human: &Human, world: &World) {
        match self.state {
            Activity::Idle => if human.hunger > 80.0 {
                self.state = Activity::Eating;
            } else if human.fatigue > 80.0 {
                self.state = Activity::Sleeping;
            } else if self.current_path.is_empty() {
                let mut rng = thread_rng();
                if rng.gen::<f32>() > 0.99 {
                    let normal = Normal::new(0.0, 5.0);
                    self.set_goal(human, human.location + Vector::new(normal.sample(&mut rng) as f32, normal.sample(&mut rng) as f32), &world.geography);
                }
            },
            Activity::Eating => if human.hunger <= 0.0 {
                self.state = Activity::Idle;
            } else if self.current_path.is_empty() && human.inventory.count(Item::Food) == 0 && self.adjacent_container.is_none() {
                let mut food_containers: Vec<&Container> = world.containers.iter().filter(|container| container.inventory.count(Item::Food) > 0).collect();
                food_containers.sort_by_key(|container| MinFloat(human.location.distance(container.location)));
                if let Some(container) = food_containers.first() {
                    if TilePoint::from_vector(&container.location) == TilePoint::from_vector(&human.location) {
                        self.adjacent_container = Some(container.inventory.sender());
                        // TODO set this back to none later
                    } else {
                        self.set_goal(human, container.location, &world.geography);
                    }
                }
            },
            Activity::Sleeping => if human.fatigue <= 0.0 {
                self.state = Activity::Idle;
            } else if TilePoint::from_vector(&self.home) != TilePoint::from_vector(&human.location) && self.current_path.is_empty() {
                self.set_goal(human, self.home, &world.geography);
            },
        }
    }

    pub fn act(&mut self, human: &mut Human) {
        human.inventory.receive();
        match self.state {
            Activity::Eating => {
                if human.inventory.do_take(Item::Food, 1) {
                    human.hunger -= 1.0;
                } else {
                    if let Some(sender) = &self.adjacent_container {
                        if sender.send(ItemMessage::Take(Item::Food, 1, human.inventory.sender())).is_err() {
                            self.adjacent_container = None;
                        }
                    }
                }
            },
            Activity::Sleeping => if self.current_path.is_empty() {
                human.fatigue -= SLEEP_PER_TICK
            } else {
            },
            _ => (),
        }
        human.fatigue += FATIGUE_PER_TICK;
        human.hunger += HUNGER_PER_TICK;
        self.travel(human);
    }
    
    fn travel(&mut self, human: &mut Human) {
        if let Some(next_tile) = self.current_path.last() {
            let current_tile = TilePoint::from_vector(&human.location);
            if current_tile == *next_tile {
                self.current_path.pop();
            }
        }

        if let Some(next_tile) = self.current_path.last() {
            // Aim for middle of the tile
            let goal = Vector::new(next_tile.x as f32 + 0.5, next_tile.y as f32 + 0.5);
            let direct_path = goal - human.location;
            human.location += direct_path.with_len(human.speed);

            // Clear memory of anything regarding location
            self.adjacent_container = None;
        }
    }

}
