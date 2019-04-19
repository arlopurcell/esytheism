use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, BinaryHeap};
use std::u32;
use num_traits::ops::saturating::Saturating;

pub struct Geography {
    tile_costs: Vec<Vec<u32>>,
    width: usize,
    height: usize,
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
        other.cost.cmp(&self.cost)
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

    pub fn new(tile_costs: Vec<Vec<u32>>, width: usize, height: usize) -> Geography {
        Geography {
            tile_costs: tile_costs,
            height: height,
            width: width,
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
        //let mut fScore = HashMap::new();
        //fScore.insert(start, goal_distance_squared(&start));

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

            for neighbor in self.get_neighbors(&current.position) {
                if !closed_set.contains(&neighbor) {
                    let tentative_g_score = g_score.get(&current.position).unwrap().saturating_add(self.get_cost(&neighbor));
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
        
        None
    }
    
    fn get_neighbors(&self, point: &TilePoint) -> Vec<TilePoint> {
        let mut neighbors = Vec::new();
        if point.x > 0 {
            neighbors.push(TilePoint::new(point.x - 1, point.y));
        }
        if point.x < self.width - 1 {
            neighbors.push(TilePoint::new(point.x + 1, point.y));
        }

        if point.y > 0 {
            neighbors.push(TilePoint::new(point.x, point.y - 1));
        }
        if point.y < self.height - 1 {
            neighbors.push(TilePoint::new(point.x, point.y + 1));
        }
        neighbors
    }

    fn get_cost(&self, point: &TilePoint) -> u32 {
        self.tile_costs[point.x][point.y]
    }
}

fn reconstruct_path(came_from: HashMap<TilePoint, TilePoint>, goal: &TilePoint) -> Vec<TilePoint> {
    let mut result = vec![*goal];
    let mut current = goal;
    while let Some(prev) = came_from.get(&current) {
        current = prev;
        result.push(*current);
    }
    result.into_iter().rev().collect()
}

impl TilePoint {
    pub fn new(x: usize, y: usize) -> TilePoint {
        TilePoint {
            x: x,
            y: y,
        }
    }
}

