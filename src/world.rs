use quicksilver::geom::Vector;

use crate::geography::Geography;
use crate::human::Human;
use crate::item::Inventory;

pub const TICKS_PER_HOUR: u32 = 200;

pub struct World {
    pub geography: Geography,
    pub humans: Vec<Human>,
    pub containers: Vec<Container>,
    pub time: Time,
}

pub struct Container {
    pub location: Vector,
    pub inventory: Inventory,
}

pub struct Time {
    ticks: u64,
}

impl Time {
    pub fn new() -> Time {
        Time {ticks: 0}
    }

    pub fn tick(&mut self) {
        self.ticks += 1;
    }

    pub fn current_day(&self) -> u64 {
        self.ticks / (TICKS_PER_HOUR as u64 * 24u64)
    }

    pub fn current_hour(&self) -> u64 {
        (self.ticks / TICKS_PER_HOUR as u64) % 24u64
    }

    pub fn current_minute(&self) -> u64 {
        (self.ticks % TICKS_PER_HOUR as u64) * 60 / TICKS_PER_HOUR as u64
    }
 
    pub fn is_new_hour(&self) -> bool {
        self.ticks % TICKS_PER_HOUR as u64 == 0
    }

    pub fn is_new_day(&self) -> bool {
        self.ticks % (TICKS_PER_HOUR as u64 * 24u64) == 0
    }
    // TODO new week?
    // TODO new month?
    // TODO new season?
    // TODO new year?
}
