use serde::{Deserialize, Serialize};
use serde_json;

use crate::map::Map;

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct Solution(pub Vec<Object>);

impl Solution {
    pub fn from_json_file(path: &str) -> Result<Self, impl std::error::Error> {
        // TODO: use better error conept
        let s = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&s)
    }

    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

impl From<&Map> for Solution {
    fn from(map: &Map) -> Self {
        let objects = map
            .get_objects()
            .filter_map(|object| match object {
                crate::object::Object::Obstacle { .. } => None,
                crate::object::Object::Deposit { .. } => None,
                object => Some(Object::from(object)),
            })
            .collect();

        Solution(objects)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Object {
    #[serde(rename = "type")]
    pub kind: String,
    pub subtype: u8,
    pub x: i8,
    pub y: i8,
}

impl From<&crate::object::Object> for Object {
    fn from(object: &crate::object::Object) -> Self {
        let (x, y) = object.coords();
        Object {
            kind: object.kind().into(),
            subtype: object.subtype().unwrap(),
            x,
            y,
        }
    }
}
