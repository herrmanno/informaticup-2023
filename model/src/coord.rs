use crate::object::Coord;

/// A 2d coordinate on a [Map]
pub type Point = (Coord, Coord);

/// Generates all four adjacent neighbours of a [Coord]
pub fn neighbours(x: Coord, y: Coord) -> [Point; 4] {
    [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]
}
