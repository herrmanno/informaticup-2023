use std::{collections::HashMap, fmt::Display};

use crate::{
    coord::{neighbours, Coord},
    object::{Object, ObjectCell, ObjectType},
};

#[derive(Debug, Clone)]
pub struct Map {
    width: u32,
    height: u32,
    map: HashMap<Coord, ObjectCell>,
    objects: Vec<MapObject>,
}

#[derive(Debug, Clone)]
pub struct MapObject {
    pub object: Object,
    pub ingresses: Vec<Coord>,
    pub exgresses: Vec<Coord>,
}

impl From<(usize, Object)> for MapObject {
    fn from((index, object): (usize, Object)) -> Self {
        let cells = object.get_cells(index);
        let ingresses = cells
            .iter()
            .cloned()
            .filter(|cell| matches!(cell.1, ObjectCell::Ingress { .. }))
            .map(|(coord, _)| coord)
            .collect();
        let exgresses = cells
            .iter()
            .cloned()
            .filter(|cell| matches!(cell.1, ObjectCell::Exgress { .. }))
            .map(|(coord, _)| coord)
            .collect();

        MapObject {
            object,
            ingresses,
            exgresses,
        }
    }
}

impl Map {
    pub fn new(width: u32, height: u32, objects: Vec<Object>) -> Self {
        let mut map = Map {
            width,
            height,
            objects: Vec::with_capacity(objects.len()),
            map: HashMap::new(),
        };

        for object in objects {
            if let Err(e) = map.insert_object(object) {
                panic!("Cannot create map from objects: '{}'", e);
            }
        }

        map
    }

    pub fn get_objects(&self) -> &Vec<MapObject> {
        &self.objects
    }

    pub fn get_cell(&self, x: i32, y: i32) -> Option<&ObjectCell> {
        self.map.get(&(x, y))
    }

    pub fn width(&self) -> i32 {
        self.width as i32
    }

    pub fn height(&self) -> i32 {
        self.height as i32
    }

    pub fn insert_object(&mut self, object: Object) -> Result<usize, String> {
        let width = self.width();
        let height = self.height();
        let index = self.objects.len();

        let cells = object.get_cells(index);
        for ((x, y), cell) in cells.iter() {
            if *x < 0 || *y < 0 || *x >= width || *y >= height {
                return Err(format!("Cannot insert cell at {:?}", (x, y)));
            } else if let Some(old_cell) = self.map.get(&(*x, *y)) {
                if !(matches!(
                    old_cell,
                    ObjectCell::Inner {
                        kind: ObjectType::Conveyor,
                        ..
                    }
                ) && matches!(
                    cell,
                    ObjectCell::Inner {
                        kind: ObjectType::Conveyor,
                        ..
                    }
                )) {
                    return Err(format!(
                        "Cannot place {:?} above {:?} at {:?}",
                        cell,
                        old_cell,
                        (x, y)
                    ));
                }
            }
        }

        // check that the new part's exgress does not touch a deposits ingress, unless it is a mine
        if object.kind() != ObjectType::Mine {
            for (x, y) in object.ingresses() {
                let neighbour_to_deposit = neighbours(x, y).iter().any(|coord| {
                    matches!(
                        self.get_cell(coord.0, coord.1),
                        Some(ObjectCell::Exgress {
                            kind: ObjectType::Deposit,
                            ..
                        })
                    )
                });
                if neighbour_to_deposit {
                    return Err(format!(
                        "Cannot place {:?} because its ingress touches a deposit's exgress",
                        object,
                    ));
                }
            }
        }

        // check that the new part's exgress does not touch multiple ingresses
        if object.kind() == ObjectType::Conveyor || object.kind() == ObjectType::Combiner {
            if let Some((x, y)) = object.exgress() {
                let num_neighbouring_ingresses = neighbours(x, y)
                    .iter()
                    .filter(|coord| {
                        matches!(
                            self.get_cell(coord.0, coord.1),
                            Some(ObjectCell::Ingress { .. })
                        )
                    })
                    .count();

                if num_neighbouring_ingresses >= 2 {
                    return Err(format!(
                        "Cannot place {:?} because its exgress touches multiple ingress",
                        object,
                    ));
                }
            }
        }

        // check that the new part does not touch an exgress (w/ its ingress), that is already
        // connected to another ingress
        for (x, y) in object.ingresses() {
            let neighbouring_exgresses = neighbours(x, y).into_iter().filter(|coord| {
                matches!(
                    self.get_cell(coord.0, coord.1),
                    Some(ObjectCell::Exgress { .. })
                )
            });

            for exgress in neighbouring_exgresses {
                let num_neighbouring_ingresses = neighbours(exgress.0, exgress.1)
                    .iter()
                    .filter(|coord| {
                        matches!(
                            self.get_cell(coord.0, coord.1),
                            Some(ObjectCell::Ingress { .. })
                        )
                    })
                    .count();

                if num_neighbouring_ingresses >= 1 {
                    return Err(format!(
                        "Cannot place {:?} because its exgress touches an ingress that is already connected to another exgress",
                        object,
                    ));
                }
            }
        }

        for ((x, y), cell) in cells {
            self.map.insert((x, y), cell);
        }

        self.objects.push(MapObject::from((index, object)));

        Ok(index)
    }
}

impl Display for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.map.is_empty() {
            f.write_str("Empty map")?;
            return Ok(());
        }

        f.write_str("   ")?;
        for i in 0..self.width() {
            f.write_fmt(format_args!("{}", i / 10))?;
        }
        f.write_str("\n   ")?;

        for i in 0..self.width() {
            f.write_fmt(format_args!("{}", i % 10))?;
        }
        f.write_str("\n")?;

        let width = self.width() as i32;
        let height = self.height() as i32;
        for y in 0..height {
            f.write_fmt(format_args!("{:0>2} ", y))?;
            for x in 0..width {
                let c = self.map.get(&(x, y)).map(|cell| cell.into()).unwrap_or('.');
                f.write_fmt(format_args!("{}", c))?;
            }
            f.write_str("\n")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_pieces_can_be_placed_on_empty_map() {
        let map = Map::new(10, 10, vec![]);

        {
            let mut map = map.clone();
            let result = map.insert_object(Object::Obstacle {
                x: 3,
                y: 3,
                width: 3,
                height: 3,
            });
            assert!(result.is_ok());
        }

        {
            let mut map = map.clone();
            let result = map.insert_object(Object::Deposit {
                x: 3,
                y: 3,
                width: 3,
                height: 3,
                subtype: 0,
            });
            assert!(result.is_ok());
        }

        {
            let mut map = map.clone();
            let result = map.insert_object(Object::Factory {
                x: 3,
                y: 3,
                subtype: 0,
            });
            assert!(result.is_ok());
        }

        for subtype in 0..=3 {
            let mut map = map.clone();
            let result = map.insert_object(Object::Mine {
                x: 3,
                y: 3,
                subtype,
            });
            assert!(result.is_ok());
        }

        for subtype in 0..=7 {
            let mut map = map.clone();
            let result = map.insert_object(Object::Conveyor {
                x: 3,
                y: 3,
                subtype,
            });
            assert!(result.is_ok());
        }

        for subtype in 0..=3 {
            let mut map = map.clone();
            let result = map.insert_object(Object::Combiner {
                x: 3,
                y: 3,
                subtype,
            });
            assert!(result.is_ok());
        }
    }

    #[test]
    fn no_piece_can_be_placed_on_occupied_map() {
        let map = Map::new(
            10,
            10,
            vec![Object::Obstacle {
                x: 0,
                y: 0,
                width: 10,
                height: 10,
            }],
        );

        {
            let mut map = map.clone();
            let result = map.insert_object(Object::Obstacle {
                x: 3,
                y: 3,
                width: 3,
                height: 3,
            });
            assert!(result.is_err());
        }

        {
            let mut map = map.clone();
            let result = map.insert_object(Object::Deposit {
                x: 3,
                y: 3,
                width: 3,
                height: 3,
                subtype: 0,
            });
            assert!(result.is_err());
        }

        {
            let mut map = map.clone();
            let result = map.insert_object(Object::Factory {
                x: 3,
                y: 3,
                subtype: 0,
            });
            assert!(result.is_err());
        }

        for subtype in 0..=3 {
            let mut map = map.clone();
            let result = map.insert_object(Object::Mine {
                x: 3,
                y: 3,
                subtype,
            });
            assert!(result.is_err());
        }

        for subtype in 0..=7 {
            let mut map = map.clone();
            let result = map.insert_object(Object::Conveyor {
                x: 3,
                y: 3,
                subtype,
            });
            assert!(result.is_err());
        }

        for subtype in 0..=3 {
            let mut map = map.clone();
            let result = map.insert_object(Object::Combiner {
                x: 3,
                y: 3,
                subtype,
            });
            assert!(result.is_err());
        }
    }

    #[test]
    fn no_piece_can_be_placed_outside_map() {
        let map = Map::new(10, 10, vec![]);

        for (x, y) in [(-1, 0), (0, -1), (20, 0), (0, 20)] {
            {
                let mut map = map.clone();
                let result = map.insert_object(Object::Obstacle {
                    x,
                    y,
                    width: 3,
                    height: 3,
                });
                assert!(result.is_err());
            }

            {
                let mut map = map.clone();
                let result = map.insert_object(Object::Deposit {
                    x,
                    y,
                    width: 3,
                    height: 3,
                    subtype: 0,
                });
                assert!(result.is_err());
            }

            {
                let mut map = map.clone();
                let result = map.insert_object(Object::Factory { x, y, subtype: 0 });
                assert!(result.is_err());
            }

            for subtype in 0..=3 {
                let mut map = map.clone();
                let result = map.insert_object(Object::Mine { x, y, subtype });
                assert!(result.is_err());
            }

            for subtype in 0..=7 {
                let mut map = map.clone();
                let result = map.insert_object(Object::Conveyor { x, y, subtype });
                assert!(result.is_err());
            }

            for subtype in 0..=3 {
                let mut map = map.clone();
                let result = map.insert_object(Object::Combiner { x, y, subtype });
                assert!(result.is_err());
            }
        }
    }

    // TODO: test special rules
}
