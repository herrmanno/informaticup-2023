use std::collections::HashMap;

use crate::{task, solution};

#[derive(Debug)]
pub enum Object {
    Obstacle { x: u32, y: u32, width: u32, height: u32 },
    Deposit { x: u32, y: u32, width: u32, height: u32, subtype: u8 },
    Mine { x: u32, y: u32, subtype: u8 },
    Factory { x: u32, y: u32, subtype: u8 },
    Conveyor { x: u32, y: u32, subtype: u8 },
    Combiner { x: u32, y: u32, subtype: u8 },
}

#[derive(Debug, Clone)]
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
    // TODO: link cell with specific object by giving it a type ('factory', 'deposit') and
    //       an index into a list of objects of such type
    Exgress,
    Ingress,
    Inner { kind: ObjectType, subtype: Option<u8> },
}

impl From<&ObjectCell> for char {
    fn from(cell: &ObjectCell) -> char {
        match cell {
            ObjectCell::Exgress => '-',
            ObjectCell::Ingress => '+',
            ObjectCell::Inner { kind: ObjectType::Obstacle, .. } => 'X',
            ObjectCell::Inner { kind: ObjectType::Factory, subtype: Some(st) } =>
                char::from_digit(*st as u32, 10).unwrap(),
            ObjectCell::Inner { kind: ObjectType::Deposit, subtype: Some(st) } =>
                char::from_digit(*st as u32, 10).unwrap(),
            ObjectCell::Inner { .. } => 'O',
        }
    }
}

impl Object {
    pub fn place_on_map(&self, map: &mut HashMap<(u32, u32), ObjectCell>) {
        for ((x, y), cell) in self.get_cells() {
            if let Some(old_cell) = map.insert((x, y), cell.clone()) {
                match (old_cell.clone(), cell.clone()) {
                    (
                        ObjectCell::Inner { kind: ObjectType::Conveyor, .. },
                        ObjectCell::Inner { kind: ObjectType::Conveyor, .. }
                    ) => { }
                    _ => panic!("Cannot place {:?} above {:?} at {:?}", cell, old_cell, (x,y))
                }
            }
        }
    }

    /// Calculates the fields occupied by this object
    fn get_cells(&self) -> Vec<((u32, u32), ObjectCell)> {
        use Object::*;
        use ObjectCell::*;

        match *self {
            Obstacle { x, y, width, height } => {
                let mut cells = Vec::new();
                for px in x..(x + width) {
                    for py in y..(y + height) {
                        cells.push(((px, py), Inner { kind: ObjectType::Obstacle, subtype: None }));
                    }
                }
                cells
            }
            Deposit { x, y, width, height, subtype } => {
                let mut cells = Vec::with_capacity(25);
                for px in x..(x + width) {
                    for py in y..(y + height) {
                        if px == x || px == (x + width - 1) || py == y || py == (y + width - 1) {
                            cells.push(((px, py), Exgress));
                        } else {
                            cells.push(((px, py), Inner { kind: ObjectType::Deposit, subtype: Some(subtype) } ));
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
                            cells.push(((px, py), Ingress));
                        } else {
                            cells.push(((px, py), Inner { kind: ObjectType::Factory, subtype: Some(subtype) } ));
                        };
                    }
                }
                cells
            }
            Mine { x, y, subtype } => {
                if subtype == 0 {
                    vec![
                        ((x, y), Inner { kind: ObjectType::Combiner, subtype: Some(subtype) }),
                        ((x + 1, y), Inner { kind: ObjectType::Combiner, subtype: Some(subtype) }),
                        ((x, y + 1), Inner { kind: ObjectType::Combiner, subtype: Some(subtype) }),
                        ((x + 1, y + 1), Inner { kind: ObjectType::Combiner, subtype: Some(subtype) }),
                        ((x - 1, y + 1), Ingress),
                        ((x + 2, y + 1), Exgress)
                    ]
                } else if subtype == 1 {
                    vec![
                        ((x, y), Inner { kind: ObjectType::Combiner, subtype: Some(subtype) }),
                        ((x + 1, y), Inner { kind: ObjectType::Combiner, subtype: Some(subtype) }),
                        ((x, y + 1), Inner { kind: ObjectType::Combiner, subtype: Some(subtype) }),
                        ((x + 1, y + 1), Inner { kind: ObjectType::Combiner, subtype: Some(subtype) }),
                        ((x, y + 2), Ingress),
                        ((x, y - 1), Exgress)
                    ]
                } else if subtype == 2 {
                    vec![
                        ((x, y), Inner { kind: ObjectType::Combiner, subtype: Some(subtype) }),
                        ((x + 1, y), Inner { kind: ObjectType::Combiner, subtype: Some(subtype) }),
                        ((x, y + 1), Inner { kind: ObjectType::Combiner, subtype: Some(subtype) }),
                        ((x + 1, y + 1), Inner { kind: ObjectType::Combiner, subtype: Some(subtype) }),
                        ((x - 1, y), Exgress),
                        ((x + 2, y), Ingress)
                    ]
                } else if subtype == 3 {
                    vec![
                        ((x, y), Inner { kind: ObjectType::Combiner, subtype: Some(subtype) }),
                        ((x + 1, y), Inner { kind: ObjectType::Combiner, subtype: Some(subtype) }),
                        ((x, y + 1), Inner { kind: ObjectType::Combiner, subtype: Some(subtype) }),
                        ((x + 1, y + 1), Inner { kind: ObjectType::Combiner, subtype: Some(subtype) }),
                        ((x + 1, y - 1), Exgress),
                        ((x + 1, y + 2), Ingress)
                    ]
                } else {
                    panic!("Invalid mine subtype: {}", subtype);
                }
            }
            Combiner { x, y, subtype } => {
                let mut points = vec![
                    ((0, 0), Inner { kind: ObjectType::Combiner, subtype: Some(subtype) }), // root cell
                    ((-1, -1), Ingress),((-1, 0), Ingress),((-1, 1), Ingress),
                    ((0, - 1), Inner { kind: ObjectType::Combiner, subtype: Some(subtype) }),
                    ((0, 1), Inner { kind: ObjectType::Combiner, subtype: Some(subtype) }),
                    ((1, 0), Exgress)
                ];

                for _ in 0..subtype {
                    for ((x,y), _) in points.iter_mut() {
                        let tmp = *y;
                        *y = *x;
                        *x = -tmp;
                    }
                }

                points
                    .into_iter()
                    .map(|((dx, dy), cell)| {
                        ( ( (x as i32 + dx) as u32
                          ,(y as i32 + dy) as u32
                          )
                        , cell
                        )
                    })
                    .collect()
            }
            Conveyor { x, y, subtype } => {
                if subtype == 0 {
                    vec![
                        ((x, y), Inner { kind: ObjectType::Combiner, subtype: Some(subtype) }),
                        ((x - 1, y), Ingress),
                        ((x + 1, y), Exgress),
                    ]
                } else if subtype == 1 {
                    vec![
                        ((x, y), Inner { kind: ObjectType::Combiner, subtype: Some(subtype) }),
                        ((x, y - 1), Ingress),
                        ((x, y + 1), Exgress),
                    ]
                } else if subtype == 2 {
                    vec![
                        ((x, y), Inner { kind: ObjectType::Combiner, subtype: Some(subtype) }),
                        ((x - 1, y), Exgress),
                        ((x + 1, y), Ingress),
                    ]
                } else if subtype == 3 {
                    vec![
                        ((x, y), Inner { kind: ObjectType::Combiner, subtype: Some(subtype) }),
                        ((x, y - 1), Exgress),
                        ((x, y + 1), Ingress),
                    ]
                } else if subtype == 4 {
                    vec![
                        ((x, y), Inner { kind: ObjectType::Combiner, subtype: Some(subtype) }),
                        ((x + 1, y), Inner { kind: ObjectType::Combiner, subtype: Some(subtype) }),
                        ((x - 1, y), Ingress),
                        ((x + 2, y), Exgress),
                    ]
                } else if subtype == 5 {
                    vec![
                        ((x, y), Inner { kind: ObjectType::Combiner, subtype: Some(subtype) }),
                        ((x, y + 1), Inner { kind: ObjectType::Combiner, subtype: Some(subtype) }),
                        ((x, y - 1), Ingress),
                        ((x, y + 2), Exgress),
                    ]
                } else if subtype == 6 {
                    vec![
                        ((x, y), Inner { kind: ObjectType::Combiner, subtype: Some(subtype) }),
                        ((x + 1, y), Inner { kind: ObjectType::Combiner, subtype: Some(subtype) }),
                        ((x - 1, y), Exgress),
                        ((x + 2, y), Ingress),
                    ]
                } else if subtype == 7 {
                    vec![
                        ((x, y), Inner { kind: ObjectType::Combiner, subtype: Some(subtype) }),
                        ((x, y + 1), Inner { kind: ObjectType::Combiner, subtype: Some(subtype) }),
                        ((x, y - 1), Exgress),
                        ((x, y + 2), Ingress),
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
        let task::Object { kind, subtype, x, y, width, height } = obj;
        match kind.as_str() {
            "obstacle" => Object::Obstacle { x, y, width, height },
            "deposit" => Object::Deposit { x, y, width, height, subtype: subtype.unwrap() },
            // TODO: prepare for tasks with other object types already set!
            _ => panic!("Cannot convert task object w/ type '{}' into proper object", kind)
            
        }
    }
}

impl From<solution::Object> for Object {
    fn from(obj: solution::Object) -> Self {
        let solution::Object { kind, subtype, x, y, width, height } = obj;
        match kind.as_str() {
            "deposit" => Object::Deposit { x, y, width: width.unwrap(), height: height.unwrap(), subtype },
            "mine" => Object::Mine { x, y, subtype },
            "conveyor" => Object::Conveyor { x, y, subtype },
            "combiner" => Object::Combiner { x, y, subtype },
            "factory" => Object::Factory { x, y, subtype },
            _ => panic!("Cannot convert solution object w/ type '{}' into proper object", kind)
        }
    }
}