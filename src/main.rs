extern crate quicksilver;
extern crate rand;

use rand::distributions::{Distribution, Uniform};

use quicksilver::{
    Result,
    geom::{Circle, Line, Rectangle, Transform, Triangle, Vector},
    graphics::{Background::Col, Color},
    lifecycle::{Settings, State, Window, run},
};

struct GameState {
    worlds: [World; 2],
    counter: usize,
}

struct World {
    temps: Vec<i32>,
    target_temps: Vec<i32>,
}

/*
struct WorldComponent<T> {
    phases: [T; 2],
    phase: bool,
}

impl<T> WorldComponent<T> {
    fn current<T>(&self) -> &T {
        &self.phases[0 ? self.phase : 1]
    }

    fn next<T>(&mut self) -> &mut T {
        &mut self.phases[1 ? self.phase : 0]
    }

    fn swap(&mut self) {
        self.phase = !self.phase
    }
}
*/

impl GameState {
    fn current_world(&self) -> &World {
        &self.worlds[self.counter % 2]
    }

    fn next_world(&mut self) -> &mut World {
        &mut self.worlds[(self.counter + 1) % 2]
    }

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
                //temps.push(100 -(2 * (50 - y as i8).abs()));
            }
        }
        Ok(GameState {
            worlds: [
                World {
                    temps: temps.clone(),
                    target_temps: temps.clone(),
                },
                World {
                    temps: temps.clone(),
                    target_temps: temps,
                },
            ],
            counter: 0,
        })
    }

    fn update(&mut self, window: &mut Window) -> Result<()> {
        //let current = &self.worlds[self.current];
        //let mut next = &mut self.worlds[(self.current + 1) % 2];
        // TODO update target temps based on time of year

        if self.counter % 11 == 0 {
            let mut next_temp;
            let mut divisor;

            let mut rng = rand::thread_rng();
            let temp_sampler = Uniform::from(-10..10);
            for i in 0..100 {
                for j in 0..100 {
                    next_temp = self.current_world().temps[i * 100 + j];
                    divisor = 1;

                    if i != 99 {
                        next_temp += self.current_world().temps[(i + 1) * 100 + j];
                        divisor += 1;
                    }
                    if i != 0 {
                        next_temp += self.current_world().temps[(i - 1) * 100 + j];
                        divisor += 1;
                    }

                    if j != 99 {
                        next_temp += self.current_world().temps[i * 100 + j + 1];
                        divisor += 1;
                        if i != 99 {
                            next_temp += self.current_world().temps[(i + 1) * 100 + j + 1];
                            divisor += 1;
                        }
                        if i != 0 {
                            next_temp += self.current_world().temps[(i - 1) * 100 + j + 1];
                            divisor += 1;
                        }
                    }

                    if j != 0 {
                        next_temp += self.current_world().temps[i * 100 + j - 1];
                        divisor += 1;
                        if i != 99 {
                            next_temp += self.current_world().temps[(i + 1) * 100 + j - 1];
                            divisor += 1;
                        }
                        if i != 0 {
                            next_temp += self.current_world().temps[(i - 1) * 100 + j - 1];
                            divisor += 1;
                        }
                    }
                    next_temp += self.current_world().target_temps[i * 100 + j];
                    next_temp /= divisor + 1;
                    next_temp += temp_sampler.sample(&mut rng);

                    self.next_world().temps[i * 100 + j] = next_temp;
                }
            }
        }

        //self.current = (self.current + 1) % 2;
        self.increment();
        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::BLACK)?;

        for x in 0..100 {
            for y in 0..100 {
                let temp = self.current_world().temps[x * 100 + y];
                window.draw(
                    &Rectangle::new((x as u32 * 6 + 100, y as u32 * 6), (6, 6)), 
                    Col(heatmap_value(temp, 0, 100, vec![Color::BLUE, Color::CYAN, Color::GREEN, Color::YELLOW, Color::RED])),
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
