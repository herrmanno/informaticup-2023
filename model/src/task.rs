use serde::{Deserialize, Serialize};
use serde_json;

use crate::object::Object;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Task {
    pub width: u8,
    pub height: u8,
    pub objects: Vec<Object>,
    pub products: Vec<Product>,
    pub turns: u32,
    pub time: Option<u32>, //TODO: check if this is not optional
}

impl Task {
    pub fn from_json_file(path: &str) -> Result<Self, impl std::error::Error> {
        // TODO: use better error conept
        let s = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&s)
    }

    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Product {
    #[serde(rename = "type")]
    pub kind: String,
    pub subtype: u8,
    pub resources: Vec<u32>,
    pub points: u32,
}
