use crate::game_params::WIN_LEN;
use crate::map::Neighbours;

pub trait Heuristic {
	fn new() -> Box<Self> where Self: Sized;

	fn from(all: Vec<Neighbours>) -> Box<Self> where Self: Sized {
		let mut new_threats: Box<Self> = Heuristic::new();
		for neigh in all {
			neigh.on_owned_tiles((WIN_LEN - 1) as i64, |x, y| {
				new_threats.run_heuristic(&neigh, x, y, 1);
			});
		}
		new_threats
	}

	fn run_heuristic(&mut self, neigh: &Neighbours, x: i64, y: i64, mult: i64);

	fn update(&mut self, neigh: &Neighbours, mult: i64) {
		neigh.on_all_tiles((WIN_LEN - 1) as i64, |x, y| {
			self.run_heuristic(neigh, x, y, mult)
		});
	}
}