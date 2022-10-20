use std::{borrow::Borrow, rc::Rc};

use model::{
    coord::Point,
    object::{Object, ObjectCell, ObjectType},
};

pub type PathID = u128;

#[derive(Debug, Clone)]
pub(crate) enum Path {
    End { ingresses: Vec<Point> },
    Segment { object: Object, tail: Rc<Path> },
}

/// A path of objects
impl Path {
    /// Creates an empty path with given ingresses as heads
    pub fn from_starting_points(starting_points: Vec<Point>) -> Self {
        Path::End {
            ingresses: starting_points,
        }
    }

    /// Appends an object to an existing path to create a new path
    ///
    /// Returns an error if the new object interferes with any object of the path
    pub fn append(object: Object, tail: &Rc<Path>) -> Result<Path, String> {
        match tail.check_object(&object) {
            Ok(_) => Ok(Path::Segment {
                object,
                tail: Rc::clone(tail),
            }),
            Err(e) => Err(e),
        }
    }

    /// Calculates a hash-like id for this path, based on its objects
    pub fn id(&self) -> PathID {
        let mut a = 0u64;
        let mut b = 0u64;
        let mut t = false;
        for object in self.objects() {
            if t {
                a ^= object.id();
            } else {
                b ^= object.id();
            }

            t ^= t;
        }

        ((a as u128) << 64) | (b as u128)
    }

    /// Returns all ingresses of the path's head
    pub fn heads(&self) -> Vec<Point> {
        match self {
            Path::End { ingresses } => ingresses.clone(),
            Path::Segment { object, .. } => object.ingresses(),
        }
    }

    /// Returns all ingresses along the path
    pub fn all_ingresses(&self) -> Vec<Point> {
        let mut ingresses = vec![];

        for object in self.objects() {
            for ingress in object.ingresses() {
                ingresses.push(ingress);
            }
        }

        ingresses
    }

    /// Check if new object may be added to path legally
    fn check_object(&self, object: &Object) -> Result<(), String> {
        let cells = object.get_cells(0);

        for obj in self.objects() {
            for ((x, y), cell) in cells.iter() {
                for ((dx, dy), dcell) in obj.get_cells(0) {
                    if *x == dx && *y == dy {
                        match (cell, dcell) {
                            (
                                ObjectCell::Inner {
                                    kind: ObjectType::Conveyor,
                                    ..
                                },
                                ObjectCell::Inner {
                                    kind: ObjectType::Conveyor,
                                    ..
                                },
                            ) => {}
                            _ => return Err(format!("Cannot place {:?} over {:?}", object, obj)),
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Return an Iterator over this path's objects
    pub fn objects(&self) -> impl Iterator<Item = &Object> {
        PathObjects { path: self }
    }
}

impl From<&Path> for Vec<Object> {
    fn from(mut path: &Path) -> Self {
        let mut v = vec![];

        loop {
            match path {
                Path::End { .. } => {
                    break;
                }
                Path::Segment { object, tail } => {
                    v.push(object.clone());
                    path = tail.borrow();
                }
            }
        }

        v
    }
}

impl From<Path> for Vec<Object> {
    fn from(mut path: Path) -> Self {
        let mut v = vec![];

        loop {
            match path {
                Path::End { .. } => {
                    break;
                }
                Path::Segment { object, tail } => {
                    v.push(object.clone());
                    path = Rc::try_unwrap(tail).expect(
                        "Cannot turn path into objects. Path is still (partially) referenced.",
                    )
                }
            }
        }

        v
    }
}

struct PathObjects<'a> {
    path: &'a Path,
}

impl<'a> Iterator for PathObjects<'a> {
    type Item = &'a Object;

    fn next(&mut self) -> Option<Self::Item> {
        match self.path {
            Path::End { .. } => None,
            Path::Segment { object, tail } => {
                self.path = tail;
                Some(object)
            }
        }
    }
}
