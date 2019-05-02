use crate::geography::{Geography, TilePoint};
use crate::world::{World, Container, TICKS_PER_MINUTE};
use crate::item::{Item, Inventory, ItemMessage};

use std::sync::mpsc::Sender;
use std::cmp::Ordering;

use quicksilver::geom::Vector;
use rand::prelude::*;
use rand::distributions::{Normal, Distribution};

const FATIGUE_PER_TICK: f32 = 1.0 / ( // below is ticks per unit, so invert for units per tick
    (TICKS_PER_MINUTE as f32)
    * 60.0
    * 18.0 // hours non-sleep / day
    * (1.0 / 100.0) // day / units sleep
);

const SLEEP_PER_TICK: f32 = 1.0 / ( // below is ticks per unit, so invert for units per tick
    (TICKS_PER_MINUTE as f32)
    * 60.0
    * 6.0 // hours sleep / day
    * (1.0 / 100.0) // day / units sleep
) + FATIGUE_PER_TICK; // make up for fatigue added even while sleeping

enum Activity {
    Idle,
    Eating(EatingState),
    Sleeping,
}

enum EatingState {
    Eating,
    Finding,
}

pub struct Human {
    pub location: Vector,
    pub inventory: Inventory,
    pub fatigue: f32,
    pub hunger: f32,
    owned_container_indeces: Vec<usize>, // should this be a HashSet to handle duplicates?
    speed: f32,
}

pub struct Mind {
    current_path: Vec<TilePoint>,
    state: Activity,

    home: Vector,

    had_breakfast: bool,
    had_dinner: bool,
    meal_size: u32,
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
            inventory: Inventory::new(100.0),
            speed: 0.1,
            fatigue: 80.0,
            hunger: 0.0,
            owned_container_indeces: Vec::new(),
        }
    }

    fn owned_item_count(&self, item: Item, world: &World) -> u32 {
        let container_count: u32 = self.owned_container_indeces.iter().map(|&i| world.containers[i].inventory.count(item)).sum();
        self.inventory.count(item) + container_count
    }

    pub fn give_container(&mut self, container_index: usize) {
        self.owned_container_indeces.push(container_index)
    }

    fn daily_food(&self, world: &World) -> u32 {
        let owned_food = self.owned_item_count(Item::Food, world);
        if self.hunger < 80.0 {
            40
        } else if self.hunger < 110.0 {
            if owned_food > 80 {
                40
            } else {
                30
            }
        } else if self.hunger < 130.0 {
            if owned_food > 60 {
                30
            } else {
                20
            }
        } else {
            if owned_food > 40 {
                20
            } else {
                10
            }
        }
    }
}


impl Mind {
    pub fn new() -> Mind {
        Mind {
            current_path: Vec::new(),
            state: Activity::Idle,

            home: Vector::new(63.5, 22.5),

            had_breakfast: false,
            had_dinner: false,
            meal_size: 0,
        }
    }

    // For debugging
    pub fn state(&self) -> &'static str {
        match &self.state {
            Activity::Idle => "Idle",
            Activity::Sleeping => "Sleeping",
            Activity::Eating(EatingState::Eating) => "Eating",
            Activity::Eating(EatingState::Finding) => "Find Food",
        }
    }

    fn set_goal(&mut self, human: &Human, goal: Vector, geography: &Geography) {
        let goal = TilePoint::from_vector(&goal);
        let start = TilePoint::from_vector(&human.location);
        self.current_path = geography.find_path(start, goal).unwrap_or(Vec::new());
    }

    // TODO function to calculate value of items based on how much you have stored (exponential decay)
    // only want Wood if you're a crafter

    // TODO go sell at market if you have surplus of the thing you make
    // TODO go buy at market if you have money and free time
    // (hard-coded market location for now)
    
    // TODO when selling, a store has X quanitity to sell and wants to sell it within Y hours.
    // every minute or so, the store will update it's prices so that it is on target to just barely
    // go out of stock at the end of time. This, and assuming buyers buy the (nearly) cheapest
    // goods should simulate supply and demand economics well enough

    pub fn think(&mut self, human: &Human, world: &World) {
        // percieve
        if world.time.is_new_day() {
            self.meal_size = human.daily_food(world) / 2;
            self.had_breakfast = false;
            self.had_dinner = false;
        }

        // think
        match &self.state {
            Activity::Idle => {
                let current_hours = world.time.hour;
                if current_hours > 6 && !self.had_breakfast {
                    self.state = Activity::Eating(EatingState::Finding);
                } else if current_hours > 17 && !self.had_dinner {
                    self.state = Activity::Eating(EatingState::Finding);
                } else if human.fatigue > 80.0 {
                    // TODO sleep based on time of day
                    self.state = Activity::Sleeping;
                } else if self.current_path.is_empty() {
                    let mut rng = thread_rng();
                    if rng.gen::<f32>() > 0.99 {
                        let normal = Normal::new(0.0, 5.0);
                        self.set_goal(human, human.location + Vector::new(normal.sample(&mut rng) as f32, normal.sample(&mut rng) as f32), &world.geography);
                    }
                }
            },

            Activity::Eating(eating_state) => match eating_state {
                EatingState::Finding => if self.current_path.is_empty() {
                    let meal_size = self.meal_size.min(human.hunger as u32);
                    if human.inventory.count(Item::Food) < meal_size {
                        let mut food_containers: Vec<&Container> = human.owned_container_indeces.iter().map(|&i| &world.containers[i]).filter(|&container| container.inventory.count(Item::Food) > 0).collect();
                        food_containers.sort_by_key(|container| MinFloat(human.location.distance(container.location)));
                        if let Some(container) = food_containers.first() {
                            if TilePoint::from_vector(&container.location) == TilePoint::from_vector(&human.location) {
                                container.inventory.sender().send(ItemMessage::Take(Item::Food, meal_size, human.inventory.sender()));
                                // TODO if message doesn't send, do something
                            } else {
                                self.set_goal(human, container.location, &world.geography);
                            }
                        }
                    } else {
                        self.state = Activity::Eating(EatingState::Eating);
                    }
                },
                EatingState::Eating => if human.inventory.count(Item::Food) == 0 {
                    if !self.had_breakfast {
                        self.had_breakfast = true;
                    } else {
                        self.had_dinner = true;
                    }
                    self.state = Activity::Idle;
                },
            },

            Activity::Sleeping => if human.fatigue <= 0.0 {
                self.state = Activity::Idle;
            } else if TilePoint::from_vector(&self.home) != TilePoint::from_vector(&human.location) && self.current_path.is_empty() {
                self.set_goal(human, self.home, &world.geography);
            },
        }
    }

    pub fn act(&mut self, human: &mut Human) {
        match &self.state {
            Activity::Eating(eating_state) => match eating_state {
                EatingState::Eating => if human.inventory.do_take_exact(Item::Food, 1) {
                    human.hunger -= 1.0;
                },
                EatingState::Finding => (), // just traveling
            },

            Activity::Sleeping => if self.current_path.is_empty() {
                human.fatigue -= SLEEP_PER_TICK
            },
            _ => (),
        }
        // TODO make these on a curve (get tired/hungry slower when you're low)
        human.fatigue += FATIGUE_PER_TICK;

        if human.hunger < 80.0 {
            human.hunger += 40.0 / (TICKS_PER_MINUTE as f32 * 60.0 * 24.0);
        } else if human.hunger < 110.0 {
            human.hunger += 30.0 / (TICKS_PER_MINUTE as f32 * 60.0 * 24.0);
        } else if human.hunger < 130.0 {
            human.hunger += 20.0 / (TICKS_PER_MINUTE as f32 * 60.0 * 24.0);
        } else {
            human.hunger += 10.0 / (TICKS_PER_MINUTE as f32 * 60.0 * 24.0);
        }

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
            // TODO aim for middle of nearest edge for smoother pathing
            // Aim for middle of the tile
            let goal = Vector::new(next_tile.x as f32 + 0.5, next_tile.y as f32 + 0.5);
            let direct_path = goal - human.location;
            human.location += direct_path.with_len(human.speed);
        }
    }

}
