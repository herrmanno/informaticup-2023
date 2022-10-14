mod cli;

use clap::Parser;
use cli::Args;

use model::{cli::CliFile, map::Map, object::Object, solution::Solution, task::Task};

fn main() {
    let args = Args::parse();
    let (task, solution) = if let Some(cli_path) = args.cli {
        CliFile::from_json_file(&cli_path).expect("Could not read cli file")
    } else {
        let task = if let Some(task_path) = args.task {
            Task::from_json_file(&task_path).expect("Could not read task file")
        } else {
            panic!("Neither 'cli' nor 'task' supplied");
        };
        let solution = if let Some(solution_path) = args.solution {
            Solution::from_json_file(&solution_path).expect("Could not read solution file")
        } else {
            Solution::default()
        };
        (task, solution)
    };

    let mut objects = Vec::with_capacity(task.objects.len() + solution.0.len());
    objects.extend(task.objects.into_iter().map(Object::from));
    objects.extend(solution.0.into_iter().map(Object::from));

    let map = Map::new(objects);

    println!("{}", map);
}
