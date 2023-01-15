use std::io::Read;

use crate::{object::Object, solution::Solution, task::Task};

// TODO: proper error types
pub fn read_input_from_stdin() -> Result<(Task, Option<Solution>), String> {
    let mut input_string = String::new();
    std::io::stdin()
        .lock()
        .read_to_string(&mut input_string)
        .map_err(|_| String::from("Could not read from stdin"))?;

    read_input(input_string.as_str())
}

pub fn read_input_from_file(file_path: &str) -> Result<(Task, Option<Solution>), String> {
    let input_string = std::fs::read_to_string(file_path)
        .map_err(|_| format!("Cannot read input from file {}", file_path))?;

    read_input(input_string.as_str())
}

fn read_input(input: &str) -> Result<(Task, Option<Solution>), String> {
    match serde_json::de::from_str::<Task>(input) {
        Ok(mut task) => {
            let construction_objects: Vec<Object> = task
                .objects
                .iter()
                .filter(|obj| !matches!(obj, Object::Deposit { .. } | Object::Obstacle { .. }))
                .cloned()
                .collect();

            if !construction_objects.is_empty() {
                let landscape_objects: Vec<Object> = task
                    .objects
                    .iter()
                    .filter(|obj| matches!(obj, Object::Deposit { .. } | Object::Obstacle { .. }))
                    .cloned()
                    .collect();

                task.objects = landscape_objects;
                let solution = Solution::from(construction_objects);
                Ok((task, Some(solution)))
            } else {
                Ok((task, None))
            }
        }
        Err(e) => Err(format!("{:?}", e)),
    }
}
