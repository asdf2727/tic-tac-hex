use super::super::map::*;
use std::cmp::*;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

#[derive(Clone, Copy, Debug)]
#[derive(Eq, PartialEq)]
enum SearchBound {
	Lower,
	Upper,
	Exact,
}

#[derive(Clone, Copy, Debug)]
#[derive(Eq, PartialEq)]
struct SearchResult {
	score: GameMeasure,
	bound: SearchBound,
	depth: u64,
}

impl SearchResult {
	fn is_valid(&self, max_depth: u64) -> bool {
		self.depth >= max_depth || match self.bound {
			SearchBound::Lower => self.score.won_by() > 0,
			SearchBound::Upper => self.score.won_by() < 0,
			SearchBound::Exact => self.score.won_by() != 0,
		}
	}
}

#[derive(Default, Debug, Clone, Copy)]
struct Stats {
	get_any_score: usize,
	get_score: usize,
	alpha_beta: usize,
	tt_hits: usize,
	tt_non_hits: usize,
}

pub struct Engine {
	map: GameMap,
	tt: HashMap<Hash, SearchResult>,
	stats: Stats,
}

const MOVE_DIST: i64 = 1;

impl Engine {
	pub fn new(map: GameMap) -> Engine {
		let mut eng = Engine {
			map,
			tt: HashMap::new(),
			stats: Stats::default(),
		};
		eng.tt.insert(eng.map.get_hash(), SearchResult {
			score: *eng.map.get_heuristic(),
			bound: SearchBound::Exact,
			depth: 0,
		});
		eng
	}

	fn move_candidates(&self, max_dist: i64) -> Vec<(i64, i64)> {
		let mut used = self.map.get_move_list().clone();
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
		}
		candidates
	}

	fn get_any_score(&mut self, x: i64, y: i64) -> SearchResult {
		self.stats.get_any_score += 1;
		if let Entry::Occupied(entry) = self.tt.entry(self.map.peek_hash(x, y)) {
			self.stats.tt_hits += 1;
			return *entry.get();
		}
		self.stats.tt_non_hits += 1;
		self.map.place(x, y);
		let result = SearchResult {
			score: *self.map.get_heuristic(),
			bound: SearchBound::Exact,
			depth: self.map.get_depth(),
		};
		self.tt.insert(self.map.get_hash(), result.clone());
		self.map.undo();
		result
	}

	fn get_score(&mut self, max_depth: u64, alpha: &GameMeasure, beta: &GameMeasure,
	             x: i64, y: i64, eval: SearchResult) -> GameMeasure {
		self.stats.get_score += 1;
		if eval.is_valid(max_depth) { return eval.score }
		self.map.place(x, y);
		let result = self.alpha_beta(max_depth, *alpha, *beta);
		self.tt.insert(self.map.get_hash(), result.clone());
		self.map.undo();
		result.score
	}

	fn alpha_beta(&mut self, max_depth: u64, mut alpha: GameMeasure, mut beta: GameMeasure) -> SearchResult {
		self.stats.alpha_beta += 1;
		if self.stats.alpha_beta % 1000000 == 0 {
			println!("{:?}", self);
		}

		let depth = self.map.get_depth();
		let now_heur = self.map.get_heuristic();
		if depth >= max_depth + 2 || now_heur.won_by() != 0 ||
			(depth >= max_depth && !now_heur.is_critical()) {
			return SearchResult { score: *now_heur, bound: SearchBound::Exact, depth };
		}

		let mut moves = self.move_candidates(MOVE_DIST).iter()
			.map(|&(x, y)| ((x, y), self.get_any_score(x, y)))
			.collect::<Vec<_>>();

		let mut val: GameMeasure;
		if self.map.get_player() == Tile::X {
			moves.sort_by(|a, b| {
				a.1.score.cmp(&b.1.score).reverse().then(a.1.depth.cmp(&b.1.depth).reverse()) });
			val = GameMeasure::new_min();
			for ((x, y), eval) in moves {
				let score = self.get_score(max_depth, &alpha, &beta, x, y, eval);
				val = val.max(score);
				alpha = alpha.max(score);
				if alpha >= beta {
					return SearchResult { score, bound: SearchBound::Lower, depth: max_depth };
				}
		    }
		}
		else {
			moves.sort_by(|a, b| {
				a.1.score.cmp(&b.1.score).then(a.1.depth.cmp(&b.1.depth).reverse())
			});
			val = GameMeasure::new_max();
			for ((x, y), eval) in moves {
				let score = self.get_score(max_depth, &alpha, &beta, x, y, eval);
				val = val.min(score);
				beta = beta.min(score);
				if alpha >= beta {
					return SearchResult { score, bound: SearchBound::Upper, depth: max_depth };
				}
		    }
		}
		SearchResult { score: val, bound: SearchBound::Exact, depth: max_depth, }
	}

	pub fn run_search(&mut self, max_depth: u64) {
		let old_hash = self.map.get_hash();
		let old_heur = *self.map.get_heuristic();
		let depth = self.map.get_depth();
		self.stats = Stats::default();
		self.alpha_beta(depth + max_depth, GameMeasure::new_min(), GameMeasure::new_max());
		debug_assert_eq!(self.map.get_depth(), depth);
		debug_assert_eq!(self.map.get_hash(), old_hash);
		debug_assert_eq!(*self.map.get_heuristic(), old_heur);
	}

	pub fn get_best_move(&mut self) -> (i64, i64) {
		let mut moves = self.move_candidates(MOVE_DIST).iter()
				.map(|&(x, y)| ((x, y), self.get_any_score(x, y)))
				.collect::<Vec<_>>();
		if self.map.get_player() == Tile::X {
			moves.sort_by(|a, b| {
				a.1.score.cmp(&b.1.score).reverse().then(a.1.depth.cmp(&b.1.depth).reverse()) });
		}
		else {
			moves.sort_by(|a, b| {
				a.1.score.cmp(&b.1.score).then(a.1.depth.cmp(&b.1.depth).reverse())
			});
		}
		println!("{:?}", moves[0]);
		println!("{:?}", moves[1]);
		println!("{:?}", moves[2]);
		println!("{:?}", moves[3]);
		println!("{:?}", moves[4]);
		moves[0].0
	}

	pub fn won_by(&self) -> i64 { self.map.get_heuristic().won_by() as i64 }

	pub fn place(&mut self, x: i64, y: i64) {
		self.map.place(x, y);
	}
	pub fn undo(&mut self) {
		self.map.undo();
	}
}

impl Debug for Engine {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let cand = self.move_candidates(MOVE_DIST);
		let min_x = cand.iter().map(|val| val.0).min().unwrap();
		let max_x = cand.iter().map(|val| val.0).max().unwrap() + 1;
		let min_y = cand.iter().map(|val| val.1).min().unwrap();
		let max_y = cand.iter().map(|val| val.1).max().unwrap() + 1;

		let depth = self.map.get_depth();
		write!(f, "Turn {}, move {} for {}\n", depth, 1 + (depth & 1), self.map.get_player())?;

		for x in min_x..max_x {
			write!(f, "{}", " ".repeat((max_x - x) as usize))?;
			for y in min_y..max_y {
				if cand.contains(&(x, y)) { write!(f, "_ ")? }
				else { write!(f, "{} ", self.map.get_tile(x, y))? }
			}
			write!(f, "\n")?
		}

		write!(f, "{:?}\n", self.map.get_heuristic())?;
		write!(f, "{:?}, {:?}\n", self.map.tree_level(), self.stats)?;
		Ok(())
	}
}