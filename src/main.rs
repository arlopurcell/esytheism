extern crate quicksilver;
extern crate rand;
extern crate rayon;

use rand::distributions::{Distribution, Uniform, Normal};
use rayon::prelude::*;

use quicksilver::{
    Result,
    geom::{Rectangle, Vector},
    graphics::{Background::Col, Color},
    input::{ButtonState, Key},
    lifecycle::{Event, Settings, State, Window, run},
};

const WORLD_SIZE: usize = 100;
const TICKS_PER_DAY: i32 = 10;
const DAYS_PER_YEAR: i32 = 100;
const TICKS_PER_YEAR: i32 = TICKS_PER_DAY * DAYS_PER_YEAR;
const MAX_TEMP: i32 = 100;
const MAX_CLOUD: i32 = 100;

struct GameState {
    world: World,
    view: ViewType,
}

enum ViewType {
    Temp,
    Cloud,
}

struct World {
    counter: i32,

    temps: WorldComponent<Vec<i32>>,
    target_temps: WorldComponent<Vec<i32>>,

    cloud_threshold: i32,
    rain_threshold: i32,
    clouds: WorldComponent<Vec<i32>>,
    target_clouds: WorldComponent<Vec<i32>>,
    // seasonal_cloud_phases: Vec<i32>,
    // seasonal_cloud_amplitude: Vec<i32>,
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

impl World {
    fn tick(&mut self) {
        {
            let (temps, next_temps) = self.temps.phases();
            let (target_temps, next_target_temps) = self.target_temps.phases();
            let (clouds, next_clouds) = self.clouds.phases();
            let (target_clouds, _next_target_clouds) = self.target_clouds.phases();

            let cloud_threshold = &self.cloud_threshold;
            let rain_threshold = &self.rain_threshold;

            if self.counter % TICKS_PER_DAY == 0 {
                // Update target temps
                let equator = ((((TICKS_PER_YEAR / 2) - self.counter % TICKS_PER_YEAR).abs() - (TICKS_PER_YEAR / 4)) * (WORLD_SIZE as i32 / 3) / TICKS_PER_YEAR) + (WORLD_SIZE as i32 / 2);
                next_target_temps.par_iter_mut().enumerate().for_each(|(n, next_target_temp)| {
                    let y = n / WORLD_SIZE;
                    let distance_from_equator = (equator - y as i32).abs();
                    *next_target_temp = WORLD_SIZE as i32 - 2 * distance_from_equator;
                });

                // Update temps
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

                    if clouds[n] > *cloud_threshold {
                        // up to 10 degrees cooler in the shade
                        *next_temp -= (clouds[n].min(*rain_threshold) - *cloud_threshold) / 2;
                    }
                });
                
                // Update clouds
                let cloud_sampler = Normal::new(0.0, 15.0);
                next_clouds.par_iter_mut().enumerate().for_each(|(n, next_cloud)| {
                    let mut rng = rand::thread_rng();
                    let y = n / WORLD_SIZE;
                    let x = n % WORLD_SIZE;

                    let mut divisor = 1;
                    *next_cloud = 0;
                    *next_cloud += clouds[y * WORLD_SIZE + x];

                    if y != (WORLD_SIZE - 1) {
                        *next_cloud += clouds[(y + 1) * WORLD_SIZE + x];
                        divisor += 1;
                    }
                    if y != 0 {
                        *next_cloud += clouds[(y - 1) * WORLD_SIZE + x];
                        divisor += 1;
                    }

                    if x != (WORLD_SIZE - 1) {
                        *next_cloud += clouds[y * WORLD_SIZE + x + 1];
                        divisor += 1;
                        if y != (WORLD_SIZE - 1) {
                            *next_cloud += clouds[(y + 1) * WORLD_SIZE + x + 1];
                            divisor += 1;
                        }
                        if y != 0 {
                            *next_cloud += clouds[(y - 1) * WORLD_SIZE + x + 1];
                            divisor += 1;
                        }
                    }

                    if x != 0 {
                        *next_cloud += clouds[y * WORLD_SIZE + x - 1];
                        divisor += 1;
                        if y != (WORLD_SIZE - 1) {
                            *next_cloud += clouds[(y + 1) * WORLD_SIZE + x - 1];
                            divisor += 1;
                        }
                        if y != 0 {
                            *next_cloud += clouds[(y - 1) * WORLD_SIZE + x - 1];
                            divisor += 1;
                        }
                    }
                    *next_cloud *= 5;
                    divisor *= 5;

                    *next_cloud += target_clouds[y * WORLD_SIZE + x];
                    *next_cloud /= divisor + 1;
                    *next_cloud += cloud_sampler.sample(&mut rng) as i32 * 2;
                });
            }
        }

        if self.counter % TICKS_PER_DAY == 0 {
            self.temps.swap();
            self.target_temps.swap();

            self.clouds.swap();
            self.target_clouds.swap();
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
            view: ViewType::Temp,
            world: World {
                counter: 0,

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

                cloud_threshold: 50,
                rain_threshold: 80,
                clouds: WorldComponent {
                    phase_t: vec![20; WORLD_SIZE * WORLD_SIZE],
                    phase_f: vec![20; WORLD_SIZE * WORLD_SIZE],
                    phase: true,
                },
                target_clouds: WorldComponent {
                    phase_t: vec![20; WORLD_SIZE * WORLD_SIZE],
                    phase_f: vec![20; WORLD_SIZE * WORLD_SIZE],
                    phase: true,
                },
            },
        })
    }

    fn event(&mut self, event: &Event, window: &mut Window) -> Result<()> {
            match *event {
                Event::Key(Key::T, ButtonState::Pressed) => {
                    self.view = ViewType::Temp;
                }
                Event::Key(Key::C, ButtonState::Pressed) => {
                    self.view = ViewType::Cloud;
                }
                Event::Key(Key::Escape, ButtonState::Pressed) => {
                    window.close();
                }
                _ => (),
            }
            Ok(())
    }

    fn update(&mut self, _window: &mut Window) -> Result<()> {
        self.world.tick();
        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        match self.view {
            ViewType::Temp => {
                window.clear(Color::BLACK)?;

                let (temps, _) = self.world.temps.phases();
                for x in 0..WORLD_SIZE {
                    for y in 0..WORLD_SIZE {
                        window.draw(
                            &Rectangle::new((x as i32 * 6 + WORLD_SIZE as i32, y as i32 * 6), (6, 6)), 
                            Col(heatmap_value(temps[y * WORLD_SIZE + x], 0, MAX_TEMP, vec![Color::BLUE, Color::CYAN, Color::GREEN, Color::YELLOW, Color::RED])),
                        );
                    }
                }

                Ok(())
            },
            ViewType::Cloud => {
                window.clear(Color::BLACK)?;

                let (clouds, _) = self.world.clouds.phases();
                for x in 0..WORLD_SIZE {
                    for y in 0..WORLD_SIZE {
                        window.draw(
                            &Rectangle::new((x as i32 * 6 + WORLD_SIZE as i32, y as i32 * 6), (6, 6)), 
                            Col(heatmap_value(clouds[y * WORLD_SIZE + x], 0, MAX_TEMP, vec![Color::BLACK, Color::BLACK, Color::WHITE, Color::BLUE])),
                        );
                    }
                }

                Ok(())
            },
        }
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
