mod geography;
mod human;
mod item;
mod world;

//use rand::distributions::{Distribution, Uniform, Normal};
//use rayon::prelude::*;

use quicksilver::{
    Result,
    geom::{Rectangle, Vector, Circle},
    graphics::{Background::Col, Color},
    // input::{ButtonState, Key},
    lifecycle::{Settings, State, Window, run},
};

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use std::u32;

use crate::geography::Geography;
use crate::human::{Human, Mind};
use crate::item::{Item, Inventory};
use crate::world::{World, Container};

struct GameState {
    world: World,
    minds: Vec<Mind>,
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
                        '.' => 10,
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

        let human = Human::new(Vector::new(40.5, 40.0));
        let mind = Mind::new();

        let mut food_box = Container {
            location: Vector::new(39.5, 19.5),
            inventory: Inventory::new(),
        };
        food_box.inventory.do_give(Item::Food, u32::MAX);

        Ok(GameState {
            world: World {
                geography: geo,
                humans: vec![human],
                containers: vec![food_box],
            },
            minds: vec![mind],
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
        
        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::BLACK)?;
        for x in 0..80 {
            for y in 0..60 {
                let cost = self.world.geography.tile_costs[x][y] as u8;
                window.draw(&Rectangle::new((x as u32 * 10, y as u32 * 10), (10, 10)), Col(match cost {
                    1 => Color::from_rgba(191, 156, 116, 1.0),
                    10 => Color::from_rgba(127, 234, 117, 1.0),
                    _ => Color::BLACK,
                }));
            }
        }
        for human in &self.world.humans {
            window.draw(&Circle::new(human.location * 10.0, 5.0), Col(Color::RED));
        }
        Ok(())
    }
}

fn main() {
    run::<GameState>("Esytheism", Vector::new(800, 600), Settings::default());
}
