use std::sync::mpsc::{channel, Sender, Receiver};
use std::collections::HashMap;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum Item {
    Food,
    Money,
    Water,
    // TODO Alcohol (crafted from Food)
    // TODO Wood,
    // TODO LuxuryGood (crafted from Wood),
}

pub enum ItemMessage {
    Give(Item, u32, usize),
    GiveOrDrop(Item, u32),
    Trade((Item, u32), (Item, u32), usize),
    Take(Item, u32, usize),
    Remove(Item, u32),
}

pub struct Inventory {
    // id: u32,
    items: HashMap<Item, u32>,
    capacity: f32,
}

impl Item {
    fn weight(&self) -> f32 {
        match self {
            Item::Food => 1.0,
            Item::Money => 0.1,
            Item::Water => 1.0,
        }
    }
}

impl Inventory {
    pub fn new(capacity: f32) -> Inventory {
        Inventory {
            // id: id,
            items: HashMap::new(),
            capacity: capacity,
        }
    }

    pub fn receive_all(&mut self, receiver: &Receiver<ItemMessage>, senders: &Vec<Sender<ItemMessage>>) {
        while let Ok(msg) = receiver.try_recv() {
            self.process_msg(msg, senders);
        }
    }

    fn process_msg(&mut self, msg: ItemMessage, senders: &Vec<Sender<ItemMessage>>) {
        match msg {
            ItemMessage::Give(received_item, received_quantity, ack_sender_id) => {
                let given_quantity = self.do_give_up_to(received_item, received_quantity);
                let ungiven = received_quantity - given_quantity;
                if ungiven > 0 {
                    let _ = senders[ack_sender_id].send(ItemMessage::GiveOrDrop(received_item, ungiven));
                }
            },
            ItemMessage::GiveOrDrop(received_item, received_quantity) => {
                self.do_give_up_to(received_item, received_quantity);
            },
            ItemMessage::Trade((received_item, received_quantity), (requested_item, requested_quantity), ack_sender_id) => if self.do_take_exact(requested_item, requested_quantity) {
                if self.do_give_exact(received_item, received_quantity) {
                    let _ = senders[ack_sender_id].send(ItemMessage::GiveOrDrop(requested_item, requested_quantity));
                } else {
                    self.do_give_up_to(requested_item, requested_quantity);
                    let _ = senders[ack_sender_id].send(ItemMessage::GiveOrDrop(received_item, received_quantity));
                }
            } else {
                let _ = senders[ack_sender_id].send(ItemMessage::GiveOrDrop(received_item, received_quantity));
            },
            ItemMessage::Take(taken_item, taken_quantity, ack_sender_id) => {
                let taken_quantity = self.do_take_up_to(taken_item, taken_quantity);
                let _ = senders[ack_sender_id].send(ItemMessage::GiveOrDrop(taken_item, taken_quantity));
            },
            ItemMessage::Remove(taken_item, taken_quantity) => {
                let _ = self.do_take_up_to(taken_item, taken_quantity);
            },
        }
    }
    
    fn weight(&self) -> f32 {
        self.items.iter().map(|(item, count)| item.weight() * (*count as f32)).sum()
    }

    pub fn do_give_exact(&mut self, received_item: Item, received_quantity: u32) -> bool {
        let current_weight = self.weight();
        let added_weight = received_item.weight() + received_quantity as f32;
        if current_weight + added_weight >= self.capacity {
            if let Some(existing_quantity) = self.items.get_mut(&received_item) {
                *existing_quantity += received_quantity;
            } else {
                self.items.insert(received_item, received_quantity);
            }
            true
        } else {
            false
        }
    }

    pub fn do_give_up_to(&mut self, received_item: Item, received_quantity: u32) -> u32 {
        let current_weight = self.weight();
        let remaining_weight = self.capacity - current_weight;
        let remaining_quantity = (remaining_weight / received_item.weight()).floor() as u32;
        let given_quantity = remaining_quantity.min(received_quantity);
        if let Some(existing_quantity) = self.items.get_mut(&received_item) {
            *existing_quantity += given_quantity;
        } else {
            self.items.insert(received_item, given_quantity);
        }
        given_quantity
    }
    
    pub fn do_take_exact(&mut self, taken_item: Item, taken_quantity: u32) -> bool {
        if let Some(existing_quantity) = self.items.get_mut(&taken_item) {
            if *existing_quantity >= taken_quantity {
                *existing_quantity -= taken_quantity;
                return true;
            }
        }
        false
    }

    pub fn do_take_up_to(&mut self, taken_item: Item, taken_quantity: u32) -> u32 {
        if let Some(existing_quantity) = self.items.get_mut(&taken_item) {
            let taken_quantity = taken_quantity.min(*existing_quantity);
            *existing_quantity -= taken_quantity;
            taken_quantity
        } else {
            0
        }
    }

    pub fn count(&self, item: Item) -> u32 {
        *self.items.get(&item).unwrap_or(&0)
    }

}
