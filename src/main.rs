mod gamestate;
mod geography;
mod human;
mod item;
mod plant;
mod weather;
mod world;

use rand::distributions::{Distribution, Normal, Uniform};
use rayon::prelude::*;

use quicksilver::{
    geom::{Circle, Rectangle, Vector, Transform},
    graphics::{Background::Col, Background::Img, Color, Font, FontStyle, Image, View},
    input::{ButtonState, Key, MouseButton},
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

use crate::gamestate::GameState;
use crate::geography::{Geography, TilePoint};
use crate::human::{Human, Job, Mind};
use crate::item::{Inventory, Item, ItemMessage};
use crate::plant::Crop;
use crate::weather::Weather;
use crate::world::{Container, Time, World};

pub const SCREEN_SIZE: Vector = Vector {x: 1200.0, y: 900.0};

struct Engine {
    game_state: GameState,
    font: Asset<Font>,
    paused: bool,
    updates_per_tick: u8,
    counter: u8,

    scale: f32, // higher means more zoomed out
    camera: Vector, // represents center of window

    selected: Selected,
}

enum Selected {
    None,
    Human(usize),
    Crop(usize),
    Container(usize),
}

impl Engine {
    fn apply_camera(&self, top_left: Vector, size: Vector) -> Rectangle {
        let camera_top_left = self.camera - (SCREEN_SIZE * self.scale / 2);
        let top_left = (top_left - camera_top_left) / self.scale;
        Rectangle::new(top_left, size / self.scale)
    }
}

impl State for Engine {
    fn new() -> Result<Engine> {
        let font = Asset::new(Font::load("anonymous_pro.ttf"));
        Ok(Engine {
            game_state: GameState::new(),
            font: font,
            paused: false,
            updates_per_tick: 1,
            counter: 0,
            camera: SCREEN_SIZE / 2,
            scale: 1.0,
            selected: Selected::None,
        })
    }

    fn event(&mut self, event: &Event, window: &mut Window) -> Result<()> {
        match *event {
            Event::Key(Key::Space, ButtonState::Pressed) => {
                self.paused = !self.paused;
            },
            Event::Key(Key::Left, ButtonState::Pressed) => {
                if self.updates_per_tick < 64 {
                    self.updates_per_tick *= 2;
                }
            },
            Event::Key(Key::Right, ButtonState::Pressed) => {
                if self.updates_per_tick > 1 {
                    self.updates_per_tick /= 2;
                }
            },
            Event::MouseButton(MouseButton::Left, ButtonState::Pressed) => {
                let camera_top_left = self.camera - (SCREEN_SIZE * self.scale / 2);
                self.camera = window.mouse().pos() * self.scale + camera_top_left;

                let click_tile = TilePoint::from_vector(&(self.camera / 20.0));
                let human_loc = TilePoint::from_vector(&self.game_state.world.humans[0].location);
                // TODO track click based on actual location and size of thing, not tile approximation
                self.selected = 
                    if let Some((index, _)) = self.game_state.world.humans.iter().enumerate().find(|(_, human)| TilePoint::from_vector(&human.location) == click_tile) {
                        Selected::Human(index)
                    } else if let Some((index, _)) = self.game_state.world.crops.iter().enumerate().find(|(_, crop)| TilePoint::from_vector(&crop.location) == click_tile) {
                        Selected::Crop(index)
                    } else if let Some((index, _)) = self.game_state.world.containers.iter().enumerate().find(|(_, container)| TilePoint::from_vector(&container.location) == click_tile) {
                        Selected::Crop(index)
                    } else {
                        Selected::None
                    };
            },
            Event::MouseWheel(moved) => {
                self.scale = self.scale * if moved.y > 0.0 {
                    moved.y / 10.0
                } else {
                    -10.0 / moved.y
                };
            }
            _ => (),
        }
        Ok(())
    }

    fn update(&mut self, _window: &mut Window) -> Result<()> {
        if !self.paused
            && ({
                self.counter = self.counter.wrapping_add(1);
                self.counter
            } % self.updates_per_tick)
                == 0
        {
            self.game_state.update();
        }
        if !self.paused {
            let updates_per_tick = self.updates_per_tick;
            self.game_state.do_travel(updates_per_tick);
        }

        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::BLACK)?;

        // draw terrain
        for x in 0..self.game_state.world.geography.width {
            for y in 0..self.game_state.world.geography.height {
                let tile = &self.game_state.world.geography.tiles[x][y];
                window.draw(
                    &self.apply_camera(Vector::new(x as u32 * 20, y as u32 * 20), Vector::new(20, 20)),
                    Col(match tile.terrain_cost {
                        1 => Color::from_rgba(191, 156, 116, 1.0),
                        _ => Color::from_rgba(127, 234, 117, 1.0),
                    }),
                );
            }
        }
        
        // draw walls
        for x in 0..self.game_state.world.geography.width {
            for y in 0..self.game_state.world.geography.height {
                let tile = &self.game_state.world.geography.tiles[x][y];
                if tile.walls[0] {
                    window.draw(
                        &self.apply_camera(Vector::new(x as i32 * 20 - 1, y as i32 * 20 - 1), Vector::new(22, 2)),
                        Col(Color::BLACK),
                    );
                }
                if tile.walls[1] {
                    window.draw(
                        &self.apply_camera(Vector::new(x as i32 * 20 + 19, y as i32 * 20 - 1), Vector::new(2, 22)),
                        Col(Color::BLACK),
                    );
                }
                if tile.walls[2] {
                    window.draw(
                        &self.apply_camera(Vector::new(x as i32 * 20 - 1, y as i32 * 20 + 19), Vector::new(22, 2)),
                        Col(Color::BLACK),
                    );
                }
                if tile.walls[3] {
                    window.draw(
                        &self.apply_camera(Vector::new(x as i32 * 20 - 1, y as i32 * 20 - 1), Vector::new(2, 22)),
                        Col(Color::BLACK),
                    );
                }
            }
        }

        // draw humans
        for human in &self.game_state.world.humans {
            window.draw(
                &self.apply_camera(human.location * 20 - Vector::new(2, 2), Vector::new(4, 4)),
                Col(Color::RED),
            );
        }

        for mind in &self.game_state.minds {
            self.font.execute(|font| {
                window.draw(&Rectangle::new(((SCREEN_SIZE.x - 400.0) / 2.0, SCREEN_SIZE.y - 50.0), (400, 50)), Col(Color::BLACK));
                let style = FontStyle::new(48.0, Color::WHITE);
                let text = format!("state: {}", mind.state());
                let text_img = font.render(&text, &style).unwrap();
                window.draw(
                    &Rectangle::new(((SCREEN_SIZE.x - 400.0) / 2.0 + 10.0, SCREEN_SIZE.y - 47.0), text_img.area().size()),
                    Img(&text_img),
                );
                Ok(())
            });
        }

        let world_time = &self.game_state.world.time;
        self.font.execute(|font| {
            window.draw(&Rectangle::new((SCREEN_SIZE.x - 230.0, 0), (230, 20)), Col(Color::BLACK));
            let style = FontStyle::new(18.0, Color::WHITE);
            let time_img = font.render(&world_time.date_string(), &style).unwrap();
            window.draw(
                &Rectangle::new((SCREEN_SIZE.x - 228.0, 1), time_img.area().size()),
                Img(&time_img),
            );
            Ok(())
        });

        let lines = match self.selected {
            Selected::Human(index) => {
                let human = &self.game_state.world.humans[index];
                let top_left = human.location * 20 - Vector::new(3, 3);
                let top_right = top_left + Vector::new(4, 0);
                let bottom_left = top_left + Vector::new(0, 4);
                let vert_size = Vector::new(1, 5);
                let horiz_size = Vector::new(5, 1);
                // TODO make border aways 1px, regardless of zoom level
                window.draw(&self.apply_camera(top_left, horiz_size), Col(Color::YELLOW));
                window.draw(&self.apply_camera(top_right, vert_size), Col(Color::YELLOW));
                window.draw(&self.apply_camera(bottom_left, horiz_size), Col(Color::YELLOW));
                window.draw(&self.apply_camera(top_left, vert_size), Col(Color::YELLOW));

                Some(human.description_lines(&self.game_state.world))
            },
            // TODO crop and containers
            _ => None,
        };
        if let Some(lines) = lines {
            let height = (5 + lines.len() * 20) as u32;
            window.draw(
                &Rectangle::new((4, 4), (200, height)),
                Col(Color::from_rgba(0, 0, 0, 0.5)),
            );
            self.font.execute(|font| {
                let style = FontStyle::new(14.0, Color::WHITE);
                for (index, line) in lines.iter().enumerate() {
                    let text_img = font.render(&line, &style).unwrap();
                    window.draw(
                        &Rectangle::new((8, (8 + index * 20) as u32), text_img.area().size()),
                        Img(&text_img),
                    );
                }
                Ok(())
            });
        }
        Ok(())
    }

}

fn main() {
    run::<Engine>("Esytheism", SCREEN_SIZE, Settings::default());
}
