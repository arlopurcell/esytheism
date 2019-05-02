use quicksilver::geom::Vector;

use crate::geography::Geography;
use crate::human::Human;
use crate::item::Inventory;
use crate::weather::Weather;
use crate::plant::Crop;

pub const TICKS_PER_MINUTE: u8 = 3;

pub struct World {
    pub geography: Geography,
    pub humans: Vec<Human>,
    pub containers: Vec<Container>,
    pub time: Time,
    pub weather: Weather,
    pub crops: Vec<Crop>,
}

pub struct Container {
    pub location: Vector,
    pub inventory: Inventory,
}

pub struct Time {
    pub tick: u8,
    pub minute: u8,
    pub hour: u8,
    pub day: u8,
    pub weekday: u8,
    pub month: u8,
    pub year: u16,
}

impl Time {
    pub fn new() -> Time {
        Time {
            tick: 0,
            minute: 0,
            hour: 0,
            day: 0,
            weekday: 0,
            month: 0,
            year: 0,
        }
    }

    pub fn tick(&mut self) {
        self.tick += 1;
        if self.tick % TICKS_PER_MINUTE == 0 {
            self.tick = 0;
            self.minute += 1;
            if self.minute % 60 == 0 {
                self.minute = 0;
                self.hour += 1;
                if self.hour % 24 == 0 {
                    self.hour = 0;
                    self.day += 1;
                    if self.day % 30 == 0 { // all months have 30 days
                        self.day = 0;
                        self.month += 1;
                        if self.month % 12 == 0 {
                            self.month = 0;
                            self.year += 1;
                        }
                    }
                    self.weekday = (self.weekday + 1) % 7;
                }
            }
        }

    }

    pub fn is_new_day(&self) -> bool {
        self.tick == 0 && self.hour == 0
    }

    pub fn date_string(&self) -> String {
        format!(
            "{weekday}, {month} {day:02}, {year:04} {hour:02}:{minute:02}",
            weekday=match &self.weekday {
                0 => "Mon",
                1 => "Tue",
                2 => "Wed",
                3 => "Thu",
                4 => "Fri",
                5 => "Sat",
                _ => "Sun",
            },
            month=match &self.month {
                0 => "Jan",
                1 => "Feb",
                2 => "Mar",
                3 => "Apr",
                4 => "May",
                5 => "Jun",
                6 => "Jul",
                7 => "Aug",
                8 => "Sep",
                9 => "Oct",
                10 => "Nov",
                _ => "Dec",
            },
            day=self.day + 1,
            year=self.year + 1,
            hour=self.hour,
            minute=self.minute,
        )
    }
}
