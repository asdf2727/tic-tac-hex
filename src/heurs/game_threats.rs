use crate::game_params::WIN_LEN;
use crate::map::{Neighbours, Tile};

#[derive(Debug)]
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

impl super::heuristic::Heuristic for GameThreats {
	fn new() -> Box<GameThreats> {
		Box::new(GameThreats {
			threats_x: std::array::repeat(0),
			threats_o: std::array::repeat(0),
		})
	}

	fn run_heuristic(&mut self, neigh: &Neighbours, x: i64, y: i64, mult: i64) {
		self.on_line(&std::array::from_fn(|i| neigh.get_tile(x, y + i as i64)), mult);
		self.on_line(&std::array::from_fn(|i| neigh.get_tile(x + i as i64, y)), mult);
		self.on_line(&std::array::from_fn(|i| neigh.get_tile(x + i as i64, y + i as i64)), mult);
	}
}
