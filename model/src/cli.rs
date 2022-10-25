use serde::{Deserialize, Serialize};
use serde_json;

use crate::{solution::Solution, task::Task};

#[derive(Deserialize, Serialize, Debug)]
pub struct CliFile(Vec<CliFileEntry>);

impl CliFile {
    pub fn new(task: Task, solution: Solution) -> Self {
        CliFile(vec![
            CliFileEntry::TaskEntry(task),
            CliFileEntry::SolutionEntry(solution),
        ])
    }

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

    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn task(&self) -> Option<Task> {
        self.0
            .iter()
            .filter_map(|entry| match entry {
                CliFileEntry::TaskEntry(task) => Some(task.clone()),
                _ => None,
            })
            .next()
    }

    pub fn solution(&self) -> Option<Solution> {
        self.0
            .iter()
            .filter_map(|entry| match entry {
                CliFileEntry::SolutionEntry(solution) => Some(solution.clone()),
                _ => None,
            })
            .next()
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
enum CliFileEntry {
    TaskEntry(Task),
    SolutionEntry(Solution),
}
