use crate::geography::{Geography, TilePoint};
use crate::item::{Inventory, Item, ItemMessage};
use crate::plant::Crop;
use crate::world::{Container, World, TICKS_PER_MINUTE};

use std::cmp::Ordering;
use std::sync::mpsc::Sender;

use quicksilver::geom::Vector;
use rand::distributions::{Distribution, Normal};
use rand::prelude::*;

const FATIGUE_PER_TICK: f32 = 1.0
    / (
        // below is ticks per unit, so invert for units per tick
        (TICKS_PER_MINUTE as f32)
    * 60.0
    * 18.0 // hours non-sleep / day
    * (1.0 / 100.0)
        // day / units sleep
    );

const SLEEP_PER_TICK: f32 =
    1.0 / (
        // below is ticks per unit, so invert for units per tick
        (TICKS_PER_MINUTE as f32)
    * 60.0
    * 6.0 // hours sleep / day
    * (1.0 / 100.0)
        // day / units sleep
    ) + FATIGUE_PER_TICK; // make up for fatigue added even while sleeping

enum Activity {
    Idle,
    Eating(EatingState),
    Sleeping,
    Working(WorkState),
}

enum EatingState {
    Eating,
    Finding,
}

enum WorkState {
    Commuting,
    Working,
    Storing,
}

pub enum Job {
    Farmer(usize), // crop id
}

pub struct Human {
    pub location: Vector,
    pub inventory_id: usize,
    pub fatigue: f32,
    pub hunger: f32,
    owned_container_indeces: Vec<usize>, // should this be a HashSet to handle duplicates? or just one at their home?
    speed: f32,
    job: Job,
}

pub struct Mind {
    current_path: Vec<TilePoint>,
    state: Activity,

    home: Vector,

    had_breakfast: bool,
    had_dinner: bool,
    meal_size: u32,

    target_inventory_id: Option<usize>,

    progress: u32,

    wait: u32,
    travel_vector: Option<Vector>,
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
    pub fn new(location: Vector, inventory_id: usize, job: Job) -> Human {
        Human {
            location: location,
            inventory_id: inventory_id,
            speed: 0.1,
            fatigue: 80.0,
            hunger: 0.0,
            owned_container_indeces: Vec::new(),
            job: job,
        }
    }

    pub fn description_lines(&self, world: &World) -> Vec<String> {
        let mut lines = vec!["Human: <name>".to_string()];
        lines.push(format!("Fatigue: {:.2}", self.fatigue));
        lines.push(format!("Hunger: {:.2}", self.hunger));
        lines.push(format!("Job: {}", match self.job {
            Job::Farmer(_) => "Farmer",
        }));

        let mut inventory_lines = world.inventories[self.inventory_id].description_lines();
        lines.append(&mut inventory_lines);
        lines
    }

    fn owned_item_count(&self, item: Item, world: &World) -> u32 {
        let container_count: u32 = self
            .owned_container_indeces
            .iter()
            .map(|&i| world.inventories[world.containers[i].inventory_id].count(item))
            .sum();
        world.inventories[self.inventory_id].count(item) + container_count
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
    pub fn new(home: Vector) -> Mind {
        Mind {
            current_path: Vec::new(),
            state: Activity::Idle,

            home: home,

            had_breakfast: false,
            had_dinner: false,
            meal_size: 0,

            target_inventory_id: None,
            progress: 0,

            wait: 0,
            travel_vector: None,
        }
    }

    // For debugging
    pub fn state(&self) -> &'static str {
        match &self.state {
            Activity::Idle => "Idle",
            Activity::Sleeping => "Sleeping",
            Activity::Eating(EatingState::Eating) => "Eating",
            Activity::Eating(EatingState::Finding) => "Find Food",
            Activity::Working(WorkState::Commuting) => "Work: Commuting",
            Activity::Working(WorkState::Working) => "Work: Working",
            Activity::Working(WorkState::Storing) => "Work: Storing",
        }
    }

    fn set_goal(&mut self, human: &Human, goal: Vector, geography: &Geography) {
        let goal = TilePoint::from_vector(&goal);
        let start = TilePoint::from_vector(&human.location);
        self.current_path = geography.find_path(start, goal).unwrap_or(Vec::new());
    }

    // TODO implement jobs (just farming first)

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
            self.progress = 0;
        }

        // think
        if self.wait > 0 {
            self.wait -= 1;
        }
        if self.wait == 0 {
            match &self.state {
                Activity::Idle => {
                    let current_hours = world.time.hour;
                    if current_hours > 6 && !self.had_breakfast {
                        self.state = Activity::Eating(EatingState::Finding);
                    } else if current_hours > 7 && self.progress == 0 {
                        self.state = Activity::Working(WorkState::Commuting);
                    } else if current_hours > 17 && !self.had_dinner {
                        self.state = Activity::Eating(EatingState::Finding);
                    } else if human.fatigue > 80.0 {
                        // TODO sleep based on time of day
                        self.state = Activity::Sleeping;
                    } else if self.current_path.is_empty() {
                        let mut rng = thread_rng();
                        if rng.gen::<f32>() > 0.99 {
                            let normal = Normal::new(0.0, 5.0);
                            self.set_goal(
                                human,
                                human.location
                                    + Vector::new(
                                        normal.sample(&mut rng) as f32,
                                        normal.sample(&mut rng) as f32,
                                    ),
                                &world.geography,
                            );
                        }
                    }
                }

                Activity::Eating(eating_state) => match eating_state {
                    EatingState::Finding => {
                        if self.current_path.is_empty() {
                            if world.inventories[human.inventory_id].count(Item::Food) == 0 {
                                let mut food_containers: Vec<&Container> = human
                                    .owned_container_indeces
                                    .iter()
                                    .map(|&i| &world.containers[i])
                                    .filter(|&container| {
                                        world.inventories[container.inventory_id].count(Item::Food)
                                            > 0
                                    })
                                    .collect();
                                food_containers.sort_by_key(|container| {
                                    MinFloat(human.location.distance(container.location))
                                });
                                if let Some(container) = food_containers.first() {
                                    if TilePoint::from_vector(&container.location)
                                        == TilePoint::from_vector(&human.location)
                                    {
                                        self.target_inventory_id = Some(container.inventory_id);
                                    } else {
                                        self.set_goal(human, container.location, &world.geography);
                                    }
                                }
                            } else {
                                self.state = Activity::Eating(EatingState::Eating);
                            }
                        }
                    }
                    EatingState::Eating => {
                        if world.inventories[human.inventory_id].count(Item::Food) == 0 {
                            if !self.had_breakfast {
                                self.had_breakfast = true;
                            } else {
                                self.had_dinner = true;
                            }
                            self.state = Activity::Idle;
                        }
                    }
                },

                Activity::Working(work_state) => match &human.job {
                    Job::Farmer(crop_id) => match work_state {
                        WorkState::Commuting => {
                            if self.current_path.is_empty() {
                                let crop = &world.crops[*crop_id];
                                if TilePoint::from_vector(&human.location)
                                    != TilePoint::from_vector(&crop.location)
                                {
                                    self.set_goal(human, crop.location, &world.geography);
                                } else {
                                    self.state = Activity::Working(WorkState::Working);
                                    self.target_inventory_id = Some(crop.inventory_id);
                                }
                            }
                        }
                        WorkState::Working => {
                            // TODO wander around crop tile
                            if self.progress > TICKS_PER_MINUTE as u32 * 60 * 6 {
                                // work 6 hours / day
                                self.state = Activity::Working(WorkState::Storing);
                                self.target_inventory_id = None;
                            }
                        }
                        WorkState::Storing => {
                            if self.current_path.is_empty() {
                                if world.inventories[human.inventory_id].count(Item::Food) != 0 {
                                    // TODO find an container with enough space. or at least exclude
                                    // containers with no space
                                    let mut food_containers: Vec<&Container> = human
                                        .owned_container_indeces
                                        .iter()
                                        .map(|&i| &world.containers[i])
                                        .collect();
                                    food_containers.sort_by_key(|container| {
                                        MinFloat(human.location.distance(container.location))
                                    });
                                    if let Some(container) = food_containers.first() {
                                        if TilePoint::from_vector(&container.location)
                                            == TilePoint::from_vector(&human.location)
                                        {
                                            self.target_inventory_id = Some(container.inventory_id);
                                        } else {
                                            self.set_goal(
                                                human,
                                                container.location,
                                                &world.geography,
                                            );
                                        }
                                    }
                                } else {
                                    self.state = Activity::Idle;
                                }
                            }
                        }
                    },
                },

                Activity::Sleeping => {
                    if human.fatigue <= 0.0 {
                        self.state = Activity::Idle;
                    } else if TilePoint::from_vector(&self.home)
                        != TilePoint::from_vector(&human.location)
                        && self.current_path.is_empty()
                    {
                        self.set_goal(human, self.home, &world.geography);
                    }
                }
            }
            self.update_travel(human);
        }
    }

    pub fn update_travel(&mut self, human: &Human) {
        if let Some(next_tile) = self.current_path.last() {
            let current_tile = TilePoint::from_vector(&human.location);
            if current_tile == *next_tile {
                self.current_path.pop();
            }
        }

        if let Some(next_tile) = self.current_path.last() {
            let current_tile = TilePoint::from_vector(&human.location);
            // aim for middle of nearest edge for smoother pathing
            let goal = if current_tile.x > next_tile.x {
                Vector::new(next_tile.x as f32 + 1.0, next_tile.y as f32 + 0.5)
            } else if current_tile.x < next_tile.x {
                Vector::new(next_tile.x as f32, next_tile.y as f32 + 0.5)
            } else if current_tile.y > next_tile.y {
                Vector::new(next_tile.x as f32 + 0.5, next_tile.y as f32 + 1.0)
            } else {
                Vector::new(next_tile.x as f32 + 0.5, next_tile.y as f32 + 0.0)
            };
            self.travel_vector = Some(goal - human.location);
        } else {
            self.travel_vector = None;
        }
    }

    pub fn act(&mut self, human: &mut Human, inventory_senders: &Vec<Sender<ItemMessage>>) {
        if self.wait == 0 {
            match &self.state {
                Activity::Idle => (),

                Activity::Eating(eating_state) => match eating_state {
                    EatingState::Eating => {
                        inventory_senders[human.inventory_id]
                            .send(ItemMessage::Remove(Item::Food, 1));
                        human.hunger -= 1.0;
                    }
                    EatingState::Finding => {
                        if let Some(target_inventory_id) = self.target_inventory_id {
                            inventory_senders[target_inventory_id].send(ItemMessage::Take(
                                Item::Food,
                                self.meal_size.min(human.hunger as u32),
                                human.inventory_id,
                            ));
                            self.target_inventory_id = None;
                            self.wait = 2;
                        }
                    }
                },

                Activity::Working(work_state) => match &human.job {
                    Job::Farmer(_) => match work_state {
                        WorkState::Commuting => (), // let travel do the work
                        WorkState::Working => {
                            self.progress += 1;
                            if self.progress > TICKS_PER_MINUTE as u32 * 60 * 6 {
                                // work 6 hours / day
                                if let Some(target_inventory_id) = self.target_inventory_id {
                                    inventory_senders[target_inventory_id].send(ItemMessage::Take(
                                        Item::Food,
                                        50,
                                        human.inventory_id,
                                    )); // TODO calculate remaining capacity in think/perceive and use that
                                    self.target_inventory_id = None;
                                    self.wait = 2;
                                } // TODO else?
                            }
                        }
                        WorkState::Storing => {
                            if let Some(target_inventory_id) = self.target_inventory_id {
                                inventory_senders[human.inventory_id].send(ItemMessage::Transfer(
                                    human.inventory_id,
                                    Item::Food,
                                    100,
                                    target_inventory_id,
                                )); // TODO is this the best way to give all?
                                self.target_inventory_id = None;
                            }
                        }
                    },
                },

                Activity::Sleeping => {
                    if self.current_path.is_empty() {
                        human.fatigue -= SLEEP_PER_TICK
                    }
                }
            }
        }
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
    }

    pub fn travel(&self, human: &mut Human, frames_per_tick: u8) {
        if let Some(travel_vector) = self.travel_vector {
            human.location += travel_vector.with_len(human.speed / frames_per_tick as f32);
        }
    }
}
