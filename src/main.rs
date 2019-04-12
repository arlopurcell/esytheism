extern crate quicksilver;
extern crate rand;
extern crate rayon;

use rand::distributions::{Distribution, Uniform};
use rayon::prelude::*;

use quicksilver::{
    Result,
    geom::{Rectangle, Vector},
    graphics::{Background::Col, Color},
    lifecycle::{Settings, State, Window, run},
};

const WORLD_SIZE: usize = 200;
const TICKS_PER_DAY: i32 = 10;
const DAYS_PER_YEAR: i32 = 100;
const TICKS_PER_YEAR: i32 = TICKS_PER_DAY * DAYS_PER_YEAR;
const MAX_TEMP: i32 = 100;

struct GameState {
    counter: i32,
    temps: WorldComponent<Vec<i32>>,
    target_temps: WorldComponent<Vec<i32>>,
}

struct WorldComponent<T> {
    phase_t: T,
    phase_f: T,
    phase: bool,
}

impl<T> WorldComponent<T> {
    fn phases(&mut self) -> (&T, &mut T) {
        if self.phase {
            (&self.phase_t, &mut self.phase_f)
        } else {
            (&self.phase_f, &mut self.phase_t)
        }
    }

    fn swap(&mut self) {
        self.phase = !self.phase
    }
}

impl GameState {
    fn tick(&mut self) {
        {
            let (temps, next_temps) = self.temps.phases();
            let (target_temps, next_target_temps) = self.target_temps.phases();

            if self.counter % TICKS_PER_DAY == 0 {
                let equator = ((((TICKS_PER_YEAR / 2) - self.counter % TICKS_PER_YEAR).abs() - (TICKS_PER_YEAR / 4)) * (WORLD_SIZE as i32 / 3) / TICKS_PER_YEAR) + (WORLD_SIZE as i32 / 2);
                next_target_temps.par_iter_mut().enumerate().for_each(|(n, next_target_temp)| {
                    let y = n / WORLD_SIZE;
                    let distance_from_equator = (equator - y as i32).abs();
                    *next_target_temp = WORLD_SIZE as i32 - 2 * distance_from_equator;
                });


                let temp_sampler = Uniform::from(-10..10);
                next_temps.par_iter_mut().enumerate().for_each(|(n, next_temp)| {
                    let mut rng = rand::thread_rng();
                    let i = n / WORLD_SIZE;
                    let j = n % WORLD_SIZE;

                    let mut divisor = 1;
                    *next_temp = 0;
                    *next_temp += temps[i * WORLD_SIZE + j];

                    if i != (WORLD_SIZE - 1) {
                        *next_temp += temps[(i + 1) * WORLD_SIZE + j];
                        divisor += 1;
                    }
                    if i != 0 {
                        *next_temp += temps[(i - 1) * WORLD_SIZE + j];
                        divisor += 1;
                    }

                    if j != (WORLD_SIZE - 1) {
                        *next_temp += temps[i * WORLD_SIZE + j + 1];
                        divisor += 1;
                        if i != (WORLD_SIZE - 1) {
                            *next_temp += temps[(i + 1) * WORLD_SIZE + j + 1];
                            divisor += 1;
                        }
                        if i != 0 {
                            *next_temp += temps[(i - 1) * WORLD_SIZE + j + 1];
                            divisor += 1;
                        }
                    }

                    if j != 0 {
                        *next_temp += temps[i * WORLD_SIZE + j - 1];
                        divisor += 1;
                        if i != (WORLD_SIZE - 1) {
                            *next_temp += temps[(i + 1) * WORLD_SIZE + j - 1];
                            divisor += 1;
                        }
                        if i != 0 {
                            *next_temp += temps[(i - 1) * WORLD_SIZE + j - 1];
                            divisor += 1;
                        }
                    }
                    *next_temp += target_temps[i * WORLD_SIZE + j];
                    *next_temp /= divisor + 1;
                    *next_temp += temp_sampler.sample(&mut rng);
                });
            }
        }

        if self.counter % TICKS_PER_DAY == 0 {
            self.temps.swap();
            self.target_temps.swap();
        }

        self.counter += 1;
    }
}

impl State for GameState {
    fn new() -> Result<GameState> {
        let mut temps = vec![0; WORLD_SIZE * WORLD_SIZE];
        for x in 0..WORLD_SIZE {
            for y in 0..WORLD_SIZE {
                let distance_from_equator = ((WORLD_SIZE as i32 * 2 / 3) - y as i32).abs(); // start in winter
                temps[y * WORLD_SIZE + x] = MAX_TEMP - 2 * distance_from_equator;
            }
        }
        Ok(GameState {
            temps: WorldComponent {
                phase_t: temps.clone(),
                phase_f: temps.clone(),
                phase: true,
            },
            target_temps: WorldComponent {
                phase_t: temps.clone(),
                phase_f: temps,
                phase: true,
            },
            counter: 0,
        })
    }

    fn update(&mut self, _window: &mut Window) -> Result<()> {
        self.tick();
        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::BLACK)?;

        let (temps, _) = self.temps.phases();
        for x in 0..WORLD_SIZE {
            for y in 0..WORLD_SIZE {
                window.draw(
                    &Rectangle::new((x as i32 * 6 + WORLD_SIZE as i32, y as i32 * 6), (6, 6)), 
                    Col(heatmap_value(temps[y * WORLD_SIZE + x], 0, MAX_TEMP, vec![Color::BLUE, Color::CYAN, Color::GREEN, Color::YELLOW, Color::RED])),
                );
            }
        }

        Ok(())
    }
}

fn heatmap_value(value: i32, min: i32, max: i32, colors: Vec<Color>) -> Color {
    if value >= max {
        *colors.last().unwrap()
    } else if value <= min {
        *colors.first().unwrap()
    } else {
        let size = (max - min) as f32;
        let norm = (value as f32 - min as f32) / size * (colors.len() - 1) as f32;
        let fract = norm - norm.floor();
        let idx1 = norm.floor() as usize;
        let idx2 = norm.ceil() as usize;
        let mut color = Color::WHITE;
        color = color.with_red((colors[idx2].r - colors[idx1].r) * fract + colors[idx1].r);
        color = color.with_green((colors[idx2].g - colors[idx1].g) * fract + colors[idx1].g);
        color = color.with_blue((colors[idx2].b - colors[idx1].b) * fract + colors[idx1].b);
        color
    }
}

fn main() {
    run::<GameState>("Esytheism", Vector::new(800, 600), Settings::default());
}
