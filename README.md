# InformatiCup 2023

Solver for the [2023 InformatiCup challenge „Profit“](https://github.com/informatiCup/informatiCup2023)

## Install / Run

For competition, the solver shall be run as docker container

### Docker build
docker build -t <TAG> .

### Docker run
docker run -i <TAG> [OPTIONS] < <input>

## Packages

The project consists of multiple packages:

### [Solver](./solver/)
Solver for puzzles

#### Example
```
cargo run -p solver -- < some_task.json
```

### [Quality Assurance](./qa/)
Test runner for a fixed set of tasks.

#### Example
```
cargo run -p qa
```

### [Printer](./printer/)
Binary to read task and solution files and print resulting map

#### Example
```
cargo run -p printer -- < some_cli.json
cargo run -p printer -- < some_task.json
```

### [Simulator](./simulator/)
Binary for running a simulation from a task/solution file

#### Example
```
cargo run -p simulator -- < some_cli.json
```

### [Model](./model/)
Model instances for task/solution files and basic building objects

## Getting started

### Build
```
cargo build [--release]
```

### Run benchmarks
```
cargo bench --bench benchmarks
```

### Profile
To create profiling data:
```
cargo build --profile=profiling
valgrind --tool=callgrind ./target/profiling/solver --time x --cores y < inputs/003.task.json
```

To view profiling data (w/ [qcachegrind](https://kcachegrind.github.io/html/Home.html))
```
qcachegrind callgrind.out.xxxx
```

## Input formats

### Task file

A JSON object that contains the tasks basic information and given objects (deposits and obstacles).

### Cli file

A JSON array that contains two values.
1. A task object as described above
2. An array of objects that are part of the solution

## Todos

- change cli to taking task / cli file directly from stdin (drop clap?!)
    - add option to solver to output cli file/json, for quick checking in online simulator
- implement hard time limit
- implement partial timers for log output
- solver
    - use distance_map for placing factories
    - skip factories with probabilty of their outcome (available resources * resource demand * points for product)
    - return 'partial path length' together with each ingress on path.ingresses() and use them as
      current path length when calculating new paths based on current paths
    - only simulate solution to MAX_ROUNDS
    - pass RNGs into Solver::new(..)
    - accept RNG seed as cli parameter
- simulator
    - accept upper bound of rounds
