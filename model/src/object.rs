use core::panic;

use crate::{coord::Coord, solution, task};

#[derive(Debug, Clone)]
pub enum Object {
    Obstacle {
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    },
    Deposit {
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        subtype: u8,
    },
    Mine {
        x: i32,
        y: i32,
        subtype: u8,
    },
    Factory {
        x: i32,
        y: i32,
        subtype: u8,
    },
    Conveyor {
        x: i32,
        y: i32,
        subtype: u8,
    },
    Combiner {
        x: i32,
        y: i32,
        subtype: u8,
    },
}

impl Object {
    pub fn mine_with_subtype_and_exgress_at(subtype: u8, exgress_position: Coord) -> Object {
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

    pub fn conveyor_with_subtype_and_exgress_at(subtype: u8, exgress_position: Coord) -> Object {
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

    pub fn combiner_with_subtype_and_exgress_at(subtype: u8, exgress_position: Coord) -> Object {
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

    pub fn coords(&self) -> Coord {
        match self {
            Object::Obstacle { x, y, .. } => (*x, *y),
            Object::Deposit { x, y, .. } => (*x, *y),
            Object::Mine { x, y, .. } => (*x, *y),
            Object::Factory { x, y, .. } => (*x, *y),
            Object::Conveyor { x, y, .. } => (*x, *y),
            Object::Combiner { x, y, .. } => (*x, *y),
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

    pub fn ingress(&self) -> Option<Coord> {
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

    pub fn ingresses(&self) -> Vec<Coord> {
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

    pub fn exgress(&self) -> Option<Coord> {
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
    pub fn get_cells(&self, index: usize) -> Vec<(Coord, ObjectCell)> {
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
                for px in x..(x + width as i32) {
                    for py in y..(y + height as i32) {
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
                for px in x..(x + width as i32) {
                    for py in y..(y + height as i32) {
                        if px == x
                            || px == (x + width as i32 - 1)
                            || py == y
                            || py == (y + height as i32 - 1)
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
                    .map(|((dx, dy), cell)| (((x as i32 + dx), (y as i32 + dy)), cell))
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
            width,
            height,
        } = obj;
        match kind.as_str() {
            "deposit" => Object::Deposit {
                x,
                y,
                width: width.unwrap(),
                height: height.unwrap(),
                subtype,
            },
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
