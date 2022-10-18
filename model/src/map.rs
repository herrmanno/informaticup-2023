use std::{collections::HashMap, fmt::Display};

use crate::object::{Object, ObjectCell};

#[derive(Debug)]
pub struct Map {
    map: HashMap<(u32, u32), ObjectCell>,
    objects: Vec<MapObject>,
}

#[derive(Debug)]
pub struct MapObject {
    pub object: Object,
    pub ingresses: Vec<(u32, u32)>,
    pub exgresses: Vec<(u32, u32)>,
}

impl Map {
    pub fn new(objects: Vec<Object>) -> Self {
        let objects: Vec<MapObject> = objects
            .into_iter()
            .enumerate()
            .map(|(index, object)| {
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
            })
            .collect();

        let mut map = HashMap::new();
        for (index, object) in objects.iter().enumerate() {
            object.object.place_on_map(index, &mut map);
        }

        Map { objects, map }
    }

    pub fn get_objects(&self) -> &Vec<MapObject> {
        &self.objects
    }

    pub fn get_cell(&self, x: u32, y: u32) -> Option<&ObjectCell> {
        self.map.get(&(x, y))
    }
}

impl Display for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.map.is_empty() {
            f.write_str("Empty map")?;
            return Ok(());
        }

        let width = self.map.keys().map(|coord| coord.0).max().unwrap();
        let height = self.map.keys().map(|coord| coord.1).max().unwrap();
        for y in 0..=height {
            for x in 0..=width {
                let c = self.map.get(&(x, y)).map(|cell| cell.into()).unwrap_or('.');
                f.write_fmt(format_args!("{}", c))?;
            }
            f.write_str("\n")?;
        }

        Ok(())
    }
}
