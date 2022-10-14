mod cli;

use std::collections::HashMap;

use clap::Parser;
use cli::Args;

use model::{task::Task, solution::Solution, object::{Object, ObjectCell}, cli::CliFile};

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

    // println!("Task:");
    // println!("{:#?}", task);
    // println!("Solution:");
    // println!("{:#?}", solution);

    let mut map = HashMap::new();
    let width = task.width;
    let height = task.height;

    for task_object in task.objects.into_iter().map(Object::from) {
        task_object.place_on_map(&mut map);
    }

    print_map(width, height, &map);

    for solution_object in solution.0.into_iter().map(Object::from) {
        solution_object.place_on_map(&mut map);
    }

    print_map(width, height, &map);
}

fn print_map(width: u32, height: u32, map: &HashMap<(u32, u32), ObjectCell>) {
    for y in 0..height {
        for x in 0..width {
            let c = map.get(&(x, y)).map(|cell| cell.into()).unwrap_or('.');
            print!("{}", c);
        }
        println!();
    }
    println!();
}