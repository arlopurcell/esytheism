use rand::prelude::*;
use rand::distributions::{Normal, Distribution};

pub struct Weather {
    current: f64,
    sun: u32,
    rain: u32,
}

impl Weather {
    pub fn new() -> Weather {
        Weather {
            current: 0.0,
            sun: 0,
            rain: 0,
        }
    }

    pub fn update(&mut self) {
        let mut rng = thread_rng();
        let normal = Normal::new(0.0, 10.0);
        self.current = ((self.current * 2.0) + normal.sample(&mut rng)) / 3.0;
        if self.current > 0.0 {
            self.sun = (self.current / 10.0) as u32;
            self.rain = 0;
        } else {
            self.rain = (self.current / -10.0) as u32;
            self.sun = 0;
        }
    }

    pub fn sun(&self) -> u32 {
        self.sun
    }

    pub fn rain(&self) -> u32 {
        self.rain
    }
}
