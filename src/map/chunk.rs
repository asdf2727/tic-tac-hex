use super::*;
use tile::TILE_BITS;

pub const CHUNK_SIZE: usize = 1 << params::CHUNK_LOG_SIZE;

type ChunkData = u64;
const DATA_ENTRIES: usize = (size_of::<ChunkData>() * 8 + 1) / TILE_BITS;
const DATA_LEN: usize = (CHUNK_SIZE * CHUNK_SIZE + DATA_ENTRIES - 1) / DATA_ENTRIES;
const TILE_MASK: ChunkData = (1 << TILE_BITS) - 1;

pub struct Chunk {
	data: [ChunkData; DATA_LEN],
}

macro_rules! shift {
	($i:expr) => {
		($i % DATA_ENTRIES) * TILE_BITS
	};
}

impl Chunk {
	pub fn new() -> Chunk {
		Chunk {
			data: std::array::repeat(0),
		}
	}

	fn data_to_tile(&self, data: ChunkData) -> Tile {
		match data {
			0 => Tile::Empty,
			1 => Tile::X,
			2 => Tile::O,
			_ => unreachable!("Invalid tile value: {}", data),
		}
	}

	pub fn get_tile(&self, x: usize, y: usize) -> Tile {
		debug_assert!(x < CHUNK_SIZE);
		debug_assert!(y < CHUNK_SIZE);
		let id = x + y * CHUNK_SIZE;
		let tile_val = self.data[id / DATA_ENTRIES] >> shift!(id) & TILE_MASK;
		self.data_to_tile(tile_val)
	}

	pub fn set_tile(&mut self, x: usize, y: usize, tile: Tile) -> Tile {
		debug_assert!(x < CHUNK_SIZE);
		debug_assert!(y < CHUNK_SIZE);
		let id = x + y * CHUNK_SIZE;
		let old_tile = self.data[id / DATA_ENTRIES] >> shift!(id) & TILE_MASK;
		self.data[id / DATA_ENTRIES] &= !((TILE_MASK as ChunkData) << shift!(id));
		self.data[id / DATA_ENTRIES] |= (tile as ChunkData) << shift!(id);
		self.data_to_tile(old_tile)
	}

	pub fn is_empty(&self) -> bool {
		self.data.iter().all(|&x| x == 0)
	}
}
