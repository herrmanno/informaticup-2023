use std::{cell::RefCell, rc::Rc};

use criterion::{criterion_group, criterion_main, Criterion};
use model::{map::Map, object::Object, task::Task};
use rand::{rngs::StdRng, SeedableRng};
use solver::paths::Paths;
use solver::solve::Solver;

const SEED: u64 = 79128620393;

macro_rules! run_task {
    ($criterion: ident, $path: expr, $name: expr) => {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../", $path);
        let task = Task::from_json_file(path).unwrap();
        let map = Map::from(&task);
        let rng = Rc::new(RefCell::new(StdRng::seed_from_u64(SEED)));
        let mut solver = Solver::new(&task, &map, rng);

        $criterion.bench_function(concat!("solve ", $name), move |b| b.iter(|| solver.next()));
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
        let rng = Rc::new(RefCell::new(StdRng::seed_from_u64(SEED)));
        let mut paths = Paths::new(&[$start_point], 0, &deposits[..], &map, rng);

        $criterion.bench_function(concat!("find path ", $name), move |b| {
            b.iter(|| paths.next())
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
