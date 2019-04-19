extern crate quicksilver;
extern crate rand;
extern crate rayon;

mod geography;
//mod human;

use rand::distributions::{Distribution, Uniform, Normal};
use rayon::prelude::*;

use quicksilver::{
    Result,
    geom::{Rectangle, Vector},
    graphics::{Background::Col, Color},
    input::{ButtonState, Key},
    lifecycle::{Event, Settings, State, Window, run},
};

use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};

use std::u32;

use geography::{Geography, TilePoint};

/*
struct GameState {
    geography: geography::Geography,
    humans: Vec<human::Human>,
}

impl State for GameState {
    fn new() -> Result<GameState> {
        Ok(GameState {
        })
    }

    fn event(&mut self, event: &Event, window: &mut Window) -> Result<()> {
            Ok(())
    }

    fn update(&mut self, _window: &mut Window) -> Result<()> {
        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        Ok(())
    }
}

fn main() {
    run::<GameState>("Esytheism", Vector::new(800, 600), Settings::default());
}
*/

fn main() {
    let file = File::open("data/test.map").unwrap();
    let reader = BufReader::new(file);

    let mut tile_costs: Vec<Vec<u32>> = reader.lines().filter_map(|line| line.ok()).map(|line| line.chars().map(|c| if c == '.' { 1 } else { u32::MAX }).collect()).collect();

    // Transpose
    for y in 0..40 {
        for x in 0..y {
            let temp = tile_costs[x][y];
            tile_costs[x][y] = tile_costs[y][x];
            tile_costs[y][x] = temp;
        }
    }

    let geo = Geography::new(tile_costs.clone(), 40, 40);
    let path = geo.find_path(
        TilePoint::new(1, 39),
        TilePoint::new(16, 35)
    ).unwrap();

    for y in 0..40 {
        for x in 0..40 {
            print!("{}", if path.iter().any(|t| *t == TilePoint::new(x, y)) { "x" } else if tile_costs[x][y] == 1 { "." } else { "+" })
        }
        println!();
    }
}