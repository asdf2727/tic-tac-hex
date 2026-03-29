use std::collections::HashMap;
use super::super::map::*;
use super::super::heurs::*;

type Heuristic = GameThreats;

struct SearchResult {
    depth: usize,
    score: Heuristic
}

struct Engine {
    map: GameMap,
    comp: HashMap<Hash, SearchResult>
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            map: GameMap::new::<Heuristic>(),
            comp: HashMap::new(),
        }
    }
}