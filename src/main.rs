mod geography;
mod human;
mod item;
mod world;
mod weather;
mod plant;

//use rand::distributions::{Distribution, Uniform, Normal};
//use rayon::prelude::*;

use quicksilver::{
    Result,
    geom::{Rectangle, Vector, Circle},
    graphics::{Background::Col, Background::Img, Color, Image, Font, FontStyle},
    // input::{ButtonState, Key},
    lifecycle::{Settings, State, Window, run, Asset},
};

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use std::u32;

use crate::geography::Geography;
use crate::human::{Human, Mind};
use crate::item::{Item, Inventory};
use crate::world::{World, Container, Time};
use crate::weather::Weather;

struct GameState {
    world: World,
    minds: Vec<Mind>,
    font: Asset<Font>,
}

impl State for GameState {
    fn new() -> Result<GameState> {
        let file = File::open("data/test.map").unwrap();
        let reader = BufReader::new(file);

        let raw_data: Vec<Vec<u32>> = reader
            .lines()
            .filter_map(|line| line.ok())
            .map(|line| {
                line.chars()
                    .map(|c| match c {
                        '_' => 1,
                        '.' => 5,
                        _ => u32::MAX,
                    })
                    .collect()
            })
            .collect();

        let mut tile_costs = vec![vec![1; 60]; 80];
        for i in 0..60 {
            for j in 0..80 {
                tile_costs[j][i] = raw_data[i][j];
            }
        }

        let geo = Geography::new(tile_costs.clone(), 80, 60);

        let mut human = Human::new(Vector::new(50.5, 30.0));
        let mind = Mind::new();

        let mut food_box = Container {
            location: Vector::new(39.5, 19.5),
            inventory: Inventory::new(10e10),
        };
        food_box.inventory.do_give_up_to(Item::Food, u32::MAX);
        human.give_container(0);

        let font = Asset::new(Font::load("anonymous_pro.ttf"));

        Ok(GameState {
            world: World {
                geography: geo,
                time: Time::new(), 
                humans: vec![human],
                containers: vec![food_box],
                weather: Weather::new(),
                crops: Vec::new(),
            },
            minds: vec![mind],
            font: font,
        })
    }

    // fn event(&mut self, event: &Event, window: &mut Window) -> Result<()> {
    //     match *event {
    //         Event::Key(Key::Space, ButtonState::Pressed) => {
    //         },
    //         _ => (),
    //     }
    //     Ok(())
    // }

    fn update(&mut self, _window: &mut Window) -> Result<()> {
        if self.world.time.is_new_day() {
            self.world.weather.update();
            let (sun, rain) = (self.world.weather.sun(), self.world.weather.rain());
            for crop in self.world.crops.iter_mut() {
                crop.grow(sun, rain);
            }
        }
        for human in self.world.humans.iter_mut() {
            human.inventory.receive();
        }
        {
            let world = &self.world;
            for (human, mind) in self.world.humans.iter().zip(self.minds.iter_mut()) {
                mind.think(human, world);
            }
        }
        for (human, mind) in self.world.humans.iter_mut().zip(self.minds.iter_mut()) {
            mind.act(human);
        }
        for container in self.world.containers.iter_mut() {
            container.inventory.receive();
        }
        for crop in self.world.crops.iter_mut() {
            crop.update();
        }
        self.world.time.tick();
        
        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::BLACK)?;
        for x in 0..80 {
            for y in 0..60 {
                let cost = self.world.geography.tile_costs[x][y] as u8;
                window.draw(&Rectangle::new((x as u32 * 10, y as u32 * 10), (10, 10)), Col(match cost {
                    1 => Color::from_rgba(191, 156, 116, 1.0),
                    5 => Color::from_rgba(127, 234, 117, 1.0),
                    _ => Color::BLACK,
                }));
            }
        }
        for human in &self.world.humans {
            window.draw(&Circle::new(human.location * 10.0, 5.0), Col(Color::RED));
            self.font.execute(|font| {
                window.draw(&Rectangle::new((200, 550), (400, 40)), Col(Color::BLACK));
                let style = FontStyle::new(36.0, Color::WHITE);
                let text = format!("hunger: {:.2}", human.hunger);
                let text_img = font.render(&text, &style).unwrap();
                window.draw(&Rectangle::new((210, 553), text_img.area().size()), Img(&text_img));
                Ok(())
            });
        }

        /*
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
        */
        
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
