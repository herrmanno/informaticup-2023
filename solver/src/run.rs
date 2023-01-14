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

#[cfg(not(feature = "stats"))]
pub struct RunnerResult {
    pub result: SimulatorResult,
    pub map: Map,
}

#[cfg(feature = "stats")]
pub struct RunnerResult {
    pub result: SimulatorResult,
    pub map: Map,
    pub solutions_per_second: u128,
}

pub fn run_solver(
    task: &Task,
    map: &Map,
    num_threads: usize,
    runtime: Duration,
    seed: Option<u64>,
) -> Option<RunnerResult> {
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
) -> Option<RunnerResult> {
    let time_start = Instant::now();
    let mut result: Option<(SimulatorResult, Map)> = None;
    let rng = match seed {
        Some(seed) => StdRng::seed_from_u64(seed),
        _ => StdRng::from_entropy(),
    };
    let mut solver = Solver::new(task, map, Rc::new(RefCell::new(rng)), runtime);

    let mut next_solution_estimate = RollingAverage::new();
    let mut last_solution = Instant::now();
    for solution in solver.by_ref() {
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

    #[cfg(feature = "stats")]
    {
        let solutions_per_second =
            1000 * solver.get_num_solutions() as u128 / time_start.elapsed().as_millis();
        result.map(|(result, map)| RunnerResult {
            result,
            map,
            solutions_per_second,
        })
    }
    #[cfg(not(feature = "stats"))]
    {
        result.map(|(result, map)| RunnerResult { result, map })
    }
}

fn run_solver_multi_threaded(
    task: &Task,
    map: &Map,
    num_threads: usize,
    runtime: Duration,
    seed: Option<u64>,
) -> Option<RunnerResult> {
    let time_start = Instant::now();
    // Extra time for accumulating gathered solutions TODO: find best value by empirical measurement
    let time_for_accumulation = runtime / 6;

    #[cfg(feature = "stats")]
    let num_solutions = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let max_iteration_time = runtime - time_for_accumulation;
    let (sender, receiver) = mpsc::channel();
    let stop_condition = Arc::new(RwLock::new(false));

    thread::scope(|scope| {
        let task = &task;
        // let map = &map;

        for i_thread in 0..num_threads {
            let map = map.clone();
            debug!("Starting thread #{}", i_thread);

            #[cfg(feature = "stats")]
            let num_solutions = Arc::clone(&num_solutions);

            let sender = sender.clone();
            let stop_condition = Arc::clone(&stop_condition);
            scope.spawn(move || {
                let rng = match seed {
                    Some(seed) => StdRng::seed_from_u64(seed.wrapping_add(i_thread as u64)),
                    _ => StdRng::from_entropy(),
                };
                let mut solver =
                    Solver::new(task, &map, Rc::new(RefCell::new(rng)), max_iteration_time);
                for solution in solver.by_ref() {
                    if *(*stop_condition).read().unwrap() {
                        #[cfg(feature = "stats")]
                        {
                            num_solutions.fetch_add(
                                solver.get_num_solutions(),
                                std::sync::atomic::Ordering::AcqRel,
                            );
                        }

                        break;
                    }
                    sender
                        .send(solution)
                        .expect("Could not send solution from worker thread to main thread");
                }
            });
        }

        debug!("Workers started");
        thread::sleep(runtime - time_start.elapsed() - time_for_accumulation);
        debug!("Stopping workers");
        *(*stop_condition).write().unwrap() = true;
        debug!("Workers stopped");
        // drop sender, so receiving results will terminate after last result was read from pipe
        drop(sender);
    });

    debug!("Accumulating results");

    let mut result: Option<(SimulatorResult, Map)> = None;
    while let Ok(solution) = receiver.recv() {
        result = match result {
            None => Some(solution),
            Some(result) if solution.0 > result.0 => Some(solution),
            _ => result,
        };
    }

    #[cfg(feature = "stats")]
    {
        let solutions_per_second = 1000
            * num_solutions.load(std::sync::atomic::Ordering::Acquire) as u128
            / time_start.elapsed().as_millis();
        result.map(|(result, map)| RunnerResult {
            result,
            map,
            solutions_per_second,
        })
    }
    #[cfg(not(feature = "stats"))]
    {
        result.map(|(result, map)| RunnerResult { result, map })
    }
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
