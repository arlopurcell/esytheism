use std::sync::mpsc::{channel, Sender, Receiver};
use std::collections::HashMap;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum Item {
    Food,
    Money,
}

pub enum ItemMessage {
    Give(Item, u32),
    Trade((Item, u32), (Item, u32), Sender<ItemMessage>),
    Take(Item, u32, Sender<ItemMessage>),
}

pub struct Inventory {
    receiver: Receiver<ItemMessage>,
    sender: Sender<ItemMessage>,
    items: HashMap<Item, u32>,
}

impl Inventory {
    pub fn new() -> Inventory {
        let (sender, receiver) = channel();
        Inventory {
            receiver: receiver,
            sender: sender,
            items: HashMap::new(),
        }
    }

    pub fn receive(&mut self) {
        while let Ok(msg) = self.receiver.try_recv() {
            match msg {
                ItemMessage::Give(received_item, received_quantity) => self.do_give(received_item, received_quantity),
                ItemMessage::Trade((received_item, received_quantity), (requested_item, requested_quantity), ack_sender) => if self.do_take(requested_item, requested_quantity) {
                    let _ = ack_sender.send(ItemMessage::Give(requested_item, requested_quantity));
                    self.do_give(requested_item, received_quantity);
                } else {
                    let _ = ack_sender.send(ItemMessage::Give(received_item, received_quantity));
                },
                ItemMessage::Take(taken_item, taken_quantity, ack_sender) => if self.do_take(taken_item, taken_quantity) {
                    let _ = ack_sender.send(ItemMessage::Give(taken_item, taken_quantity));
                },
            }
        }
    }

    pub fn do_give(&mut self, received_item: Item, received_quantity: u32) {
        if let Some(existing_quantity) = self.items.get_mut(&received_item) {
            *existing_quantity += received_quantity;
        } else {
            self.items.insert(received_item, received_quantity);
        }
    }
    
    pub fn do_take(&mut self, taken_item: Item, taken_quantity: u32) -> bool {
        if let Some(existing_quantity) = self.items.get_mut(&taken_item) {
            if *existing_quantity >= taken_quantity {
                *existing_quantity -= taken_quantity;
                return true;
            }
        }
        false
    }

    pub fn count(&self, item: Item) -> u32 {
        *self.items.get(&item).unwrap_or(&0)
    }

    pub fn sender(&self) -> Sender<ItemMessage> {
        self.sender.clone()
    }

}
