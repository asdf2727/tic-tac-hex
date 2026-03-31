use std::cmp::{min, Ordering};
use super::*;
use quad_root::*;

pub const WIN_LEN: usize = 6;

#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(Eq, PartialEq)]
pub struct GameThreats {
	threats_x: [i16; WIN_LEN],
	threats_o: [i16; WIN_LEN],
	threats_diff: [i16; WIN_LEN],
	won_by: i16,
}

impl GameThreats {
	fn on_line(&mut self, line: &[Tile; 2 * WIN_LEN - 1], mult: i16) {
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
}

impl Ord for GameThreats {
	fn cmp(&self, other: &Self) -> Ordering {
		match self.won_by().cmp(&other.won_by()) {
			Ordering::Equal => {},
			other => return other,
		}
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
			threats_x: std::array::repeat(0),
			threats_o: std::array::repeat(0),
			threats_diff: std::array::repeat(0),
			won_by: 0,
		}
	}
	fn new_max() -> GameThreats {
		let mut new_threats = GameThreats::new();
		new_threats.threats_x[WIN_LEN - 1] = 1;
		new_threats.threats_diff[WIN_LEN - 1] = 1;
		new_threats.won_by = 1;
		new_threats
	}
	fn new_min() -> GameThreats {
		let mut new_threats = GameThreats::new();
		new_threats.threats_o[WIN_LEN - 1] = 1;
		new_threats.threats_diff[WIN_LEN - 1] = -1;
		new_threats.won_by = -1;
		new_threats
	}

	fn is_critical(&self) -> bool {
		self.threats_diff[WIN_LEN - 1] != 0 ||
			self.threats_diff[WIN_LEN - 2] != 0 ||
			self.threats_diff[WIN_LEN - 3] != 0
	}

	fn won_by(&self) -> i16 { self.won_by }

	fn update(&mut self, map: &mut QuadRoot, x: i64, y: i64, mult: i16) {
		let offset = WIN_LEN as i64 - 1;
		let mut arr: [Tile; 2 * WIN_LEN - 1] = [Tile::Empty; 2 * WIN_LEN - 1];
		for i in 0..2 * WIN_LEN - 1 {
			arr[i] = map.get_tile(x - offset + i as i64, y);
		}
		self.on_line(&arr, mult);
		for i in 0..2 * WIN_LEN - 1 {
			arr[i] = map.get_tile(x, y - offset + i as i64);
		}
		self.on_line(&arr, mult);
		for i in 0..2 * WIN_LEN - 1 {
			arr[i] = map.get_tile(x - offset + i as i64, y - offset + i as i64);
		}
		self.on_line(&arr, mult);
	}

	fn update_done(&mut self, step: u64) {
		let advantage = if step & 2 == 0 { -1 } else { 1 };
		let shift = (2 - (step & 1)) as usize;
		let mut first = false;
		for i in (0..WIN_LEN).rev() {
			self.threats_diff[i] = self.threats_x[i] - self.threats_o[i];
			if first && ((step & 2 == 0 && self.threats_o[i] != 0) ||
				(step & 2 != 0 && self.threats_x[i] != 0)) {
				first = false;
				self.threats_diff[i] -= advantage;
				self.threats_diff[min(i + shift, WIN_LEN - 1)] += advantage;
			}
		}

		macro_rules! ret_not_zero {
            ($x:expr) => { if $x != 0 { self.won_by = $x; return; } };
		}
		if step & 2 == 0 {
			if step & 1 == 0 { ret_not_zero!(-self.threats_o[WIN_LEN - 3]); }
			ret_not_zero!(-self.threats_o[WIN_LEN - 2]);
			ret_not_zero!(-self.threats_o[WIN_LEN - 1]);
			self.won_by = self.threats_x[WIN_LEN - 1];
		}
		else {
			if step & 1 == 0 { ret_not_zero!(self.threats_x[WIN_LEN - 3]); }
			ret_not_zero!(self.threats_x[WIN_LEN - 2]);
			ret_not_zero!(self.threats_x[WIN_LEN - 1]);
			self.won_by = -self.threats_o[WIN_LEN - 1];
		}
	}
}
