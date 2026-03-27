use crate::game_params::WIN_LEN;
use crate::game_map::{CheckResult, LineChecker, Tile};

struct GameThreats {
	threats_x: [u64; WIN_LEN],
	threats_o: [u64; WIN_LEN],
}

impl GameThreats {
	fn new() -> GameThreats {
		GameThreats {
			threats_x: [0; WIN_LEN],
			threats_o: [0; WIN_LEN],
		}
	}
}

impl LineChecker for GameThreats {
	fn on_line(&mut self, line: &[Tile; WIN_LEN]) -> CheckResult {
		let mut cnt_x = 0;
		let mut cnt_o = 0;
		for tile in line {
			match tile {
				Tile::Empty => {},
				Tile::X => cnt_x += 1,
				Tile::O => cnt_o += 1,
			}
		}
		if cnt_o == 6 && cnt_x == 6 { return Err(()) }
		if cnt_x == 0 {
			if cnt_o == 0 { return Ok(()) }
			self.threats_o[cnt_o - 1] += 1;
		}
		else {
			if cnt_o != 0 { return Ok(()) }
			self.threats_x[cnt_x - 1] += 1;
		}
		Ok(())
	}
}