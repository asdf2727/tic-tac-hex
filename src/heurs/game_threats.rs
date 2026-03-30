use std::cmp::Ordering;
use crate::map::{Neighbours, Tile};

pub const WIN_LEN: usize = 6;

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct GameThreats {
	pub threats_x: [i64; WIN_LEN],
	pub threats_o: [i64; WIN_LEN],
}

impl GameThreats {
	fn on_line(&mut self, line: &[Tile; WIN_LEN], mult: i64) {
		let mut cnt_x = 0;
		let mut cnt_o = 0;
		for tile in line {
			match tile {
				Tile::Empty => {},
				Tile::X => cnt_x += 1,
				Tile::O => cnt_o += 1,
			}
		}
		if cnt_x == 0 && cnt_o != 0 {
			self.threats_o[cnt_o - 1] += mult;
		}
		if cnt_x != 0 && cnt_o == 0 {
			self.threats_x[cnt_x - 1] += mult;
		}
	}
}

impl Ord for GameThreats {
	fn cmp(&self, other: &Self) -> Ordering {
		for i in WIN_LEN - 1..0 {
			match self.threats_x[i].cmp(&other.threats_x[i]) {
				Ordering::Equal => {},
				other => return other,
			}
		}
		Ordering::Equal
	}
}

impl PartialOrd<Self> for GameThreats {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl super::heuristic::Heuristic for GameThreats {
	fn new() -> GameThreats {
		GameThreats {
			threats_x: std::array::repeat(0),
			threats_o: std::array::repeat(0),
		}
	}
	fn new_max() -> GameThreats {
		let mut new_threats = GameThreats::new();
		new_threats.threats_x[WIN_LEN - 1] = 1;
		new_threats
	}
	fn new_min() -> GameThreats {
		let mut new_threats = GameThreats::new();
		new_threats.threats_o[WIN_LEN - 1] = 1;
		new_threats
	}

	fn get_extra(&self) -> i64 { (WIN_LEN - 1) as i64 }

	fn run_heuristic(&mut self, neigh: &Neighbours, x: i64, y: i64, mult: i64) {
		self.on_line(&std::array::from_fn(|i| neigh.get_tile(x, y + i as i64)), mult);
		self.on_line(&std::array::from_fn(|i| neigh.get_tile(x + i as i64, y)), mult);
		self.on_line(&std::array::from_fn(|i| neigh.get_tile(x + i as i64, y + i as i64)), mult);
	}
}
