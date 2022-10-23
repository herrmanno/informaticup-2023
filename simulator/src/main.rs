mod cli;

use clap::Parser;

use model::{cli::CliFile, solution::Solution, task::Task};

use cli::Args;
use simulator::{generate_map, simulate};

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

    let map = generate_map(&task, &solution);
    let result = simulate(&task, &map, false);
    println!("{:?}", result);
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_simulation {
        ($path:expr) => {{
            let cli_path = $path;
            let (task, solution) =
                CliFile::from_json_file(cli_path).expect("Could not read cli file");
            let map = generate_map(&task, &solution);
            simulate(&task, &map, false)
        }};
    }

    #[test]
    fn test_conveyor_branch() {
        let result = test_simulation!("./inputs/conveyor_branch.json");
        assert_eq!(10, result.score);
    }

    #[test]
    fn test_simulation_1() {
        let result = test_simulation!("./inputs/test1.json");
        assert_eq!(40, result.score);
    }

    #[test]
    fn test_simulation_2() {
        let result = test_simulation!("./inputs/test2.json");
        assert_eq!(162, result.score);
    }

    #[test]
    fn test_task_004() {
        let result = test_simulation!("./inputs/test_task_004.json");
        assert_eq!(240, result.score);
    }
}
