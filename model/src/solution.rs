use serde::{Deserialize, Serialize};
use serde_json;

use crate::object::Object;

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

impl<T> From<T> for Solution
where
    T: IntoIterator<Item = Object>,
{
    fn from(objects: T) -> Self {
        Solution(objects.into_iter().collect())
    }
}
