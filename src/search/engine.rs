use std::cmp::*;
use std::collections::{BTreeSet, HashMap};
use super::super::map::*;
use super::super::heurs::*;

struct SearchResult {
	depth: usize,
	score: GameMeasure,
}

struct Engine {
	map: GameMap,
	comp: HashMap<Hash, SearchResult>,
	turn: usize,
}

impl Engine {
	pub fn new() -> Engine {
		let mut eng = Engine {
			map: GameMap::new(),
			comp: HashMap::new(),
			turn: 0,
		};
		eng.map.do_move(0, 0, Tile::X);
		eng
	}

	fn move_candidates(&self, max_dist: i64) -> Vec<(i64, i64)> {
		let mut used = self.map.get_move_list();
		used.sort_by(|a, b| { a.0.cmp(&b.0) });

		let mut x = i64::MIN;

		let mut start = 0;
		let mut next = 0;
		let mut stop = 0;

		let mut candidates = Vec::new();
		loop {
			while used[start].0 + max_dist < x && start < used.len() { start += 1; }
			if start == used.len() { break; }
			if x < used[start].0 - max_dist {
				next = start;
				x = used[start].0 + max_dist;
			}
			while x - used[stop].0 <= max_dist && stop < used.len() {
				stop += 1;
			}
			while used[next].0 < x { next += 1; }
			let mut intervals = &used[start..stop].iter().map(|&val| (
				max(val.1 - max_dist, val.1 + x - val.0 - max_dist),
				min(val.1 + max_dist, val.1 + x - val.0 + max_dist),
			)).collect::<Vec<_>>();
			intervals.sort_by(|a, b| { a.1.cmp(&b.1) });

			let mut y = i64::MIN;
			for (y_min, y_max) in intervals.into_iter() {
				y = max(y, *y_min);
				while y < *y_max {
					if used[next] == (x, y) { next += 1; }
					else { candidates.push((x, y)); }
					y += 1;
				}
			}

			x += 1;
			candidates.push((x, y));
		}
		candidates
	}

	fn alpha_beta(&mut self, max_depth: usize, alpha: &GameMeasure, beta: &GameMeasure) -> (GameMeasure, GameMeasure) {
	}

	pub fn run_search(&mut self, max_depth: usize) {

	}
}