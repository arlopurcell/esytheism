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
                    .map(|c| if c == '.' { 1 } else { u32::MAX })
                    .collect()
            })
            .collect();

        let mut tile_costs = vec![vec![1; 60]; 80];

        for i in 0..30 {
            for j in 0..40 {
                tile_costs[2*j][2*i] = raw_data[i][j];
                tile_costs[2*j+1][2*i] = raw_data[i][j];
                tile_costs[2*j][2*i+1] = raw_data[i][j];
                tile_costs[2*j+1][2*i+1] = raw_data[i][j];
            }
        }

        let geo = Geography::new(tile_costs.clone(), 80, 60);

        let mut human = Human::new(Vector::new(2.5, 57.5));
        human.set_goal(Vector::new(32.5, 50.5), &geo);

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
        self.humans.iter_mut().for_each(|human| human.update());
        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::BLUE)?;
        for x in 0..80 {
            for y in 0..60 {
                let cost = self.geography.tile_costs[x][y] as u8;
                window.draw(&Rectangle::new((x as u32 * 10, y as u32 * 10), (10, 10)), Col(Color::from_rgba(cost, cost, cost, 1.0)));
            }
        }
        for human in &self.humans {
            window.draw(&Circle::new(human.location * 10.0, 5.0), Col(Color::RED));
        }
        Ok(())
    }
}

fn main() {
    run::<GameState>("Esytheism", Vector::new(800, 600), Settings::default());
}
