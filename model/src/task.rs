use serde::Deserialize;
use serde_json;

#[derive(Deserialize, Debug, Clone)]
pub struct Task {
    pub width: u32,
    pub height: u32,
    pub objects: Vec<Object>,
    pub products: Vec<Product>,
    pub turns: u32,
    pub time: Option<u32>,
}

impl Task {
    pub fn from_json_file(path: &str) -> Result<Self, impl std::error::Error> {
        // TODO: use better error conept
        let s = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&s)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Object {
    #[serde(rename = "type")]
    pub kind: String,
    pub subtype: Option<u8>,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Product {
    #[serde(rename = "type")]
    pub kind: String,
    pub subtype: u8,
    pub resources: Vec<u32>,
    pub points: u32,
}
