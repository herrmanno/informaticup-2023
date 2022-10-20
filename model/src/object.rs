use core::panic;

use crate::{coord::Point, solution, task};

/// Object's x or y
/// TODO: change to u8 and handle subtractions
pub type Coord = i8;

/// Object's width or height
pub type Length = u8;

/// Object's subtype
pub type Subtype = u8;

/// Object type (8 bits) + object subtype (8 bits) + x (8 bits) + y (8 bits) + width (8 bits) + height (8 bits)
pub type ObjectID = u64;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Object {
    Obstacle {
        x: Coord,
        y: Coord,
        width: Length,
        height: Length,
    },
    Deposit {
        x: Coord,
        y: Coord,
        width: Length,
        height: Length,
        subtype: Subtype,
    },
    Mine {
        x: Coord,
        y: Coord,
        subtype: Subtype,
    },
    Factory {
        x: Coord,
        y: Coord,
        subtype: Subtype,
    },
    Conveyor {
        x: Coord,
        y: Coord,
        subtype: Subtype,
    },
    Combiner {
        x: Coord,
        y: Coord,
        subtype: Subtype,
    },
}

impl Object {
    pub fn mine_with_subtype_and_exgress_at(subtype: u8, exgress_position: Point) -> Object {
        let (x, y) = exgress_position;
        match subtype {
            0 => Object::Mine {
                x: x - 2,
                y: y - 1,
                subtype,
            },
            1 => Object::Mine {
                x,
                y: y - 2,
                subtype,
            },
            2 => Object::Mine {
                x: x + 1,
                y,
                subtype,
            },
            3 => Object::Mine {
                x: x - 1,
                y: y + 1,
                subtype,
            },
            _ => panic!("Invalid mine subtype {}", subtype),
        }
    }

    pub fn conveyor_with_subtype_and_exgress_at(subtype: u8, exgress_position: Point) -> Object {
        let (x, y) = exgress_position;
        match subtype {
            0 => Object::Conveyor {
                x: x - 1,
                y,
                subtype,
            },
            1 => Object::Conveyor {
                x,
                y: y - 1,
                subtype,
            },
            2 => Object::Conveyor {
                x: x + 1,
                y,
                subtype,
            },
            3 => Object::Conveyor {
                x,
                y: y + 1,
                subtype,
            },
            4 => Object::Conveyor {
                x: x - 2,
                y,
                subtype,
            },
            5 => Object::Conveyor {
                x,
                y: y - 2,
                subtype,
            },
            6 => Object::Conveyor {
                x: x + 1,
                y,
                subtype,
            },
            7 => Object::Conveyor {
                x,
                y: y + 1,
                subtype,
            },
            _ => panic!("Invalid conveyor subtype {}", subtype),
        }
    }

    pub fn combiner_with_subtype_and_exgress_at(subtype: u8, exgress_position: Point) -> Object {
        let (x, y) = exgress_position;
        match subtype {
            0 => Object::Combiner {
                x: x - 1,
                y,
                subtype,
            },
            1 => Object::Combiner {
                x,
                y: y - 1,
                subtype,
            },
            2 => Object::Combiner {
                x: x + 1,
                y,
                subtype,
            },
            3 => Object::Combiner {
                x,
                y: y + 1,
                subtype,
            },
            _ => panic!("Invalid combiner subtype {}", subtype),
        }
    }

    /// Calculate a unique id based on this object's values
    pub fn id(&self) -> ObjectID {
        // TODO: benchmark against pre-calculating and storing id
        let kind = match self {
            Object::Obstacle { .. } => 0,
            Object::Deposit { .. } => 1,
            Object::Mine { .. } => 2,
            Object::Factory { .. } => 3,
            Object::Conveyor { .. } => 4,
            Object::Combiner { .. } => 5,
        };

        let subtype = self.subtype().unwrap_or(0);
        let (x, y) = self.coords();
        let width = self.width().unwrap_or(0);
        let height = self.height().unwrap_or(0);

        ((kind as u64) << 48)
            + ((subtype as u64) << 40)
            + ((x as u64) << 32)
            + ((y as u64) << 16)
            + ((width as u64) << 8)
            + (height as u64)
    }

    pub fn coords(&self) -> Point {
        match self {
            Object::Obstacle { x, y, .. } => (*x, *y),
            Object::Deposit { x, y, .. } => (*x, *y),
            Object::Mine { x, y, .. } => (*x, *y),
            Object::Factory { x, y, .. } => (*x, *y),
            Object::Conveyor { x, y, .. } => (*x, *y),
            Object::Combiner { x, y, .. } => (*x, *y),
        }
    }

    pub fn width(&self) -> Option<Length> {
        match self {
            Object::Obstacle { width, .. } => Some(*width),
            Object::Deposit { width, .. } => Some(*width),
            _ => None,
        }
    }

    pub fn height(&self) -> Option<Length> {
        match self {
            Object::Obstacle { height, .. } => Some(*height),
            Object::Deposit { height, .. } => Some(*height),
            _ => None,
        }
    }

    pub fn kind(&self) -> ObjectType {
        match self {
            Object::Obstacle { .. } => ObjectType::Obstacle,
            Object::Deposit { .. } => ObjectType::Deposit,
            Object::Mine { .. } => ObjectType::Mine,
            Object::Factory { .. } => ObjectType::Factory,
            Object::Conveyor { .. } => ObjectType::Conveyor,
            Object::Combiner { .. } => ObjectType::Combiner,
        }
    }

    pub fn subtype(&self) -> Option<u8> {
        match self {
            Object::Obstacle { .. } => None,
            Object::Deposit { subtype, .. } => Some(*subtype),
            Object::Mine { subtype, .. } => Some(*subtype),
            Object::Factory { subtype, .. } => Some(*subtype),
            Object::Conveyor { subtype, .. } => Some(*subtype),
            Object::Combiner { subtype, .. } => Some(*subtype),
        }
    }

    pub fn ingress(&self) -> Option<Point> {
        match self {
            Object::Mine { x, y, subtype: 0 } => Some((x - 1, y + 1)),
            Object::Mine { x, y, subtype: 1 } => Some((*x, y - 1)),
            Object::Mine { x, y, subtype: 2 } => Some((x + 2, *y)),
            Object::Mine { x, y, subtype: 3 } => Some((x + 1, y + 2)),

            Object::Conveyor { x, y, subtype: 0 } => Some((x - 1, *y)),
            Object::Conveyor { x, y, subtype: 1 } => Some((*x, y - 1)),
            Object::Conveyor { x, y, subtype: 2 } => Some((x + 1, *y)),
            Object::Conveyor { x, y, subtype: 3 } => Some((*x, y + 1)),
            Object::Conveyor { x, y, subtype: 4 } => Some((x - 1, *y)),
            Object::Conveyor { x, y, subtype: 5 } => Some((*x, y - 1)),
            Object::Conveyor { x, y, subtype: 6 } => Some((x + 2, *y)),
            Object::Conveyor { x, y, subtype: 7 } => Some((*x, y + 2)),

            Object::Deposit { .. } => None,

            Object::Obstacle { .. } => None,

            _ => todo!(),
        }
    }

    pub fn ingresses(&self) -> Vec<Point> {
        match self {
            Object::Combiner { x, y, subtype: 0 } => {
                vec![(x - 1, y - 1), (x - 1, *y), (x - 1, y + 1)]
            }
            Object::Combiner { x, y, subtype: 1 } => {
                vec![(x - 1, y - 1), (*x, y - 1), (x + 1, y - 1)]
            }
            Object::Combiner { x, y, subtype: 2 } => {
                vec![(x + 1, y - 1), (x + 1, *y), (x + 1, y + 1)]
            }
            Object::Combiner { x, y, subtype: 3 } => {
                vec![(x - 1, y + 1), (*x, y + 1), (x + 1, y + 1)]
            }

            Object::Factory { x, y, .. } => {
                let mut ingresses = Vec::with_capacity(18);
                for dx in (*x)..(*x + 5) {
                    ingresses.push((dx, *y));
                    ingresses.push((dx, *y + 4));
                }
                for dy in (*y + 1)..(*y + 4) {
                    ingresses.push((*x, dy));
                    ingresses.push((*x + 4, dy));
                }
                ingresses
            }

            Object::Deposit { .. } => vec![],

            Object::Obstacle { .. } => vec![],

            _ => self.ingress().into_iter().collect(),
        }
    }

    pub fn exgress(&self) -> Option<Point> {
        match self {
            Object::Mine { x, y, subtype: 0 } => Some((x + 2, y + 1)),
            Object::Mine { x, y, subtype: 1 } => Some((*x, y + 2)),
            Object::Mine { x, y, subtype: 2 } => Some((x - 1, *y)),
            Object::Mine { x, y, subtype: 3 } => Some((x + 1, y - 1)),

            Object::Conveyor { x, y, subtype: 0 } => Some((x + 1, *y)),
            Object::Conveyor { x, y, subtype: 1 } => Some((*x, y + 1)),
            Object::Conveyor { x, y, subtype: 2 } => Some((x - 1, *y)),
            Object::Conveyor { x, y, subtype: 3 } => Some((*x, y - 1)),
            Object::Conveyor { x, y, subtype: 4 } => Some((x + 2, *y)),
            Object::Conveyor { x, y, subtype: 5 } => Some((*x, y + 2)),
            Object::Conveyor { x, y, subtype: 6 } => Some((x - 1, *y)),
            Object::Conveyor { x, y, subtype: 7 } => Some((*x, y - 1)),

            Object::Combiner { x, y, subtype: 0 } => Some((x + 1, *y)),
            Object::Combiner { x, y, subtype: 1 } => Some((*x, y + 1)),
            Object::Combiner { x, y, subtype: 2 } => Some((x - 1, *y)),
            Object::Combiner { x, y, subtype: 3 } => Some((*x, y - 1)),

            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectType {
    Obstacle,
    Deposit,
    Mine,
    Factory,
    Conveyor,
    Combiner,
}

#[derive(Debug, Clone)]
pub enum ObjectCell {
    Exgress {
        kind: ObjectType,
        index: usize,
    },
    Ingress {
        kind: ObjectType,
        index: usize,
    },
    Inner {
        kind: ObjectType,
        subtype: Option<u8>,
    },
}

impl From<&ObjectCell> for char {
    fn from(cell: &ObjectCell) -> char {
        match cell {
            ObjectCell::Exgress { .. } => '-',
            ObjectCell::Ingress { .. } => '+',
            ObjectCell::Inner {
                kind: ObjectType::Obstacle,
                ..
            } => 'X',
            ObjectCell::Inner {
                kind: ObjectType::Factory,
                subtype: Some(st),
            } => char::from_digit(*st as u32, 10).unwrap(),
            ObjectCell::Inner {
                kind: ObjectType::Deposit,
                subtype: Some(st),
            } => char::from_digit(*st as u32, 10).unwrap(),
            ObjectCell::Inner { .. } => 'O',
        }
    }
}

impl Object {
    /// Calculates the fields occupied by this object
    pub fn get_cells(&self, index: usize) -> Vec<(Point, ObjectCell)> {
        use Object::*;
        use ObjectCell::*;

        match *self {
            Obstacle {
                x,
                y,
                width,
                height,
            } => {
                let mut cells = Vec::new();
                for px in x..(x + width as Coord) {
                    for py in y..(y + height as Coord) {
                        cells.push((
                            (px, py),
                            Inner {
                                kind: ObjectType::Obstacle,
                                subtype: None,
                            },
                        ));
                    }
                }
                cells
            }
            Deposit {
                x,
                y,
                width,
                height,
                subtype,
            } => {
                let mut cells = Vec::with_capacity(25);
                for px in x..(x + width as Coord) {
                    for py in y..(y + height as Coord) {
                        if px == x
                            || px == (x + width as Coord - 1)
                            || py == y
                            || py == (y + height as Coord - 1)
                        {
                            cells.push((
                                (px, py),
                                Exgress {
                                    kind: ObjectType::Deposit,
                                    index,
                                },
                            ));
                        } else {
                            cells.push((
                                (px, py),
                                Inner {
                                    kind: ObjectType::Deposit,
                                    subtype: Some(subtype),
                                },
                            ));
                        };
                    }
                }
                cells
            }
            Factory { x, y, subtype } => {
                let mut cells = Vec::with_capacity(25);
                for px in x..(x + 5) {
                    for py in y..(y + 5) {
                        if px == x || px == (x + 4) || py == y || py == (y + 4) {
                            cells.push((
                                (px, py),
                                Ingress {
                                    kind: ObjectType::Factory,
                                    index,
                                },
                            ));
                        } else {
                            cells.push((
                                (px, py),
                                Inner {
                                    kind: ObjectType::Factory,
                                    subtype: Some(subtype),
                                },
                            ));
                        };
                    }
                }
                cells
            }
            Mine { x, y, subtype } => {
                if subtype == 0 {
                    vec![
                        (
                            (x, y),
                            Inner {
                                kind: ObjectType::Combiner,
                                subtype: Some(subtype),
                            },
                        ),
                        (
                            (x + 1, y),
                            Inner {
                                kind: ObjectType::Combiner,
                                subtype: Some(subtype),
                            },
                        ),
                        (
                            (x, y + 1),
                            Inner {
                                kind: ObjectType::Combiner,
                                subtype: Some(subtype),
                            },
                        ),
                        (
                            (x + 1, y + 1),
                            Inner {
                                kind: ObjectType::Combiner,
                                subtype: Some(subtype),
                            },
                        ),
                        (
                            (x - 1, y + 1),
                            Ingress {
                                kind: ObjectType::Mine,
                                index,
                            },
                        ),
                        (
                            (x + 2, y + 1),
                            Exgress {
                                kind: ObjectType::Mine,
                                index,
                            },
                        ),
                    ]
                } else if subtype == 1 {
                    vec![
                        (
                            (x, y),
                            Inner {
                                kind: ObjectType::Combiner,
                                subtype: Some(subtype),
                            },
                        ),
                        (
                            (x + 1, y),
                            Inner {
                                kind: ObjectType::Combiner,
                                subtype: Some(subtype),
                            },
                        ),
                        (
                            (x, y + 1),
                            Inner {
                                kind: ObjectType::Combiner,
                                subtype: Some(subtype),
                            },
                        ),
                        (
                            (x + 1, y + 1),
                            Inner {
                                kind: ObjectType::Combiner,
                                subtype: Some(subtype),
                            },
                        ),
                        (
                            (x, y - 1),
                            Ingress {
                                kind: ObjectType::Mine,
                                index,
                            },
                        ),
                        (
                            (x, y + 2),
                            Exgress {
                                kind: ObjectType::Mine,
                                index,
                            },
                        ),
                    ]
                } else if subtype == 2 {
                    vec![
                        (
                            (x, y),
                            Inner {
                                kind: ObjectType::Combiner,
                                subtype: Some(subtype),
                            },
                        ),
                        (
                            (x + 1, y),
                            Inner {
                                kind: ObjectType::Combiner,
                                subtype: Some(subtype),
                            },
                        ),
                        (
                            (x, y + 1),
                            Inner {
                                kind: ObjectType::Combiner,
                                subtype: Some(subtype),
                            },
                        ),
                        (
                            (x + 1, y + 1),
                            Inner {
                                kind: ObjectType::Combiner,
                                subtype: Some(subtype),
                            },
                        ),
                        (
                            (x - 1, y),
                            Exgress {
                                kind: ObjectType::Mine,
                                index,
                            },
                        ),
                        (
                            (x + 2, y),
                            Ingress {
                                kind: ObjectType::Mine,
                                index,
                            },
                        ),
                    ]
                } else if subtype == 3 {
                    vec![
                        (
                            (x, y),
                            Inner {
                                kind: ObjectType::Combiner,
                                subtype: Some(subtype),
                            },
                        ),
                        (
                            (x + 1, y),
                            Inner {
                                kind: ObjectType::Combiner,
                                subtype: Some(subtype),
                            },
                        ),
                        (
                            (x, y + 1),
                            Inner {
                                kind: ObjectType::Combiner,
                                subtype: Some(subtype),
                            },
                        ),
                        (
                            (x + 1, y + 1),
                            Inner {
                                kind: ObjectType::Combiner,
                                subtype: Some(subtype),
                            },
                        ),
                        (
                            (x + 1, y - 1),
                            Exgress {
                                kind: ObjectType::Mine,
                                index,
                            },
                        ),
                        (
                            (x + 1, y + 2),
                            Ingress {
                                kind: ObjectType::Mine,
                                index,
                            },
                        ),
                    ]
                } else {
                    panic!("Invalid mine subtype: {}", subtype);
                }
            }
            Combiner { x, y, subtype } => {
                let mut points = vec![
                    (
                        (0, 0),
                        Inner {
                            kind: ObjectType::Combiner,
                            subtype: Some(subtype),
                        },
                    ), // root cell
                    (
                        (-1, -1),
                        Ingress {
                            kind: ObjectType::Combiner,
                            index,
                        },
                    ),
                    (
                        (-1, 0),
                        Ingress {
                            kind: ObjectType::Combiner,
                            index,
                        },
                    ),
                    (
                        (-1, 1),
                        Ingress {
                            kind: ObjectType::Combiner,
                            index,
                        },
                    ),
                    (
                        (0, -1),
                        Inner {
                            kind: ObjectType::Combiner,
                            subtype: Some(subtype),
                        },
                    ),
                    (
                        (0, 1),
                        Inner {
                            kind: ObjectType::Combiner,
                            subtype: Some(subtype),
                        },
                    ),
                    (
                        (1, 0),
                        Exgress {
                            kind: ObjectType::Combiner,
                            index,
                        },
                    ),
                ];

                for _ in 0..subtype {
                    for ((x, y), _) in points.iter_mut() {
                        let tmp = *y;
                        *y = *x;
                        *x = -tmp;
                    }
                }

                points
                    .into_iter()
                    .map(|((dx, dy), cell)| (((x as Coord + dx), (y as Coord + dy)), cell))
                    .collect()
            }
            Conveyor { x, y, subtype } => {
                if subtype == 0 {
                    vec![
                        (
                            (x, y),
                            Inner {
                                kind: ObjectType::Combiner,
                                subtype: Some(subtype),
                            },
                        ),
                        (
                            (x - 1, y),
                            Ingress {
                                kind: ObjectType::Conveyor,
                                index,
                            },
                        ),
                        (
                            (x + 1, y),
                            Exgress {
                                kind: ObjectType::Conveyor,
                                index,
                            },
                        ),
                    ]
                } else if subtype == 1 {
                    vec![
                        (
                            (x, y),
                            Inner {
                                kind: ObjectType::Combiner,
                                subtype: Some(subtype),
                            },
                        ),
                        (
                            (x, y - 1),
                            Ingress {
                                kind: ObjectType::Conveyor,
                                index,
                            },
                        ),
                        (
                            (x, y + 1),
                            Exgress {
                                kind: ObjectType::Conveyor,
                                index,
                            },
                        ),
                    ]
                } else if subtype == 2 {
                    vec![
                        (
                            (x, y),
                            Inner {
                                kind: ObjectType::Combiner,
                                subtype: Some(subtype),
                            },
                        ),
                        (
                            (x - 1, y),
                            Exgress {
                                kind: ObjectType::Conveyor,
                                index,
                            },
                        ),
                        (
                            (x + 1, y),
                            Ingress {
                                kind: ObjectType::Conveyor,
                                index,
                            },
                        ),
                    ]
                } else if subtype == 3 {
                    vec![
                        (
                            (x, y),
                            Inner {
                                kind: ObjectType::Combiner,
                                subtype: Some(subtype),
                            },
                        ),
                        (
                            (x, y - 1),
                            Exgress {
                                kind: ObjectType::Conveyor,
                                index,
                            },
                        ),
                        (
                            (x, y + 1),
                            Ingress {
                                kind: ObjectType::Conveyor,
                                index,
                            },
                        ),
                    ]
                } else if subtype == 4 {
                    vec![
                        (
                            (x, y),
                            Inner {
                                kind: ObjectType::Combiner,
                                subtype: Some(subtype),
                            },
                        ),
                        (
                            (x + 1, y),
                            Inner {
                                kind: ObjectType::Combiner,
                                subtype: Some(subtype),
                            },
                        ),
                        (
                            (x - 1, y),
                            Ingress {
                                kind: ObjectType::Conveyor,
                                index,
                            },
                        ),
                        (
                            (x + 2, y),
                            Exgress {
                                kind: ObjectType::Conveyor,
                                index,
                            },
                        ),
                    ]
                } else if subtype == 5 {
                    vec![
                        (
                            (x, y),
                            Inner {
                                kind: ObjectType::Combiner,
                                subtype: Some(subtype),
                            },
                        ),
                        (
                            (x, y + 1),
                            Inner {
                                kind: ObjectType::Combiner,
                                subtype: Some(subtype),
                            },
                        ),
                        (
                            (x, y - 1),
                            Ingress {
                                kind: ObjectType::Conveyor,
                                index,
                            },
                        ),
                        (
                            (x, y + 2),
                            Exgress {
                                kind: ObjectType::Conveyor,
                                index,
                            },
                        ),
                    ]
                } else if subtype == 6 {
                    vec![
                        (
                            (x, y),
                            Inner {
                                kind: ObjectType::Combiner,
                                subtype: Some(subtype),
                            },
                        ),
                        (
                            (x + 1, y),
                            Inner {
                                kind: ObjectType::Combiner,
                                subtype: Some(subtype),
                            },
                        ),
                        (
                            (x - 1, y),
                            Exgress {
                                kind: ObjectType::Conveyor,
                                index,
                            },
                        ),
                        (
                            (x + 2, y),
                            Ingress {
                                kind: ObjectType::Conveyor,
                                index,
                            },
                        ),
                    ]
                } else if subtype == 7 {
                    vec![
                        (
                            (x, y),
                            Inner {
                                kind: ObjectType::Combiner,
                                subtype: Some(subtype),
                            },
                        ),
                        (
                            (x, y + 1),
                            Inner {
                                kind: ObjectType::Combiner,
                                subtype: Some(subtype),
                            },
                        ),
                        (
                            (x, y - 1),
                            Exgress {
                                kind: ObjectType::Conveyor,
                                index,
                            },
                        ),
                        (
                            (x, y + 2),
                            Ingress {
                                kind: ObjectType::Conveyor,
                                index,
                            },
                        ),
                    ]
                } else {
                    panic!("Invalid conveyor subtype: {}", subtype);
                }
            }
        }
    }
}

impl From<task::Object> for Object {
    fn from(obj: task::Object) -> Self {
        let task::Object {
            kind,
            subtype,
            x,
            y,
            width,
            height,
        } = obj;
        match kind.as_str() {
            "obstacle" => Object::Obstacle {
                x,
                y,
                width,
                height,
            },
            "deposit" => Object::Deposit {
                x,
                y,
                width,
                height,
                subtype: subtype.unwrap(),
            },
            _ => panic!(
                "Cannot convert task object w/ type '{}' into proper object",
                kind
            ),
        }
    }
}

impl From<solution::Object> for Object {
    fn from(obj: solution::Object) -> Self {
        let solution::Object {
            kind,
            subtype,
            x,
            y,
        } = obj;
        match kind.as_str() {
            "mine" => Object::Mine { x, y, subtype },
            "conveyor" => Object::Conveyor { x, y, subtype },
            "combiner" => Object::Combiner { x, y, subtype },
            "factory" => Object::Factory { x, y, subtype },
            _ => panic!(
                "Cannot convert solution object w/ type '{}' into proper object",
                kind
            ),
        }
    }
}
