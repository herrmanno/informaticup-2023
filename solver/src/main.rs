use clap::Parser;
use cli::Args;
use common::{debug, release};
use model::{
    cli::CliFile, input::read_input_from_stdin, map::Map, object::Object, solution::Solution,
    task::Task,
};
use rand::{rngs::StdRng, SeedableRng};
use simulator::SimulatorResult;
use solver::solve::Solver;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{mpsc, Arc, RwLock},
    thread,
    time::{Duration, Instant},
};

use crate::cli::OutputFormat;

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

    let runtime = Duration::from_secs(args.time.unwrap_or(100)) - now.elapsed();

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

fn run_solver(
    task: &Task,
    map: &Map,
    num_threads: usize,
    runtime: Duration,
    seed: Option<u64>,
) -> Option<(SimulatorResult, Map)> {
    if num_threads == 1 {
        run_solver_single_threaded(task, map, runtime, seed)
    } else {
        run_solver_multi_threaded(task, map, num_threads, runtime, seed)
    }
}

fn run_solver_single_threaded(
    task: &Task,
    map: &Map,
    runtime: Duration,
    seed: Option<u64>,
) -> Option<(SimulatorResult, Map)> {
    let time_start = Instant::now();
    let mut result: Option<(SimulatorResult, Map)> = None;
    let rng = match seed {
        Some(seed) => StdRng::seed_from_u64(seed),
        _ => StdRng::from_entropy(),
    };
    let solver = Solver::new(task, map, Rc::new(RefCell::new(rng)));

    let mut next_solution_estimate = RollingAverage::new();
    let mut last_solution = Instant::now();
    for solution in solver {
        let now = Instant::now();
        next_solution_estimate.add(now.duration_since(last_solution));
        last_solution = now;

        result = match result {
            None => Some(solution),
            Some(result) if solution.0 > result.0 => Some(solution),
            _ => result,
        };

        if time_start.elapsed() + next_solution_estimate.get() > runtime {
            break;
        }
    }

    result
}

fn run_solver_multi_threaded(
    task: &Task,
    map: &Map,
    num_threads: usize,
    runtime: Duration,
    seed: Option<u64>,
) -> Option<(SimulatorResult, Map)> {
    let now = Instant::now();
    // Extra time for accumulating gathered solutions TODO: find best value by empirical measurement
    let time_for_accumulation = runtime / 6;

    let (sender, receiver) = mpsc::channel();
    let stop_condition = Arc::new(RwLock::new(false));

    thread::scope(|scope| {
        let task = &task;
        let map = &map;

        for i_thread in 0..num_threads {
            debug!("Starting thread #{}", i_thread);

            let sender = sender.clone();
            let stop_condition = Arc::clone(&stop_condition);
            scope.spawn(move || {
                let rng = match seed {
                    Some(seed) => StdRng::seed_from_u64(seed.wrapping_add(i_thread as u64)),
                    _ => StdRng::from_entropy(),
                };
                let solver = Solver::new(task, map, Rc::new(RefCell::new(rng)));
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

        thread::sleep(runtime - now.elapsed() - time_for_accumulation);
        *(*stop_condition).write().unwrap() = true;
        // drop sender, so receiving results will terminate after last result was read from pipe
        drop(sender);
    });

    let mut result: Option<(SimulatorResult, Map)> = None;
    while let Ok(solution) = receiver.recv() {
        result = match result {
            None => Some(solution),
            Some(result) if solution.0 > result.0 => Some(solution),
            _ => result,
        };
    }

    result
}

struct RollingAverage {
    average: Duration,
    count: u32,
}

impl RollingAverage {
    fn new() -> RollingAverage {
        RollingAverage {
            average: Default::default(),
            count: 0,
        }
    }

    fn add(&mut self, value: Duration) {
        self.average = ((self.average * self.count) + value) / (self.count + 1);
        self.count += 1;
    }

    #[inline]
    fn get(&self) -> Duration {
        self.average
    }
}
