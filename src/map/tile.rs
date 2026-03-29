#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
pub enum Tile {Empty = 0, X = 1, O = 2}

pub const TILE_BITS: usize = 2;

impl std::fmt::Display for Tile {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Tile::Empty => write!(f, " "),
			Tile::X => write!(f, "X"),
			Tile::O => write!(f, "O"),
		}
	}
}