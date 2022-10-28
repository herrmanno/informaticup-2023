use std::{cell::RefCell, rc::Rc};

use criterion::{criterion_group, criterion_main, Criterion};
use model::{map::Map, object::Object, task::Task};
use rand::{rngs::StdRng, SeedableRng};
use solver::paths::Paths;
use solver::solve::Solver;

const SEEDS: [u64; 3] = [79128620393, 1237923833, 34329582];

macro_rules! run_task {
    ($criterion: ident, $path: expr, $name: expr) => {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../", $path);
        let task = Task::from_json_file(path).unwrap();
        let map = Map::from(&task);
        let mut solvers = (0..3).map(|i| {
            let rng = Rc::new(RefCell::new(StdRng::seed_from_u64(SEEDS[i])));
            Solver::new(&task, &map, rng)
        }).collect::<Vec<Solver<StdRng>>>();

        let mut i = 0;
        $criterion.bench_function(concat!("solve ", $name), move |b| b.iter(|| {
            solvers[i].next();
            i += 1;
            i %= 3;
        }));
    };
}

fn task_001_benchmark(c: &mut Criterion) {
    run_task!(c, "inputs/001.task.json", "task 001");
}

fn task_002_benchmark(c: &mut Criterion) {
    run_task!(c, "inputs/002.task.json", "task 002");
}

fn task_003_benchmark(c: &mut Criterion) {
    run_task!(c, "inputs/003.task.json", "task 003");
}

fn task_004_benchmark(c: &mut Criterion) {
    run_task!(c, "inputs/004.task.json", "task 004");
}

criterion_group! {
    name = solver_benches;
    config = Criterion::default().significance_level(0.25).sample_size(30);
    targets = task_001_benchmark, task_002_benchmark, task_003_benchmark, task_004_benchmark,
}

macro_rules! run_pathfinding {
    ($criterion: ident, $start_point: expr, $path: expr, $name: expr) => {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../", $path);
        let task = Task::from_json_file(path).unwrap();
        let map = Map::from(&task);
        let deposits = map
            .get_objects()
            .filter(|obj| matches!(obj, Object::Deposit { .. }))
            .cloned()
            .collect::<Vec<Object>>();
        let mut path_finders = (0..3).map(|i| {
            let rng = Rc::new(RefCell::new(StdRng::seed_from_u64(SEEDS[i])));
            Paths::new(&[$start_point], 0, &deposits[..], &map, rng)
        }).collect::<Vec<Paths<StdRng>>>();

        let mut i = 0;
        $criterion.bench_function(concat!("find path ", $name), move |b| {
            b.iter(|| {
                path_finders[i].next();
                i += 1;
                i %= 3;
            })
        });
    };
}

fn paht_finding_maze_80_80(c: &mut Criterion) {
    run_pathfinding!(
        c,
        (79, 79),
        "inputs/path_finding_80_80.json",
        "path finding in 80x80 maze"
    );
}

criterion_group! {
    name = path_finding_benches;
    config = Criterion::default().significance_level(0.25).sample_size(100);
    targets = paht_finding_maze_80_80
}

criterion_main!(solver_benches, path_finding_benches);
