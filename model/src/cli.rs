use serde::Deserialize;
use serde_json;

use crate::{solution::Solution, task::Task};

#[derive(Deserialize, Debug)]
pub struct CliFile(Vec<CliFileEntry>);

impl CliFile {
    pub fn from_json_file(path: &str) -> Result<(Task, Solution), Box<dyn std::error::Error>> {
        let s = std::fs::read_to_string(path).unwrap();
        let CliFile(entries) = serde_json::from_str(&s)?;
        let task = entries
            .iter()
            .find_map(|e| match e {
                CliFileEntry::TaskEntry(task) => Some(task),
                _ => None,
            })
            .expect("No task found in cli file");
        let solution = entries
            .iter()
            .find_map(|e| match e {
                CliFileEntry::SolutionEntry(solution) => Some(solution),
                _ => None,
            })
            .expect("No task found in cli file");

        Ok((task.clone(), solution.clone()))
    }
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum CliFileEntry {
    TaskEntry(Task),
    SolutionEntry(Solution),
}
