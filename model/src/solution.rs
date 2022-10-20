use serde::Deserialize;
use serde_json;

#[derive(Deserialize, Default, Debug, Clone)]
pub struct Solution(pub Vec<Object>);

impl Solution {
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
    pub subtype: u8,
    pub x: i8,
    pub y: i8,
}
