use std::fmt::Display;

use fxhash::FxHashMap as HashMap;

use crate::{
    coord::{neighbours, Point},
    object::{Coord, Object, ObjectCell, ObjectID, ObjectType},
    task::Task,
};

#[derive(Debug, Clone)]
pub struct Map {
    width: u8,
    height: u8,
    map: HashMap<Point, ObjectCell>,
    objects: HashMap<ObjectID, Object>, //TODO: try (and measure) turning this into hashset
}

impl Map {
    pub fn new(width: u8, height: u8, objects: Vec<Object>) -> Self {
        // TODO: assert width <= 100 && height <= 100 in debug mode
        let mut map = Map {
            width,
            height,
            objects: HashMap::default(),
            map: HashMap::default(),
        };

        for object in objects {
            if let Err(e) = map.insert_object(object) {
                panic!("Cannot create map from objects: '{}'", e);
            }
        }

        map
    }

    pub fn get_object(&self, id: ObjectID) -> &Object {
        &self.objects[&id]
    }

    pub fn get_objects(&self) -> impl Iterator<Item = &Object> {
        self.objects.values()
    }

    pub fn get_cell(&self, x: Coord, y: Coord) -> Option<&ObjectCell> {
        self.map.get(&(x, y))
    }

    pub fn is_empty_at(&self, x: Coord, y: Coord) -> bool {
        x >= 0
            && y >= 0
            && x < self.width as Coord
            && y < self.width as Coord
            && self.get_cell(x, y).is_none()
    }

    pub fn width(&self) -> u8 {
        self.width
    }

    pub fn height(&self) -> u8 {
        self.height
    }

    pub fn insert_object(&mut self, object: Object) -> Result<(), String> {
        if self.objects.contains_key(&object.id()) {
            return Ok(());
        }

        let index = self.objects.len();
        self.can_insert_object(&object)?;

        let cells = object.get_cells(index);
        for ((x, y), cell) in cells {
            self.map.insert((x, y), cell);
        }

        self.objects.insert(object.id(), object);

        Ok(())
    }

    pub fn insert_object_unchecked(&mut self, object: Object) -> Result<(), String> {
        if self.objects.contains_key(&object.id()) {
            return Ok(());
        }

        let result = self.can_insert_object(&object);

        let index = self.objects.len();

        let cells = object.get_cells(index);
        for ((x, y), cell) in cells {
            self.map.insert((x, y), cell);
        }

        self.objects.insert(object.id(), object);

        result
    }

    pub fn try_insert_objects(&mut self, objects: Vec<Object>) -> Result<(), String> {
        let mut inserted = 0;
        for object in objects.iter() {
            match self.insert_object(object.clone()) {
                Ok(_) => {
                    inserted += 1;
                }
                Err(e) => {
                    for object in objects.iter().take(inserted) {
                        self.remove_object(object)?;
                    }
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    pub fn remove_object(&mut self, object: &Object) -> Result<(), String> {
        if self.objects.remove(&object.id()).is_none() {
            return Err(String::from(
                "Cannot remove object. Map does not contain such object.",
            ));
        }

        for (point, _) in object.get_cells(0) {
            self.map.remove(&point);
        }

        Ok(())
    }

    pub fn force_remove_object(&mut self, object: &Object) {
        if self.objects.remove(&object.id()).is_some() {
            for (point, _) in object.get_cells(0) {
                self.map.remove(&point);
            }
        }
    }

    pub fn can_insert_objects(&mut self, objects: Vec<&Object>) -> Result<(), String> {
        let mut result = Ok(());
        for object in objects.iter() {
            if let Err(e) = self.insert_object((*object).clone()) {
                result = Err(e);
                break;
            }
        }

        for object in objects {
            self.force_remove_object(object);
        }

        result
    }

    pub fn can_insert_object(&self, object: &Object) -> Result<(), String> {
        if self.objects.contains_key(&object.id()) {
            return Ok(());
        }

        let width = self.width();
        let height = self.height();
        let index = self.objects.len();

        // check that no part of object is outside map or placed over another building
        let cells = object.get_cells(index);
        for ((x, y), cell) in cells.iter() {
            if *x < 0 || *y < 0 || *x >= width as Coord || *y >= height as Coord {
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
        if object.kind() == ObjectType::Conveyor
            || object.kind() == ObjectType::Combiner
            || object.kind() == ObjectType::Mine
        {
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
                        "Cannot place {:?} because its ingress touches an exgress that is already connected to another ingress",
                        object,
                    ));
                }
            }
        }

        Ok(())
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

        let width = self.width() as Coord;
        let height = self.height() as Coord;
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

impl From<&Task> for Map {
    fn from(task: &Task) -> Self {
        let objects = task.objects.iter().cloned().map(Object::from).collect();
        Map::new(task.width, task.height, objects)
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

    #[test]
    fn piece_can_be_placed_over_itself() {
        let map = Map::new(10, 10, vec![]);
        let objects = vec![
            Object::Obstacle {
                x: 3,
                y: 3,
                width: 3,
                height: 3,
            },
            Object::Deposit {
                x: 3,
                y: 3,
                width: 3,
                height: 3,
                subtype: 0,
            },
            Object::Factory {
                x: 3,
                y: 3,
                subtype: 0,
            },
            Object::Mine {
                x: 3,
                y: 3,
                subtype: 0,
            },
            Object::Conveyor {
                x: 3,
                y: 3,
                subtype: 0,
            },
            Object::Combiner {
                x: 3,
                y: 3,
                subtype: 0,
            },
        ];

        for object in objects {
            let mut map = map.clone();
            map.insert_object(object.clone()).unwrap();
            let result = map.insert_object(object);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn no_piece_but_mine_can_touch_deposit_with_ingress() {
        let map = Map::new(
            10,
            10,
            vec![Object::Deposit {
                x: 0,
                y: 0,
                width: 1,
                height: 1,
                subtype: 0,
            }],
        );

        {
            let map = map.clone();
            let result = map.can_insert_object(&Object::Mine {
                x: 1,
                y: 0,
                subtype: 0,
            });
            assert!(result.is_ok());
        }

        let objects = vec![
            Object::Factory {
                x: 1,
                y: 0,
                subtype: 0,
            },
            Object::Conveyor {
                x: 2,
                y: 0,
                subtype: 0,
            },
            Object::Combiner {
                x: 2,
                y: 1,
                subtype: 0,
            },
        ];

        for object in objects {
            let map = map.clone();
            let result = map.can_insert_object(&object);
            assert!(result.is_err());
        }
    }

    #[test]
    fn no_pieces_exgress_can_touch_multiple_ingresses() {
        let map = Map::new(
            10,
            10,
            vec![
                Object::Conveyor {
                    x: 6,
                    y: 3,
                    subtype: 0,
                },
                Object::Conveyor {
                    x: 6,
                    y: 5,
                    subtype: 0,
                },
            ],
        );

        let objects = vec![
            Object::Mine {
                x: 3,
                y: 3,
                subtype: 0,
            },
            Object::Conveyor {
                x: 4,
                y: 4,
                subtype: 0,
            },
            Object::Combiner {
                x: 4,
                y: 4,
                subtype: 0,
            },
        ];

        for object in objects {
            let map = map.clone();
            let result = map.can_insert_object(&object);
            assert!(result.is_err());
        }
    }

    #[test]
    fn no_pieces_ingress_can_touch_already_connected_exgress() {
        let map = Map::new(
            10,
            10,
            vec![
                Object::Conveyor {
                    x: 4,
                    y: 4,
                    subtype: 0,
                },
                Object::Conveyor {
                    x: 6,
                    y: 5,
                    subtype: 0,
                },
            ],
        );

        let objects = vec![
            Object::Mine {
                x: 6,
                y: 2,
                subtype: 0,
            },
            Object::Conveyor {
                x: 6,
                y: 3,
                subtype: 0,
            },
            Object::Combiner {
                x: 6,
                y: 2,
                subtype: 0,
            },
        ];

        for object in objects {
            let map = map.clone();
            let result = map.can_insert_object(&object);
            assert!(result.is_err());
        }
    }
}
