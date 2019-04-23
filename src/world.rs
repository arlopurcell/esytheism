use quicksilver::geom::Vector;

use crate::geography::Geography;
use crate::human::Human;
use crate::item::Inventory;

pub const TICKS_PER_HOUR: u32 = 200;

pub struct World {
    pub geography: Geography,
    pub humans: Vec<Human>,
    pub containers: Vec<Container>,
}

pub struct Container {
    pub location: Vector,
    pub inventory: Inventory,
}
