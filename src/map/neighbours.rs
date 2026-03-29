use super::{*};
use super::chunk::{*};

pub struct Neighbours<'a> {
	pub chunks: [[Option<&'a Chunk>; 3]; 3]
}

impl Neighbours<'_> {
	pub fn get_tile(&self, mut x: i64, mut y: i64) -> Tile {
		let mut idx = 1;
		let mut idy = 1;
		if x < 0 { idx -= 1; x += CHUNK_SIZE as i64 }
		else if x >= CHUNK_SIZE as i64 { idx += 1; x -= CHUNK_SIZE as i64 }
		if y < 0 { idy -= 1; y += CHUNK_SIZE as i64 }
		else if y >= CHUNK_SIZE as i64 { idy += 1; y -= CHUNK_SIZE as i64 }
		debug_assert!(0 <= idx && idx < 3 && 0 <= idy && idy < 3);
		match &self.chunks[idx][idy] {
			None => Tile::Empty,
			Some(chunk) => chunk.get_tile(x as usize, y as usize)
		}
	}

	pub fn on_all_tiles(&self, extra: i64, mut f: impl FnMut(i64, i64)) {
		debug_assert!(0 <= extra && extra < CHUNK_SIZE as i64);
		for x in -extra..CHUNK_SIZE as i64 {
			for y in -extra..CHUNK_SIZE as i64 {
				f(x, y);
			}
		}
	}

	pub fn on_owned_tiles(&self, extra: i64, mut f: impl FnMut(i64, i64)) {
		debug_assert!(0 <= extra && extra < CHUNK_SIZE as i64);
		let min_x: i64 = if self.chunks[0][1].is_some() { 0 } else { -extra };
		let min_y: i64 = if self.chunks[1][0].is_some() { 0 } else { -extra };
		for x in min_x..CHUNK_SIZE as i64 {
			let start_y = if x < 0 && self.chunks[0][0].is_some() { 0 } else { min_y };
			for y in start_y..CHUNK_SIZE as i64 {
				f(x, y);
			}
		}
	}
}