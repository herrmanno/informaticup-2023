use crate::solve::Solver;
use common::debug;
use model::{map::Map, task::Task};
use rand::{rngs::StdRng, SeedableRng};
use simulator::SimulatorResult;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{mpsc, Arc, RwLock},
    thread,
    time::{Duration, Instant},
};

pub fn run_solver(
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
