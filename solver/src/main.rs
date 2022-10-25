use clap::Parser;
use cli::Args;
use common::{debug, release};
use model::{cli::CliFile, map::Map, object::Object, solution::Solution, task::Task};
use simulator::SimulatorResult;
use solver::solve::Solver;
use std::{
    sync::{mpsc, Arc, RwLock},
    thread,
    time::Duration,
};

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

    let map = Map::new(
        task.width,
        task.height,
        task.objects.iter().cloned().map(Object::from).collect(),
    );

    let (sender, receiver) = mpsc::channel();

    let time = Duration::from_secs(args.time.unwrap_or(100));

    debug!("Time bound {}s", time.as_secs());

    let num_threads = args.cores.unwrap_or_else(|| {
        thread::available_parallelism()
            .map(|i| i.get())
            .unwrap_or(1)
    });

    debug!("Using {} thread(s)", num_threads);

    let stop_condition = Arc::new(RwLock::new(false));

    thread::scope(|scope| {
        let task = &task;
        let map = &map;

        for i_thread in 0..num_threads {
            debug!("Starting thread #{}", i_thread);

            let sender = sender.clone();
            let stop_condition = Arc::clone(&stop_condition);
            scope.spawn(move || {
                let solver = Solver::new(task, map);
                for solution in solver {
                    if *(*stop_condition).read().unwrap() {
                        break;
                    }
                    sender
                        .send(solution)
                        .expect("Could not send solution from worker thread to main thread");
                }
            });
        }

        thread::sleep(time);
        *(*stop_condition).write().unwrap() = true;
    });

    let mut result: Option<(SimulatorResult, Map)> = None;
    while let Ok(solution) = receiver.recv_timeout(Duration::from_micros(10)) {
        result = match result {
            None => Some(solution),
            Some(result) if solution.0 > result.0 => Some(solution),
            _ => result,
        };
    }

    if let Some(solution) = result {
        if cfg!(debug_assertions) || args.stats {
            println!("{:?}", solution.0);
        }

        if args.print {
            println!("{}", solution.1);
        }

        if cfg!(debug_assertions) || args.cli_out {
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
