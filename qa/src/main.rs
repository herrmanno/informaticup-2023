use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, time::Duration};

use model::{map::Map, object::Object, task::Task};
use simulator::SimulatorResult;
use solver::run::run_solver;

const SEEDS: [u64; 10] = [
    32491274, 923410234, 12375320, 1238493, 593810, 7382934, 3920134, 4742810, 123648, 83047,
];
const NUM_THREADS: usize = 8;
const RUNTIME_IN_SECS: u64 = 2;
const TASKS: [&str; 7] = [
    concat!(env!("CARGO_MANIFEST_DIR"), "/../inputs/001.task.json"),
    concat!(env!("CARGO_MANIFEST_DIR"), "/../inputs/002.task.json"),
    concat!(env!("CARGO_MANIFEST_DIR"), "/../inputs/003.task.json"),
    concat!(env!("CARGO_MANIFEST_DIR"), "/../inputs/004.task.json"),
    concat!(env!("CARGO_MANIFEST_DIR"), "/../inputs/long_path_001.json"),
    concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../inputs/path_finding_80_80.json"
    ),
    concat!(env!("CARGO_MANIFEST_DIR"), "/../inputs/xxl_001.json"),
];

macro_rules! OUT_DIR_NAME {
    () => {
        "qa"
    };
}

macro_rules! run_task {
    ($path: expr) => {{
        let task = Task::from_json_file($path).unwrap();

        let map = Map::new(
            task.width,
            task.height,
            task.objects.iter().cloned().map(Object::from).collect(),
        );

        let results = SEEDS
            .iter()
            .filter_map(|seed| {
                run_solver(
                    &task,
                    &map,
                    NUM_THREADS,
                    Duration::from_secs(RUNTIME_IN_SECS),
                    Some(*seed),
                )
                .map(|r| r.result)
            })
            .collect::<Vec<SimulatorResult>>();

        let score_best = results.iter().map(|o| o.score).max().unwrap() as f32;
        let turn_best = results.iter().map(|o| o.turn).max().unwrap() as f32;

        let score_worst = results.iter().map(|o| o.score).min().unwrap() as f32;
        let turn_worst = results.iter().map(|o| o.turn).min().unwrap() as f32;

        let score_sum: u32 = results.iter().map(|o| o.score).sum();
        let turn_sum: u32 = results.iter().map(|o| o.turn).sum();
        let score_avg = score_sum as f32 / SEEDS.len() as f32;
        let turn_avg = turn_sum as f32 / SEEDS.len() as f32;

        Some(TestResultMetric {
            best: TestResult {
                score: score_best,
                turn: turn_best,
            },
            worst: TestResult {
                score: score_worst,
                turn: turn_worst,
            },
            average: TestResult {
                score: score_avg,
                turn: turn_avg,
            },
        })
    }};
}

fn main() {
    let commit = String::from(env!("GIT_HASH"));
    let out_dir_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../target/",
        OUT_DIR_NAME!(),
        "/"
    );
    let out_file_path = format!("{}current.json", out_dir_path);
    let commit_file_path = format!("{}{}.json", out_dir_path, commit);
    let last_file_path = format!("{}last.json", out_dir_path);

    let last_result: Option<TestResults> = std::fs::File::open(&out_file_path)
        .map_err(|_| "cannot open last result")
        .and_then(|file| serde_json::de::from_reader(file).map_err(|_| "cannot parse last result"))
        .ok();

    if last_result.is_some() {
        std::fs::rename(&out_file_path, last_file_path).expect("Cannot move old result file");
    }

    let mut test_results = TestResults {
        seeds: SEEDS.to_vec(),
        time_per_task: RUNTIME_IN_SECS,
        cores: NUM_THREADS,
        results: BTreeMap::new(),
        commit,
    };

    for task in TASKS {
        let task_name = task.split_terminator('/').last().unwrap();
        let result = run_task!(task);
        test_results.results.insert(String::from(task_name), result);
    }

    let result_str = serde_json::ser::to_string_pretty(&test_results).unwrap();
    std::fs::create_dir_all(out_dir_path).expect("Cannot create out dir");
    std::fs::write(out_file_path, &result_str).expect("Cannot write results to file");
    std::fs::write(commit_file_path, &result_str).expect("Cannot write results to file");

    if let Some(last_results) = last_result {
        let mut warning = false;
        if last_results.seeds != test_results.seeds {
            println!("WARN: Seeds changed");
            warning = true;
        }
        if last_results.time_per_task != test_results.time_per_task {
            println!("WARN: Time per task changed");
            warning = true;
        }
        if last_results.cores != test_results.cores {
            println!("WARN: Cores changed");
            warning = true;
        }

        if warning {
            println!();
        }

        for (name, result) in test_results.results {
            if let Some(last_result) = last_results.results.get(&name) {
                match (last_result, &result) {
                    (Some(a), Some(b)) => {
                        println!("{}", name);
                        for (metric, a, b) in [
                            ("best", &a.best, &b.best),
                            ("worst", &a.worst, &b.worst),
                            ("average", &a.average, &b.average),
                        ] {
                            let score_change = (b.score - a.score) / a.score;
                            let turn_change = (b.turn - a.turn) / a.turn;

                            println!(
                                "\t{}:\n\t\tScore: {:.2}%\t({:.2} -> {:.2})\n\t\tTurns: {:.2}%\t({:.2} -> {:.2})",
                                metric,
                                score_change * 100f32,
                                a.score,
                                b.score,
                                turn_change,
                                a.turn,
                                b.turn,
                            );
                        }
                    }
                    (Some(_), None) => {
                        println!("{}: NO RESULTS", name);
                    }

                    (None, Some(b)) => {
                        println!("{}", name);
                        for (metric, b) in [
                            ("best", &b.best),
                            ("worst", &b.worst),
                            ("average", &b.average),
                        ] {
                            println!(
                                "\t{}:\n\t\tScore: {}\n\t\tTurns: {}",
                                metric, b.score, b.turn,
                            );
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct TestResults {
    commit: String,
    seeds: Vec<u64>,
    time_per_task: u64,
    cores: usize,
    results: BTreeMap<String, Option<TestResultMetric>>,
}

#[derive(Clone, Serialize, Deserialize)]
struct TestResultMetric {
    best: TestResult,
    worst: TestResult,
    average: TestResult,
}

#[derive(Clone, Serialize, Deserialize)]
struct TestResult {
    score: f32,
    turn: f32,
}

impl From<&SimulatorResult> for TestResult {
    fn from(s: &SimulatorResult) -> Self {
        TestResult {
            score: s.score as f32,
            turn: s.turn as f32,
        }
    }
}
