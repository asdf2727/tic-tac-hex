use super::{*};
use chunk::{*};
use quad_node::{*};
use params::{*};
use super::super::heurs::Heuristic;

pub struct GameMap {
	off_x: usize,
	off_y: usize,
	lvl: Level,
	quad_tree: QuadNode,
	heuristic: Box<dyn Heuristic>,
}

impl GameMap {
	pub fn new<T: Heuristic + 'static>() -> GameMap {
		GameMap {
			off_x: 0,
			off_y: 0,
			lvl: 0,
			quad_tree: QuadNode::new(),
			heuristic: T::new(),
		}
	}

	fn get_heuristic(&mut self) -> &dyn Heuristic {
		&*self.heuristic
	}

	fn get_chunk(&mut self, x: usize, y: usize) -> Option<&Chunk> {
		let max_pos = 1 << (self.lvl + 1) * NODE_LOG_SIZE;
		if x >= max_pos || y >= max_pos {
			return None
		}
		self.quad_tree.try_get_chunk(x, y, self.lvl)
	}

	pub fn get_tile(&mut self, x: i64, y: i64) -> Tile {
		match self.get_chunk((x >> CHUNK_LOG_SIZE) as usize + self.off_x,
		                     (y >> CHUNK_LOG_SIZE) as usize + self.off_y) {
			None => Tile::Empty,
			Some(chunk) => chunk.get_tile(
				x as usize & (CHUNK_SIZE - 1),
				y as usize & (CHUNK_SIZE - 1))
		}
	}

	pub fn set_tile(&mut self, x: i64, y: i64, tile: Tile) {
		let mut chk_x = (x >> CHUNK_LOG_SIZE) + self.off_x as i64;
		let mut chk_y = (y >> CHUNK_LOG_SIZE) + self.off_y as i64;
		loop {
			let max_pos = 1 << (self.lvl + 1) * NODE_LOG_SIZE;
			if 0 <= chk_x && chk_x < max_pos as i64 && 0 <= chk_y && chk_y < max_pos as i64 { break }

			let wrap_x = if chk_x < 0 { NODE_SIZE - 1 } else { 0 };
			let wrap_y = if chk_y < 0 { NODE_SIZE - 1 } else { 0 };
			self.quad_tree.wrap(wrap_x, wrap_y);

			let shift_x = max_pos * wrap_x;
			let shift_y = max_pos * wrap_y;
			self.off_x += shift_x;
			self.off_y += shift_y;
			chk_x += shift_x as i64;
			chk_y += shift_y as i64;

			self.lvl += 1;
		}

		let neigh = self.quad_tree.get_chunk_mut(chk_x as usize, chk_y as usize, self.lvl);
		self.heuristic.
		.set_tile(
			x as usize & (CHUNK_SIZE - 1),
			y as usize & (CHUNK_SIZE - 1), tile)
	}
}

impl std::fmt::Debug for GameMap {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let max_pos: i64 = 1 << (CHUNK_LOG_SIZE + (self.lvl + 1) * NODE_LOG_SIZE);
		for x in 0i64 - (CHUNK_SIZE * self.off_x) as i64..max_pos - (CHUNK_SIZE * self.off_x) as i64 {
			write!(f, "{}", " ".repeat((max_pos - x) as usize))?;
			for y in 0i64 - (CHUNK_SIZE * self.off_y) as i64..max_pos - (CHUNK_SIZE * self.off_y) as i64 {
				let mut delim: char = ' ';
				if y - (x >> 1) == 0 && (x & 1) != 0 { delim = '|'; }
				if x == 0 { delim = '-'; }

				let tile = match self.quad_tree.try_get_chunk_const(
					((x >> CHUNK_LOG_SIZE) + self.off_x as i64) as usize,
					((y >> CHUNK_LOG_SIZE) + self.off_y as i64) as usize, self.lvl) {
					None => Tile::Empty,
					Some(chunk) => chunk.get_tile(
						x as usize & (CHUNK_SIZE - 1),
						y as usize & (CHUNK_SIZE - 1))
				};

				write!(f, "{}{}", tile, delim)?
			}
			writeln!(f)?
		}
		Ok(())
	}
}