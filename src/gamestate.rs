use rand::distributions::{Distribution, Normal, Uniform};
use rayon::prelude::*;

use quicksilver::{
    geom::{Circle, Rectangle, Vector},
    graphics::{Background::Col, Background::Img, Color, Font, FontStyle, Image},
    input::{ButtonState, Key},
    lifecycle::{run, Asset, Event, Settings, State, Window},
    load_file, Future, Result,
};

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::num::Wrapping;
use std::sync::mpsc::{channel, Receiver, Sender};

use std::u32;

use rand::prelude::*;

use crate::geography::Geography;
use crate::human::{Human, Job, Mind};
use crate::item::{Inventory, Item, ItemMessage};
use crate::plant::Crop;
use crate::weather::Weather;
use crate::world::{Container, Time, World};

pub struct GameState {
    pub world: World,
    pub minds: Vec<Mind>,
    inventory_senders: Vec<Sender<ItemMessage>>,
    inventory_receivers: Vec<Receiver<ItemMessage>>,
}

impl GameState {
    pub fn new() -> GameState {
        let geo = load_file("data/test.map")
            .map(|data| Geography::from_data(40, 30, &data))
            .wait()
            .unwrap();

        let mut gs = GameState {
            world: World {
                geography: geo,
                time: Time::new(),
                humans: Vec::new(),
                containers: Vec::new(),
                weather: Weather::new(),
                crops: Vec::new(),
                inventories: Vec::new(),
            },
            minds: Vec::new(),
            inventory_senders: Vec::new(),
            inventory_receivers: Vec::new(),
        };

        let mut crop = Crop::new(Vector::new(29.5, 16.5), gs.create_inventory(10.0));
        // TODO remove, just testing storage by making sure crop has grown some food
        gs.world.inventories[crop.inventory_id].do_give_up_to(Item::Food, 10);

        gs.world.crops.push(crop);

        let mut human = Human::new(
            Vector::new(25.5, 15.0),
            gs.create_inventory(100.0),
            Job::Farmer(0),
        );
        let mind = Mind::new(Vector::new(29.5, 14.5));

        let mut food_box = Container {
            location: Vector::new(29.5, 14.5),
            inventory_id: gs.create_inventory(10e10),
        };
        gs.world.inventories[food_box.inventory_id].do_give_up_to(Item::Food, 100);
        human.give_container(0);

        gs.world.humans.push(human);
        gs.minds.push(mind);
        gs.world.containers.push(food_box);

        gs
    }

    pub fn update(&mut self) {
        if self.world.time.is_new_day() {
            self.world.weather.update();
            let (sun, rain) = (self.world.weather.sun(), self.world.weather.rain());
            self.world
                .crops
                .par_iter_mut()
                .for_each_with(self.inventory_senders.clone(), |senders, crop| {
                    crop.grow(sun, rain, senders)
                });
        }
        {
            let world = &self.world;
            self.minds
                .par_iter_mut()
                .zip(&self.world.humans)
                .for_each(|(mind, human)| mind.think(&human, world));
        }

        self.minds
            .par_iter_mut()
            .zip(self.world.humans.par_iter_mut())
            .for_each_with(self.inventory_senders.clone(), |senders, (mind, human)| {
                mind.act(human, senders)
            });
        self.inventory_receivers
            .par_iter_mut()
            .zip(self.world.inventories.par_iter_mut())
            .for_each_with(
                self.inventory_senders.clone(),
                |senders, (recv, inventory)| inventory.receive_all(recv, senders),
            );
        self.world.time.tick();
    }

    pub fn do_travel(&mut self, updates_per_tick: u8) {
        self.minds
            .par_iter_mut()
            .zip(self.world.humans.par_iter_mut())
            .for_each(|(mind, human)| mind.travel(human, updates_per_tick));
    }

    fn create_inventory(&mut self, capacity: f32) -> usize {
        let index = self.world.inventories.len();
        let (send, recv) = channel();
        self.world.inventories.push(Inventory::new(capacity));
        self.inventory_senders.push(send);
        self.inventory_receivers.push(recv);
        index
    }
}
