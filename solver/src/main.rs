use clap::Parser;
use cli::Args;
use common::{debug, release};
use model::{
    cli::CliFile, input::read_input_from_stdin, map::Map, object::Object, solution::Solution,
};
use std::{
    thread,
    time::{Duration, Instant},
};

use crate::cli::OutputFormat;
use solver::run::run_solver;

mod cli;

fn main() {
    let now = Instant::now();
    let args = Args::parse();

    let (task, _) = read_input_from_stdin().unwrap();

    let map = Map::new(
        task.width,
        task.height,
        task.objects.iter().cloned().map(Object::from).collect(),
    );

    let runtime = {
        let runtime_in_secs = args.time.unwrap_or(task.time.unwrap_or(100) as u64);
        Duration::from_secs(runtime_in_secs) - now.elapsed()
    };

    debug!("Time bound {}s", runtime.as_secs());

    let num_threads = args.cores.unwrap_or_else(|| {
        thread::available_parallelism()
            .map(|i| i.get())
            .unwrap_or(1)
    });

    debug!("Using {} thread(s)", num_threads);

    let result = run_solver(&task, &map, num_threads, runtime, args.seed);

    if let Some(solution) = result {
        if cfg!(debug_assertions) || args.stats {
            println!("{:?}", solution.0);
        }

        if args.print {
            println!("{}", solution.1);
        }

        if cfg!(debug_assertions) || args.output_format() == OutputFormat::Cli {
            println!(
                "{}",
                CliFile::new(task, Solution::from(&solution.1))
                    .to_json_string()
                    .unwrap()
            );
        } else {
            println!("{}", Solution::from(&solution.1).to_json_string().unwrap());
        }
    } else {
        debug!("No solution found");
        release!("{}", Solution::default().to_json_string().unwrap());
    }
}
