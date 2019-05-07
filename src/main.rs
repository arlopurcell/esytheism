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
use crate::geography::Geography;
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
            },
            Event::MouseWheel(moved) => {
                self.scale = self.scale * if moved.y > 0.0 {
                    moved.y / 10.0
                } else {
                    -10.0 / moved.y
                };
                println!("scale: {}", self.scale);
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
                    // &Rectangle::new((x as u32 * 20, y as u32 * 20), (20, 20)),
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
                        // &Rectangle::new((x as i32 * 20 - 1, y as i32 * 20 - 1), (22, 2)),
                        &self.apply_camera(Vector::new(x as i32 * 20 - 1, y as i32 * 20 - 1), Vector::new(22, 2)),
                        Col(Color::BLACK),
                    );
                }
                if tile.walls[1] {
                    window.draw(
                        // &Rectangle::new((x as i32 * 20 + 19, y as i32 * 20 - 1), (2, 22)),
                        &self.apply_camera(Vector::new(x as i32 * 20 + 19, y as i32 * 20 - 1), Vector::new(2, 22)),
                        Col(Color::BLACK),
                    );
                }
                if tile.walls[2] {
                    window.draw(
                        // &Rectangle::new((x as i32 * 20 - 1, y as i32 * 20 + 19), (22, 2)),
                        &self.apply_camera(Vector::new(x as i32 * 20 - 1, y as i32 * 20 + 19), Vector::new(22, 2)),
                        Col(Color::BLACK),
                    );
                }
                if tile.walls[3] {
                    window.draw(
                        // &Rectangle::new((x as i32 * 20 - 1, y as i32 * 20 - 1), (2, 22)),
                        &self.apply_camera(Vector::new(x as i32 * 20 - 1, y as i32 * 20 - 1), Vector::new(2, 22)),
                        Col(Color::BLACK),
                    );
                }
            }
        }

        // draw humans
        for human in &self.game_state.world.humans {
            window.draw(
                // &Circle::new(human.location * 20.0, 3.0), 
                &self.apply_camera(human.location * 20 - Vector::new(5, 5), Vector::new(10, 10)),
                Col(Color::RED),
            );
            // self.font.execute(|font| {
            //     window.draw(&Rectangle::new((200, 550), (400, 40)), Col(Color::BLACK));
            //     let style = FontStyle::new(36.0, Color::WHITE);
            //     let text = format!("hunger: {:.2}", human.hunger);
            //     let text_img = font.render(&text, &style).unwrap();
            //     window.draw(&Rectangle::new((210, 553), text_img.area().size()), Img(&text_img));
            //     Ok(())
            // });
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
        Ok(())
    }

}

fn main() {
    run::<Engine>("Esytheism", SCREEN_SIZE, Settings::default());
}
