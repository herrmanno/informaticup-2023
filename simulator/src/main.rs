mod cli;

use clap::Parser;

use model::input::read_input_from_stdin;

use cli::Args;
use simulator::{generate_map, simulate};

fn main() {
    let args = Args::parse();
    let (task, solution) = read_input_from_stdin().unwrap();
    let solution = solution.unwrap_or_default();

    let map = generate_map(&task, &solution);
    let result = simulate(&task, &map, args.quiet);
    println!("{:?}", result);
}

#[cfg(test)]
mod tests {
    use super::*;
    use model::input::read_input_from_file;

    macro_rules! test_simulation {
        ($path:expr) => {{
            let cli_path = $path;
            let (task, solution) = read_input_from_file(cli_path).expect("Could not read cli file");
            let map = generate_map(&task, &solution.unwrap());
            simulate(&task, &map, false)
        }};
    }

    #[test]
    fn test_conveyor_branch() {
        let result = test_simulation!("./inputs/conveyor_branch.json");
        assert_eq!(10, result.score);
    }

    #[test]
    fn test_simulation_1() {
        let result = test_simulation!("./inputs/test1.json");
        assert_eq!(40, result.score);
    }

    #[test]
    fn test_simulation_2() {
        let result = test_simulation!("./inputs/test2.json");
        assert_eq!(162, result.score);
    }

    #[test]
    fn test_task_004() {
        let result = test_simulation!("./inputs/test_task_004.json");
        assert_eq!(240, result.score);
    }
}
