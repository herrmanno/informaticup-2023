use crate::object::Coord;

pub type Point = (Coord, Coord);

// fn neighbours(x: u32, y: u32) -> Vec<Coord> {
//     if x > 0 && y > 0 {
//         vec![(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]
//     } else if x > 0 {
//         vec![(x - 1, y), (x + 1, y), (x, y + 1)]
//     } else if y > 0 {
//         vec![(x + 1, y), (x, y - 1), (x, y + 1)]
//     } else {
//         vec![(x + 1, y), (x, y + 1)]
//     }
// }

pub fn neighbours(x: Coord, y: Coord) -> [Point; 4] {
    [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]
}
