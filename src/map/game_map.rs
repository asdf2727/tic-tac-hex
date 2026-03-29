use super::{*};
use chunk::{*};
use quad_node::{*};
use params::{*};
use super::super::heurs::Heuristic;

pub type Hash = u64;

pub struct GameMap {
	off_x: usize,
	off_y: usize,
	lvl: Level,
	quad_tree: QuadNode,
	heuristic: Box<dyn Heuristic>,
	hash: Hash,
}

impl GameMap {
	pub fn new<T: Heuristic + 'static>() -> GameMap {
		GameMap {
			off_x: 1,
			off_y: 1,
			lvl: 1,
			quad_tree: QuadNode::new(),
			heuristic: T::new(),
			hash: 0,
		}
	}

	pub fn get_heuristic(&mut self) -> &dyn Heuristic {
		&*self.heuristic
	}

	pub fn get_hash(&self) -> Hash { self.hash }

	pub fn get_tile(&mut self, x: i64, y: i64) -> Tile {
		let chk_x = (x >> CHUNK_LOG_SIZE) + self.off_x as i64;
		let chk_y = (y >> CHUNK_LOG_SIZE) + self.off_y as i64;
		let max_pos = 1 << (self.lvl * NODE_LOG_SIZE);
		if 0 > chk_x || chk_x >= max_pos || 0 > chk_y || chk_y >= max_pos {
			return Tile::Empty;
		}
		let chk_x = chk_x as usize;
		let chk_y = chk_y as usize;
		match self.quad_tree.try_get_chunk(chk_x, chk_y, self.lvl) {
			None => Tile::Empty,
			Some(chunk) => chunk.get_tile(
				x as usize & (CHUNK_SIZE - 1),
				y as usize & (CHUNK_SIZE - 1))
		}
	}

	fn build_neighbours(tree: &QuadNode, x: usize, y: usize, lvl: Level) -> Option<Neighbours> {
		let Some(mid) = tree.try_get_chunk_const(x, y, lvl) else { return None; };
		let size = 1 << (lvl * NODE_LOG_SIZE);

		macro_rules! safe_get_const {
			($x:expr, $y:expr) => {
				if 0 > $x || $x >= size || 0 > $y || $y >= size { None }
				else { tree.try_get_chunk_const($x as usize, $y as usize, lvl) }
			};
		}

		let x = x as i64;
		let y = y as i64;

		Some(Neighbours {
			chunks: [[
				safe_get_const!(x - 1, y - 1),
				safe_get_const!(x - 1, y),
				safe_get_const!(x - 1, y + 1),
			], [
				safe_get_const!(x, y - 1),
				Some(mid),
				safe_get_const!(x, y + 1),
			], [
				safe_get_const!(x + 1, y - 1),
				safe_get_const!(x + 1, y),
				safe_get_const!(x + 1, y + 1),
			]]
		})
	}

	fn get_neighbours(tree: &QuadNode, x: usize, y: usize, lvl: Level) -> Option<Neighbours> {
		let size = 1 << (lvl * NODE_LOG_SIZE);
		if x == 0 || x == size - 1 || y == 0 || y == size - 1 {
			return GameMap::build_neighbours(tree, x, y, lvl);
		}
		tree.get_neighbours(x, y, lvl)
	}

	/// See https://github.com/lemire/testingRNG/blob/master/source/splitmix64.h
	const fn splitmix64(mut x: Hash) -> Hash {
		x = x.wrapping_add(0x9E3779B97F4A7C15);
		x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
		x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
		x ^ (x >> 31)
	}
	fn get_tile_hash(x: usize, y: usize, tile: Tile) -> Hash {
		let tile_hash = match tile {
			Tile::Empty => return 0,
			Tile::X => 0xC85A0D3A7E3B7508,
			Tile::O => 0x3658570FED0B5A86,
		};
		Self::splitmix64((x ^ (y << 32)) as Hash ^ tile_hash)
	}

	pub fn set_tile(&mut self, x: i64, y: i64, tile: Tile) {
		let mut chk_x = (x >> CHUNK_LOG_SIZE) + self.off_x as i64;
		let mut chk_y = (y >> CHUNK_LOG_SIZE) + self.off_y as i64;
		loop {
			let max_pos = 1 << (self.lvl * NODE_LOG_SIZE);
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

		let chk_x = chk_x as usize;
		let chk_y = chk_y as usize;
		let x = (x & (CHUNK_SIZE - 1) as i64) as usize;
		let y = (y & (CHUNK_SIZE - 1) as i64) as usize;

		// TODO: Optimize by unsafely modifying the neighbours of the chunk directly
		if let Some(neigh) = GameMap::get_neighbours(&self.quad_tree, chk_x, chk_y, self.lvl) {
			self.heuristic.update(&neigh, -1);
		}

		let chunk = self.quad_tree.get_chunk_mut(chk_x, chk_y, self.lvl);
		self.hash ^= Self::get_tile_hash(x, y, chunk.get_tile(x, y));
		self.hash ^= Self::get_tile_hash(x, y, tile);
		chunk.set_tile(x, y, tile);

		if let Some(neigh) = GameMap::get_neighbours(&self.quad_tree, chk_x, chk_y, self.lvl) {
			self.heuristic.update(&neigh, 1);
		}
	}
}

impl std::fmt::Debug for GameMap {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let size: i64 = 1 << (CHUNK_LOG_SIZE + self.lvl * NODE_LOG_SIZE);
		let min_x = -((CHUNK_SIZE * self.off_x) as i64);
		let max_x = min_x + size;
		let min_y = -((CHUNK_SIZE * self.off_y) as i64);
		let max_y = min_x + size;

		for x in min_x..max_x {
			write!(f, "{}", " ".repeat((size - x) as usize))?;
			let chk_x = (x >> CHUNK_LOG_SIZE) + self.off_x as i64;

			for y in min_y..max_y {
				let mut delim: char = ' ';
				if y - (x >> 1) == 0 && (x & 1) != 0 { delim = '|'; }
				if x == 0 { delim = '-'; }

				let chk_y = (y >> CHUNK_LOG_SIZE) + self.off_y as i64;
				let tile = match self.quad_tree.try_get_chunk_const(chk_x as usize, chk_y as usize, self.lvl) {
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