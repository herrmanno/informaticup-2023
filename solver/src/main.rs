use clap::Parser;
use cli::Args;
use common::{debug, release};
use model::{cli::CliFile, map::Map, object::Object, solution::Solution, task::Task};
use solver::solve::solve;

mod cli;

fn main() {
    let args = Args::parse();
    let (task, _) = if let Some(cli_path) = args.cli {
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

    let mut map = Map::new(
        task.width,
        task.height,
        task.objects.iter().cloned().map(Object::from).collect(),
    );

    if let Some(solution) = solve(&task, &mut map) {
        debug!("{:?}", solution.0);
        debug!(
            "{}",
            CliFile::new(task.clone(), Solution::from(&solution.1))
                .to_json_string()
                .unwrap()
        );
        release!("{}", Solution::from(&solution.1).to_json_string().unwrap());
    } else {
        debug!("No solution found");
        release!("{}", Solution::default().to_json_string().unwrap());
    }
}
