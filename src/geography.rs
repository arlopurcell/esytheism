use num_traits::ops::saturating::Saturating;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::u32;
use quicksilver::geom::Vector;

pub struct Geography {
    pub tiles: Vec<Vec<Tile>>,
    pub width: usize,
    pub height: usize,
}

pub struct Tile {
    pub terrain_cost: u16,
    pub walls: [bool; 4], // css/clockwise order: top, right, bottom, left
}

impl Tile {
    fn is_wall_to(&self, position_index: usize) -> bool {
        self.walls[position_index]
    }

    fn is_wall_from(&self, position_index: usize) -> bool {
        self.walls[(position_index + 2) % 4]
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct TilePoint {
    pub x: usize,
    pub y: usize,
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct PathState {
    cost: u32,
    position: TilePoint,
}

impl Ord for PathState {
    fn cmp(&self, other: &PathState) -> Ordering {
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.position.x.cmp(&other.position.x))
            .then_with(|| self.position.y.cmp(&other.position.y))
    }
}

impl PartialOrd for PathState {
    fn partial_cmp(&self, other: &PathState) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Geography {
    pub fn from_data(width: usize, height: usize, data: &[u8]) -> Geography {
        let mut tiles = Vec::new();
        for x in 0..width {
            let mut col = Vec::new();
            for y in 0..height {
                col.push(Tile {
                    // width + 1 everywhere to account for newlines
                    terrain_cost: match data[(width*2+2)*(2*y + 1) + (x*2 + 1)] as char {
                        '+' => 1u16, // road
                        _ => 5u16, // anything else
                    },
                    walls: [
                        data[(width*2 + 2)*2*y + (x*2 + 1)] == '-' as u8,
                        data[(width*2 + 2)*(2*y+1) + (x*2 + 2)] == '|' as u8,
                        data[(width*2 + 2)*(2*y+2) + (x*2 + 1)] == '-' as u8,
                        data[(width*2 + 2)*(2*y+1) + x*2] == '|' as u8,
                    ],
                })
            }
            tiles.push(col);
        }

        Geography {
            tiles: tiles,
            width: width,
            height: height,
        }
    }

    pub fn find_path(&self, start: TilePoint, goal: TilePoint) -> Option<Vec<TilePoint>> {
        let mut closed_set = HashSet::new();

        let mut came_from = HashMap::new();

        let mut g_score = HashMap::new();
        g_score.insert(start, 0);

        let goal_distance_squared = |a: &TilePoint| {
            let x = a.x as i32 - goal.x as i32;
            let y = a.y as i32 - goal.y as i32;
            (x * x + y * y) as u32
        };

        let mut open_set = BinaryHeap::new();
        open_set.push(PathState {
            cost: goal_distance_squared(&start),
            position: start,
        });

        while let Some(current) = open_set.pop() {
            if current.position == goal {
                return Some(reconstruct_path(came_from, &current.position));
            }
            closed_set.insert(current.position);

            let neighbors = self.get_neighbors(&current.position);
            for position_index in 0..4 {
                if let Some(neighbor) = neighbors[position_index] {
                    if !closed_set.contains(&neighbor) {
                        let tentative_g_score = g_score
                            .get(&current.position)
                            .unwrap()
                            .saturating_add(self.get_cost(&current.position, &neighbor, position_index));
                        let old_g_score = g_score.get(&neighbor).unwrap_or(&u32::MAX);
                        if tentative_g_score < *old_g_score {
                            open_set.push(PathState {
                                cost: tentative_g_score + goal_distance_squared(&neighbor),
                                position: neighbor,
                            });
                            came_from.insert(neighbor, current.position);
                            g_score.insert(neighbor, tentative_g_score);
                        }
                    }
                }
            }
        }

        None
    }

    fn get_neighbors(&self, point: &TilePoint) -> [Option<TilePoint>; 4] {
        let mut neighbors = [None; 4];
        if point.x > 0 {
            neighbors[3] = Some(TilePoint::new(point.x - 1, point.y));
        }
        if point.x < self.width - 1 {
            neighbors[1] = Some(TilePoint::new(point.x + 1, point.y));
        }

        if point.y > 0 {
            neighbors[0] = Some(TilePoint::new(point.x, point.y - 1));
        }
        if point.y < self.height - 1 {
            neighbors[2] = Some(TilePoint::new(point.x, point.y + 1));
        }
        neighbors
    }

    fn get_cost(&self, current_point: &TilePoint, neighbor_point: &TilePoint, position_index: usize) -> u32 {
        let current_tile = &self.tiles[current_point.x][current_point.y];
        let neighbor_tile = &self.tiles[neighbor_point.x][neighbor_point.y];
        if current_tile.is_wall_to(position_index) || neighbor_tile.is_wall_from(position_index) {
            u32::MAX
        } else {
            current_tile.terrain_cost as u32 + neighbor_tile.terrain_cost as u32
        }
    }
}

fn reconstruct_path(came_from: HashMap<TilePoint, TilePoint>, goal: &TilePoint) -> Vec<TilePoint> {
    let mut result = vec![*goal];
    let mut current = goal;
    while let Some(prev) = came_from.get(&current) {
        current = prev;
        result.push(*current);
    }
    result
}

impl TilePoint {
    pub fn new(x: usize, y: usize) -> TilePoint {
        TilePoint { x: x, y: y }
    }

    pub fn from_vector(v: &Vector) -> TilePoint {
        TilePoint { x: v.x.floor() as usize, y: v.y.floor() as usize }
    }
}
