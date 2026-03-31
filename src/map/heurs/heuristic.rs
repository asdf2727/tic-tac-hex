use super::*;
use quad_root::*;
use std::fmt::Debug;

pub trait Heuristic: Debug + Ord + Copy {
	fn new() -> Self;
	fn new_max() -> Self;
	fn new_min() -> Self;

	fn is_critical(&self) -> bool { false }

	fn won_by(&self) -> i16;

	fn update(self: &mut Self, map: &mut QuadRoot, x: i64, y: i64, mult: i16);
	fn update_done(&mut self, step: u64);
}