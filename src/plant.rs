use quicksilver::geom::Vector;

use crate::item::{Item, Inventory, ItemMessage};

pub struct Crop {
    pub location: Vector,
    pub inventory: Inventory,
}

impl Crop {
    pub fn new(location: Vector) -> Crop {
        Crop {
            location: location,
            inventory: Inventory::new(100.0),
        }
    }

    pub fn grow(&mut self, sun: u32, rain: u32) {
        self.inventory.do_give_up_to(Item::Water, rain);
        let growth = self.inventory.do_take_up_to(Item::Water, sun);
        self.inventory.do_give_up_to(Item::Food, growth);
    }

    pub fn update(&mut self) {
        self.inventory.receive();
    }

}
