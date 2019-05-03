use std::sync::mpsc::Sender;

use quicksilver::geom::Vector;

use crate::item::{Item, Inventory, ItemMessage};

pub struct Crop {
    pub location: Vector,
    pub inventory_id: usize,
}

impl Crop {
    pub fn new(location: Vector, inventory_id: usize) -> Crop {
        Crop {
            location: location,
            inventory_id: inventory_id,
        }
    }

    pub fn grow(&mut self, sun: u32, rain: u32, senders: &Vec<Sender<ItemMessage>>) {
        let sender = &senders[self.inventory_id];

        // self.inventory.do_give_up_to(Item::Water, rain);
        sender.send(ItemMessage::GiveOrDrop(Item::Water, rain));

        // let growth = self.inventory.do_take_up_to(Item::Water, sun);
        // self.inventory.do_give_up_to(Item::Food, growth);
        for _ in 0..sun {
            sender.send(ItemMessage::Trade((Item::Food, 1), (Item::Water, 1), self.inventory_id));
        }
    }
}
