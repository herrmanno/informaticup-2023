use std::io::Read;

use crate::{cli::CliFile, solution::Solution, task::Task};

// TODO: proper error types
pub fn read_input_from_stdin() -> Result<(Task, Option<Solution>), String> {
    let mut input_string = String::new();
    std::io::stdin()
        .lock()
        .read_to_string(&mut input_string)
        .map_err(|_| String::from("Could not read from stdin"))?;

    if let Ok(cli_file) = serde_json::de::from_str::<CliFile>(input_string.as_str()) {
        let task = cli_file
            .task()
            .expect("Could not extract task from cli file");
        return Ok((task, cli_file.solution()));
    };

    if let Ok(task) = serde_json::de::from_str::<Task>(input_string.as_str()) {
        return Ok((task, None));
    };

    Err(String::from("Could not read input"))
}
