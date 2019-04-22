mod geography;
mod human;

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
use crate::human::Human;

struct GameState {
    geography: geography::Geography,
    humans: Vec<human::Human>,
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

        Ok(GameState {
            geography: geo,
            humans: vec![human],
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
        let geography = &self.geography;
        self.humans.iter_mut().for_each(|human| {
            human.think(geography);
            human.act();
        });
        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::BLACK)?;
        for x in 0..80 {
            for y in 0..60 {
                let cost = self.geography.tile_costs[x][y] as u8;
                window.draw(&Rectangle::new((x as u32 * 10, y as u32 * 10), (10, 10)), Col(match cost {
                    1 => Color::from_rgba(191, 156, 116, 1.0),
                    10 => Color::from_rgba(127, 234, 117, 1.0),
                    _ => Color::BLACK,
                }));
            }
        }
        for human in &self.humans {
            window.draw(&Circle::new(human.location() * 10.0, 5.0), Col(Color::RED));

            // hunger bar
            window.draw(&Rectangle::new((9.0, 569.0), (102.0, 22.0)), Col(Color::BLACK));
            window.draw(&Rectangle::new((10.0, 570.0), (100.0 - human.hunger, 20.0)), Col(Color::RED));

            // fatigue bar
            window.draw(&Rectangle::new((209.0, 569.0), (102.0, 22.0)), Col(Color::BLACK));
            window.draw(&Rectangle::new((210.0, 570.0), (100.0 - human.fatigue, 20.0)), Col(Color::BLUE));
        }
        Ok(())
    }
}

fn main() {
    run::<GameState>("Esytheism", Vector::new(800, 600), Settings::default());
}
