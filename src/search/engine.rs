use std::io::Write;
use super::super::map::*;
use std::cmp::*;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::io::stdout;

struct SearchResult {
	score: GameMeasure,
	depth: usize,
}

pub struct Engine {
	pub map: GameMap,
	comp: HashMap<Hash, SearchResult>,
	search_cnt: usize,
}

impl Engine {
	pub fn new() -> Engine {
		let mut eng = Engine {
			map: GameMap::new(),
			comp: HashMap::new(),
			search_cnt: 0,
		};
		eng.comp.insert(eng.map.get_hash(), SearchResult {
			score: *eng.map.get_heuristic(),
			depth: 0,
		});
		eng
	}

	fn move_candidates(&self, max_dist: i64) -> Vec<(i64, i64)> {
		let mut used = self.map.get_move_list().clone();
		used.push((0, 0));
		used.sort_by(|a, b| { a.0.cmp(&b.0).then(a.1.cmp(&b.1)) });
		let used = used;

		let mut x = i64::MIN;

		let mut start = 0;
		let mut next = 0;
		let mut stop = 0;

		let mut candidates = Vec::new();
		loop {
			while start < used.len() && used[start].0 + max_dist < x { start += 1; }
			if start == used.len() { break; }
			if x < used[start].0 - max_dist {
				next = start;
				x = used[start].0 - max_dist;
			}
			while stop < used.len() && x >= used[stop].0 - max_dist {
				stop += 1;
			}
			while next < used.len() && used[next].0 < x { next += 1; }
			let mut intervals: Vec<(i64, i64)> = used[start..stop].iter().map(|val| (
				max(val.1 - max_dist, val.1 + x - val.0 - max_dist),
				min(val.1 + max_dist, val.1 + x - val.0 + max_dist),
			)).collect();
			intervals.sort_by(|a, b| { a.0.cmp(&b.0) });

			let mut y = i64::MIN;
			for (y_min, y_max) in intervals.into_iter() {
				y = max(y, y_min);
				while y <= y_max {
					if next < used.len() && used[next] == (x, y) { next += 1; }
					else { candidates.push((x, y)); }
					y += 1;
				}
			}

			x += 1;
			candidates.push((x, y));
		}
		candidates
	}

	fn max_player(depth: usize) -> bool { depth & 2 != 0 }

	pub fn sort_moves(&mut self) -> Vec<((i64, i64), GameMeasure)> {
		let depth = self.map.get_move_list().len();
		let player_tile = if Engine::max_player(depth) { Tile::X } else { Tile::O };
		let mut moves = Vec::new();

		for (x, y) in self.move_candidates(1) {
			let score = match self.comp.entry(self.map.peek_hash(x, y, player_tile)) {
				Entry::Occupied(entry) => entry.get().score,
				Entry::Vacant(_) => {
					self.map.place(x, y, player_tile);
					let comp = *self.map.get_heuristic();
					self.map.undo();
					comp
				},
			};
			moves.push(((x, y), score));
		}
		if Engine::max_player(depth) {
			moves.sort_by(|a, b| { a.1.cmp(&b.1).reverse() });
		}
		else {
			moves.sort_by(|a, b| { a.1.cmp(&b.1) });
		}
		moves
	}

	fn get_score(&mut self, max_depth: usize, alpha: &GameMeasure, beta: &GameMeasure, x: i64, y: i64, tile: Tile) -> GameMeasure {
		if let Entry::Occupied(entry) = self.comp.entry(self.map.peek_hash(0, 0, Tile::Empty)) {
			if entry.get().depth >= max_depth { return entry.get().score; }
		}
		self.map.place(x, y, tile);
		let alpha_beta = self.alpha_beta(max_depth, *alpha, *beta);
		self.map.undo();
		let result = SearchResult {
			score: alpha_beta.clone(),
			depth: max(self.map.get_move_list().len(), max_depth),
		};
		self.comp.insert(self.map.get_hash(), result);
		alpha_beta
	}

	fn alpha_beta(&mut self, max_depth: usize, mut alpha: GameMeasure, mut beta: GameMeasure) -> GameMeasure {
		let depth = self.map.get_move_list().len();
		let player_tile = if Engine::max_player(depth) { Tile::X } else { Tile::O };
		let now_heur = self.map.get_heuristic();
		if (depth >= max_depth && !now_heur.is_critical()) || now_heur.won_by() != 0 { return *now_heur; }

		let moves = self.sort_moves();

		let mut val;
		if Engine::max_player(depth) {
			val = GameMeasure::new_min();
	        for ((x, y), _) in moves {
				let score = self.get_score(max_depth, &alpha, &beta, x, y, player_tile);
	            val = val.max(score);
		        alpha = alpha.max(val);
	            if alpha >= beta { break; }
	        }
	    } else {
			val = GameMeasure::new_max();
	        for ((x, y), _) in moves {
				let score = self.get_score(max_depth, &alpha, &beta, x, y, player_tile);
		        val = val.min(score);
		        beta = beta.min(val);
	            if beta <= alpha { break; }
	        }
	    }
		val
	}

	pub fn run_search(&mut self, max_depth: usize) {
		let old_hash = self.map.get_hash();
		let old_heur = *self.map.get_heuristic();
		let depth = self.map.get_move_list().len();
		self.map.clean_tree();
		self.search_cnt = 0;
		self.alpha_beta(max_depth, GameMeasure::new_min(), GameMeasure::new_max());
		debug_assert_eq!(self.map.get_hash(), old_hash);
		debug_assert_eq!(*self.map.get_heuristic(), old_heur);
	}

	pub fn do_step(&mut self, x: i64, y: i64) {
		let depth = self.map.get_move_list().len();
		let player_tile = if Engine::max_player(depth) { Tile::X } else { Tile::O };
		self.map.place(x, y, player_tile);
	}

	pub fn undo_step(&mut self) {
		self.map.undo();
	}

	pub fn get_best_move(&mut self) -> (i64, i64) {
		self.sort_moves().first().unwrap().0
	}
}

impl Debug for Engine {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let cand = self.move_candidates(1);
		let min_x = cand.iter().map(|val| val.0).min().unwrap();
		let max_x = cand.iter().map(|val| val.0).max().unwrap();
		let min_y = cand.iter().map(|val| val.1).min().unwrap();
		let max_y = cand.iter().map(|val| val.1).max().unwrap();

		for x in min_x..max_x {
			write!(f, "{}", " ".repeat((max_x - x) as usize))?;
			for y in min_y..max_y {
				write!(f, "{} ",
				       if cand.contains(&(x, y)) { '_' }
				       else { self.map.get_tile(x, y).to_char() })?
			}
			writeln!(f)?
		}

		writeln!(f, "{:?}", self.map.get_heuristic())?;
		writeln!(f, "{:?}", self.search_cnt)?;
		Ok(())
	}
}