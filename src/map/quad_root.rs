use super::*;
use chunk::*;
use params::*;
use quad_node::*;

pub struct QuadRoot {
	off_x: usize,
	off_y: usize,
	lvl: Level,
	quad_tree: QuadNode,
}

// TODO implement caching of previously found nodes to reduce the number of calls to get_chunk

impl QuadRoot {
	pub fn new() -> QuadRoot {
		QuadRoot {
			off_x: 1,
			off_y: 1,
			lvl: 1,
			quad_tree: QuadNode::new(),
		}
	}

	pub fn get_tile(&self, x: i64, y: i64) -> Tile {
		let chk_x = (x >> CHUNK_LOG_SIZE) + self.off_x as i64;
		let chk_y = (y >> CHUNK_LOG_SIZE) + self.off_y as i64;
		let max_pos = 1 << (self.lvl * NODE_LOG_SIZE);
		if 0 > chk_x || chk_x >= max_pos || 0 > chk_y || chk_y >= max_pos {
			return Tile::Empty;
		}
		let chk_x = chk_x as usize;
		let chk_y = chk_y as usize;
		let loc_x = (x & (CHUNK_SIZE - 1) as i64) as usize;
		let loc_y = (y & (CHUNK_SIZE - 1) as i64) as usize;
		match self.quad_tree.try_get_chunk(chk_x, chk_y, self.lvl) {
			None => Tile::Empty,
			Some(chunk) => chunk.get_tile(loc_x, loc_y)
		}
	}

	pub fn set_tile(&mut self, x: i64, y: i64, tile: Tile) -> Tile {
		let mut chk_x = (x >> CHUNK_LOG_SIZE) + self.off_x as i64;
		let mut chk_y = (y >> CHUNK_LOG_SIZE) + self.off_y as i64;
		loop {
			let max_pos = 1 << (self.lvl * NODE_LOG_SIZE);
			if 0 <= chk_x && chk_x < max_pos as i64 && 0 <= chk_y && chk_y < max_pos as i64 { break }

			let wrap_x = if chk_x < 0 { NODE_SIZE - 1 } else { 0 };
			let wrap_y = if chk_y < 0 { NODE_SIZE - 1 } else { 0 };
			let old_root = std::mem::replace(&mut self.quad_tree, QuadNode::new());
			self.quad_tree.chd[wrap_x][wrap_y] = QuadChd::Node(Box::new(old_root));

			let shift_x = max_pos * wrap_x;
			let shift_y = max_pos * wrap_y;
			self.off_x += shift_x;
			self.off_y += shift_y;
			chk_x += shift_x as i64;
			chk_y += shift_y as i64;

			self.lvl += 1;
		}

		let chk_x = chk_x as usize;
		let chk_y = chk_y as usize;
		let loc_x = (x & (CHUNK_SIZE - 1) as i64) as usize;
		let loc_y = (y & (CHUNK_SIZE - 1) as i64) as usize;
		let chunk = match tile {
			Tile::Empty => match self.quad_tree.try_get_chunk_mut(chk_x, chk_y, self.lvl) {
				Some(chunk) => chunk,
				None => return Tile::Empty,
			}
			_ => self.quad_tree.get_chunk_mut(chk_x, chk_y, self.lvl)
		};
		chunk.set_tile(loc_x, loc_y, tile)
	}

	pub fn clean(&mut self) {
		self.quad_tree.clean();
	}
}

impl std::fmt::Debug for QuadRoot {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let size: i64 = 1 << (CHUNK_LOG_SIZE + self.lvl * NODE_LOG_SIZE);
		let min_x = -((CHUNK_SIZE * self.off_x) as i64);
		let max_x = min_x + size;
		let min_y = -((CHUNK_SIZE * self.off_y) as i64);
		let max_y = min_x + size;

		for x in min_x..max_x {
			write!(f, "{}", " ".repeat((max_x - x) as usize))?;
			for y in min_y..max_y {
				let mut delim: char = ' ';
				if y - (x >> 1) == 0 && (x & 1) != 0 { delim = '|'; }
				if x == 0 { delim = '-'; }
				write!(f, "{}{}", self.get_tile(x, y), delim)?
			}
			writeln!(f)?
		}
		Ok(())
	}
}