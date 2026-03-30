use std::cmp::{min, Ordering};
use super::*;
use quad_root::*;

pub const WIN_LEN: usize = 6;

#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(Eq, PartialEq)]
pub struct GameThreats {
	step: i32,
	threats_x: [i16; WIN_LEN],
	threats_o: [i16; WIN_LEN],
	threats_diff: [i16; WIN_LEN],
}

impl GameThreats {
	fn on_line(&mut self, line: [Tile; 2 * WIN_LEN - 1], mult: i16) {
		let mut cnt_x = 0;
		let mut cnt_o = 0;
		for tile in line[0..WIN_LEN - 1].iter() {
			match tile {
				Tile::Empty => {},
				Tile::X => cnt_x += 1,
				Tile::O => cnt_o += 1,
			}
		}
		for i in WIN_LEN - 1..2 * WIN_LEN - 1 {
			match line[i] {
				Tile::Empty => {},
				Tile::X => cnt_x += 1,
				Tile::O => cnt_o += 1,
			}
			if cnt_x != 0 && cnt_o == 0 {
				self.threats_x[cnt_x - 1] += mult;
			}
			if cnt_x == 0 && cnt_o != 0 {
				self.threats_o[cnt_o - 1] += mult;
			}
			match line[i + 1 - WIN_LEN] {
				Tile::Empty => {},
				Tile::X => cnt_x -= 1,
				Tile::O => cnt_o -= 1,
			}
		}
	}

	fn apply_correction(&mut self) {
		let advantage = if self.step & 2 == 0 { -1 } else { 1 };
		let shift = (2 - (self.step & 1)) as usize;
		let mut first = true;
		for i in (0..WIN_LEN).into_iter().rev() {
			self.threats_diff[i] = self.threats_x[i] - self.threats_o[i];
			if first && ((self.step & 2 == 0 && self.threats_o[i] != 0) ||
				(self.step & 2 != 0 && self.threats_x[i] != 0)) {
				first = false;
				self.threats_diff[i] -= advantage;
				self.threats_diff[min(i + shift, WIN_LEN - 1)] += advantage;
			}
		}
	}
}

impl Ord for GameThreats {
	fn cmp(&self, other: &Self) -> Ordering {
		for i in (0..WIN_LEN).rev() {
			match self.threats_diff[i].cmp(&other.threats_diff[i]) {
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

impl Heuristic for GameThreats {
	fn new() -> GameThreats {
		GameThreats {
			step: 0,
			threats_x: std::array::repeat(0),
			threats_o: std::array::repeat(0),
			threats_diff: std::array::repeat(0),
		}
	}
	fn new_max() -> GameThreats {
		let mut new_threats = GameThreats::new();
		new_threats.threats_x[WIN_LEN - 1] = 1;
		new_threats.threats_diff[WIN_LEN - 1] = 1;
		new_threats
	}
	fn new_min() -> GameThreats {
		let mut new_threats = GameThreats::new();
		new_threats.threats_o[WIN_LEN - 1] = 1;
		new_threats.threats_diff[WIN_LEN - 1] = -1;
		new_threats
	}

	fn get_extra(&self) -> i64 { (WIN_LEN - 1) as i64 }

	fn is_critical(&self) -> bool { self.threats_diff[WIN_LEN - 1] != 0 }

	fn won_by(&self) -> i16 { self.threats_diff[WIN_LEN - 1] }

	fn update(&mut self, map: &mut QuadRoot, x: i64, y: i64, mult: i16) {
		let offset = WIN_LEN as i64 - 1;
		macro_rules! make_dir {
		    ($map:expr) => {
			    self.on_line(std::array::from_fn($map), mult);
		    };
		}
		make_dir!(|i| map.get_tile(x - offset + i as i64, y));
		make_dir!(|i| map.get_tile(x, y - offset + i as i64));
		make_dir!(|i| map.get_tile(x - offset + i as i64, y - offset + i as i64));
	}

	fn update_step(&mut self, step: i32) {
		self.step += step;
		self.apply_correction();
	}
}
