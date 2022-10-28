use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, time::Duration};

use model::{map::Map, object::Object, task::Task};
use simulator::SimulatorResult;
use solver::run::run_solver;

const SEEDS: [u64; 3] = [32491274, 923410234, 12375320];
const NUM_THREADS: usize = 2;
const RUNTIME_IN_SECS: u64 = 10;
const TASKS: [&str; 4] = [
    concat!(env!("CARGO_MANIFEST_DIR"), "/../inputs/001.task.json"),
    concat!(env!("CARGO_MANIFEST_DIR"), "/../inputs/002.task.json"),
    concat!(env!("CARGO_MANIFEST_DIR"), "/../inputs/003.task.json"),
    concat!(env!("CARGO_MANIFEST_DIR"), "/../inputs/004.task.json"),
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
            .map(|seed| {
                run_solver(
                    &task,
                    &map,
                    NUM_THREADS,
                    Duration::from_secs(RUNTIME_IN_SECS),
                    Some(*seed),
                )
                .map(|r| TestResult::from(&r.0))
            })
            .collect::<Vec<Option<TestResult>>>();

        if results.iter().any(Option::is_none) {
            None
        } else {
            let score_sum = results
                .iter()
                .cloned()
                .map(|o| o.unwrap().score)
                .sum::<f32>();
            let turn_sum = results
                .iter()
                .cloned()
                .map(|o| o.unwrap().turn)
                .sum::<f32>();

            let score = score_sum / SEEDS.len() as f32;
            let turn = turn_sum / SEEDS.len() as f32;
            Some(TestResult { score, turn })
        }
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
    std::fs::create_dir_all(&out_dir_path).expect("Cannot create out dir");
    std::fs::write(&out_file_path, &result_str).expect("Cannot write results to file");
    std::fs::write(&commit_file_path, &result_str).expect("Cannot write results to file");

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
                        let score_change = (b.score - a.score) / a.score;
                        let turn_change = (b.turn - a.turn) / a.turn;

                        println!(
                            "{}:\n\tScore: {:.2}%\t({:.2} -> {:.2})\n\tTurns: {:.2}%\t({:.2} -> {:.2})",
                            name,
                            score_change,
                            a.score,
                            b.score,
                            turn_change,
                            a.turn,
                            b.turn,
                        );
                    }
                    (Some(_), None) => {
                        println!("{}: NO RESULTS", name);
                    }

                    (None, Some(b)) => {
                        println!("{}:\n\tScore: {}\n\tTurns: {}", name, b.score, b.turn,);
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
    results: BTreeMap<String, Option<TestResult>>,
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
