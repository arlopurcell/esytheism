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
    fn increment(&mut self) {
        self.counter += 1;
    }
}

impl State for GameState {
    fn new() -> Result<GameState> {
        let mut temps = vec![0; 100 * 100];
        for x in 0..100 {
            for y in 0..100 {
                let distance_from_equator = (50 - y as i32).abs();
                temps[x * 100 + y] = 100 - 2 * distance_from_equator;
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
        {
            let (temps, next_temps) = self.temps.phases();
            let (target_temps, next_target_temps) = self.target_temps.phases();
            // TODO update target temps based on time of year

            if self.counter % 10 == 0 {
                let equator = ((500 - (self.counter % 1000)).abs() - 250) / 30 + 50;
                next_target_temps.par_iter_mut().enumerate().for_each(|(n, next_target_temp)| {
                    let y = n % 100;
                    let distance_from_equator = (equator - y as i32).abs();
                    *next_target_temp = 100 - 2 * distance_from_equator;
                });


                let temp_sampler = Uniform::from(-10..10);
                next_temps.par_iter_mut().enumerate().for_each(|(n, next_temp)| {
                    let mut rng = rand::thread_rng();
                    let i = n / 100;
                    let j = n % 100;

                    let mut divisor = 1;
                    *next_temp = 0;
                    *next_temp += temps[i * 100 + j];

                    if i != 99 {
                        *next_temp += temps[(i + 1) * 100 + j];
                        divisor += 1;
                    }
                    if i != 0 {
                        *next_temp += temps[(i - 1) * 100 + j];
                        divisor += 1;
                    }

                    if j != 99 {
                        *next_temp += temps[i * 100 + j + 1];
                        divisor += 1;
                        if i != 99 {
                            *next_temp += temps[(i + 1) * 100 + j + 1];
                            divisor += 1;
                        }
                        if i != 0 {
                            *next_temp += temps[(i - 1) * 100 + j + 1];
                            divisor += 1;
                        }
                    }

                    if j != 0 {
                        *next_temp += temps[i * 100 + j - 1];
                        divisor += 1;
                        if i != 99 {
                            *next_temp += temps[(i + 1) * 100 + j - 1];
                            divisor += 1;
                        }
                        if i != 0 {
                            *next_temp += temps[(i - 1) * 100 + j - 1];
                            divisor += 1;
                        }
                    }
                    *next_temp += target_temps[i * 100 + j];
                    *next_temp /= divisor + 1;
                    *next_temp += temp_sampler.sample(&mut rng);
                });
            }
        }

        if self.counter % 10 == 0 {
            self.temps.swap();
            self.target_temps.swap();
        }

        self.increment();
        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::BLACK)?;

        let (temps, _) = self.temps.phases();
        for x in 0..100 {
            for y in 0..100 {
                window.draw(
                    &Rectangle::new((x as u32 * 6 + 100, y as u32 * 6), (6, 6)), 
                    Col(heatmap_value(temps[x * 100 + y], 0, 100, vec![Color::BLUE, Color::CYAN, Color::GREEN, Color::YELLOW, Color::RED])),
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
