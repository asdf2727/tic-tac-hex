use crate::map::Neighbours;
use std::fmt::Debug;

pub trait Heuristic: Debug + Ord {
	fn new() -> Self;
	fn new_max() -> Self;
	fn new_min() -> Self;

	fn get_extra(&self) -> i64;

	fn from(all: Vec<Neighbours>) -> Self where Self: Sized {
		let mut new_threats: Self = Heuristic::new();
		for neigh in all {
			neigh.on_owned_tiles(new_threats.get_extra(), |x, y| {
				new_threats.run_heuristic(&neigh, x, y, 1);
			});
		}
		new_threats
	}

	fn run_heuristic(&mut self, neigh: &Neighbours, x: i64, y: i64, mult: i64);

	fn update(self: &mut Self, neigh: &Neighbours, mult: i64) {
		neigh.on_all_tiles(self.get_extra(), |x, y| {
			self.run_heuristic(neigh, x, y, mult)
		});
	}
}