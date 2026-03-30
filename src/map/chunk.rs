use super::*;

pub const CHUNK_SIZE: usize = 1 << params::CHUNK_LOG_SIZE;

type ChunkData = u64;
const DATA_ENTRIES: usize = (size_of::<ChunkData>() + 1) / tile::TILE_BITS;
const DATA_LEN: usize = (CHUNK_SIZE * CHUNK_SIZE + DATA_ENTRIES - 1) / DATA_ENTRIES;

pub struct Chunk {
	data: [ChunkData; DATA_LEN],
}

impl Chunk {
	pub fn new() -> Chunk {
		Chunk {
			data: std::array::repeat(0),
		}
	}

	pub fn get_tile(&self, x: usize, y: usize) -> Tile {
		debug_assert!(x < CHUNK_SIZE);
		debug_assert!(y < CHUNK_SIZE);
		let id = x + y * CHUNK_SIZE;
		let tile_val = self.data[id / DATA_ENTRIES] >> ((id % DATA_ENTRIES) * 2) & 3;
		match tile_val {
			0 => Tile::Empty,
			1 => Tile::X,
			2 => Tile::O,
			_ => unreachable!("Invalid tile value: {}", tile_val),
		}
	}

	pub fn set_tile(&mut self, x: usize, y: usize, tile: Tile) {
		debug_assert!(x < CHUNK_SIZE);
		debug_assert!(y < CHUNK_SIZE);
		let id = x + y * CHUNK_SIZE;
		self.data[id / DATA_ENTRIES] &= !(3 as ChunkData) << ((id % DATA_ENTRIES) * 2);
		self.data[id / DATA_ENTRIES] |= (tile as ChunkData) << ((id % DATA_ENTRIES) * 2);
	}

	pub fn is_empty(&self) -> bool {
		self.data.iter().all(|&x| x == 0)
	}
}
