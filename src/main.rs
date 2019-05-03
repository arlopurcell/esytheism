mod geography;
mod human;
mod item;
mod world;
mod weather;
mod plant;

use rand::distributions::{Distribution, Uniform, Normal};
use rayon::prelude::*;

use quicksilver::{
    Result,
    geom::{Rectangle, Vector, Circle},
    graphics::{Background::Col, Background::Img, Color, Image, Font, FontStyle},
    input::{ButtonState, Key},
    lifecycle::{Settings, State, Window, run, Asset, Event},
    load_file,
    Future,
};

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::collections::HashMap;
use std::num::Wrapping;

use std::u32;

use rand::prelude::*;

use crate::geography::Geography;
use crate::human::{Human, Mind};
use crate::item::{Item, Inventory, ItemMessage};
use crate::world::{World, Container, Time};
use crate::weather::Weather;

struct GameState {
    world: World,
    minds: Vec<Mind>,
    font: Asset<Font>,
    inventory_senders: Vec<Sender<ItemMessage>>,
    inventory_receivers: Vec<Receiver<ItemMessage>>,
    paused: bool,
    updates_per_tick: u8,
    counter: u8,
}

impl GameState {
    fn create_inventory(&mut self, capacity: f32) -> usize {
        let index = self.world.inventories.len();
        let (send, recv) = channel();
        self.world.inventories.push(Inventory::new(capacity));
        self.inventory_senders.push(send);
        self.inventory_receivers.push(recv);
        index
    }
}

impl State for GameState {
    fn new() -> Result<GameState> {
        let geo = load_file("data/test.map").map(|data| Geography::from_data(40, 30, &data)).wait().unwrap();
        
        let font = Asset::new(Font::load("anonymous_pro.ttf"));
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
            font: font,
            inventory_senders: Vec::new(),
            inventory_receivers: Vec::new(),
            paused: false,
            updates_per_tick: 1,
            counter: 0,
        };

        let mut human = Human::new(Vector::new(25.5, 15.0), gs.create_inventory(100.0));
        let mind = Mind::new(Vector::new(29.5, 14.5));

        let mut food_box = Container {
            location: Vector::new(17.5, 9.5),
            inventory_id: gs.create_inventory(10e10),
        };
        gs.world.inventories[food_box.inventory_id].do_give_up_to(Item::Food, u32::MAX);
        human.give_container(0);

        // TODO add some crops

        gs.world.humans.push(human);
        gs.minds.push(mind);
        gs.world.containers.push(food_box);

        Ok(gs)
    }

    fn event(&mut self, event: &Event, window: &mut Window) -> Result<()> {
        match *event {
            Event::Key(Key::Space, ButtonState::Pressed) => {
                self.paused = ! self.paused;
            },
            Event::Key(Key::Left, ButtonState::Pressed) => {
                if self.updates_per_tick < 64 {
                    self.updates_per_tick *= 2;
                }
            }
            Event::Key(Key::Right, ButtonState::Pressed) => {
                if self.updates_per_tick > 1 {
                    self.updates_per_tick /= 2;
                }
            }
            _ => (),
        }
        Ok(())
    }

    fn update(&mut self, _window: &mut Window) -> Result<()> {
        if !self.paused && ({self.counter = self.counter.wrapping_add(1); self.counter} % self.updates_per_tick) == 0 {
            if self.world.time.is_new_day() {
                self.world.weather.update();
                let (sun, rain) = (self.world.weather.sun(), self.world.weather.rain());
                self.world.crops.par_iter_mut().for_each_with(self.inventory_senders.clone(), |senders, crop| crop.grow(sun, rain, senders));
            }
            {
                let world = &self.world;
                self.minds.par_iter_mut().zip(&self.world.humans).for_each(|(mind, human)| mind.think(&human, world));
            }

            self.minds.par_iter_mut().zip(self.world.humans.par_iter_mut()).for_each_with(self.inventory_senders.clone(), |senders, (mind, human)| mind.act(human, senders));
            self.inventory_receivers.par_iter_mut().zip(self.world.inventories.par_iter_mut()).for_each_with(self.inventory_senders.clone(), |senders, (recv, inventory)| inventory.receive_all(recv, senders));
            self.world.time.tick();
        }
        if !self.paused {
            let updates_per_tick = self.updates_per_tick;
            self.minds.par_iter_mut().zip(self.world.humans.par_iter_mut()).for_each(|(mind, human)| mind.travel(human, updates_per_tick));
        }
        
        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::BLACK)?;
        for x in 0..self.world.geography.width {
            for y in 0..self.world.geography.height {
                let tile = &self.world.geography.tiles[x][y];
                window.draw(&Rectangle::new((x as u32 * 20, y as u32 * 20), (20, 20)), Col(match tile.terrain_cost {
                    1 => Color::from_rgba(191, 156, 116, 1.0),
                    _ => Color::from_rgba(127, 234, 117, 1.0),
                }));
                // draw walls
                if tile.walls[0] {
                    window.draw(&Rectangle::new((x as i32 * 20 - 1, y as i32 * 20 - 1), (22, 2)), Col(Color::BLACK));
                }
                if tile.walls[1] {
                    window.draw(&Rectangle::new((x as i32 * 20 + 19, y as i32 * 20 - 1), (2, 22)), Col(Color::BLACK));
                }
                if tile.walls[2] {
                    window.draw(&Rectangle::new((x as i32 * 20 - 1, y as i32 * 20  + 19), (22, 2)), Col(Color::BLACK));
                }
                if tile.walls[3] {
                    window.draw(&Rectangle::new((x as i32 * 20 - 1, y as i32 * 20 - 1), (2, 22)), Col(Color::BLACK));
                }
            }
        }
        for human in &self.world.humans {
            window.draw(&Circle::new(human.location * 20.0, 3.0), Col(Color::RED));
            // self.font.execute(|font| {
            //     window.draw(&Rectangle::new((200, 550), (400, 40)), Col(Color::BLACK));
            //     let style = FontStyle::new(36.0, Color::WHITE);
            //     let text = format!("hunger: {:.2}", human.hunger);
            //     let text_img = font.render(&text, &style).unwrap();
            //     window.draw(&Rectangle::new((210, 553), text_img.area().size()), Img(&text_img));
            //     Ok(())
            // });
        }

        for mind in &self.minds {
            self.font.execute(|font| {
                window.draw(&Rectangle::new((200, 550), (400, 50)), Col(Color::BLACK));
                let style = FontStyle::new(48.0, Color::WHITE);
                let text = format!("state: {}", mind.state());
                let text_img = font.render(&text, &style).unwrap();
                window.draw(&Rectangle::new((210, 553), text_img.area().size()), Img(&text_img));
                Ok(())
            });
        }
        
        let world_time = &self.world.time;
        self.font.execute(|font| {
            window.draw(&Rectangle::new((570, 0), (230, 20)), Col(Color::BLACK));
            let style = FontStyle::new(18.0, Color::WHITE);
            let time_img = font.render(&world_time.date_string(), &style).unwrap();
            window.draw(&Rectangle::new((572, 1), time_img.area().size()), Img(&time_img));
            Ok(())
        });
        Ok(())
    }
}

fn main() {
    run::<GameState>("Esytheism", Vector::new(800, 600), Settings::default());
}
