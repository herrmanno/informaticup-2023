//! Representation of a single path, as constructed by [Paths]

use std::{borrow::Borrow, rc::Rc};

use model::{
    coord::Point,
    object::{Object, ObjectType},
};

pub type PathID = u128;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Path {
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

    /// Appends `object` to this path to create a new Path
    pub fn append(object: Object, tail: &Rc<Path>) -> Path {
        Path::Segment {
            object,
            tail: Rc::clone(tail),
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

    /// Returns all ingresses along the path, except start and final ingresses
    ///
    /// Effectively returns all ingresses where other paths can start from, where
    /// - a _final_ ingress belongs to a mine - no path can _start on a mine_
    /// - a _start_ ingress belongs to a factory - if you want to build paths from
    ///   a factory you should know its ingresses
    pub fn all_ingresses(&self) -> Vec<Point> {
        let mut ingresses = vec![];

        for object in self.objects() {
            if object.kind() != ObjectType::Mine {
                for ingress in object.ingresses() {
                    ingresses.push(ingress);
                }
            }
        }

        ingresses
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

/// An iterator of objects along a path
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
