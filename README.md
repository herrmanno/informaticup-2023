# InformatiCup 2023

Solver for the [2023 InformatiCup challenge „Profit“](https://github.com/informatiCup/informatiCup2023)

## Install / Run

### Build
```sh
cargo build --release [--features stats]
```

If the `stats` feature is activated, the solver will print the number of calculated solutions per
second.

Note: For competition, the solver shall be run as docker container.

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
target/release/solver
    --time [runtime in seconds]
    --cores [number of threads to use]
    [--print] # prints the final solution to stdout
    [--stats] # prints evaluation stats (score and turn, when score was achieved) to stdout
    < some_task.json
```

### [Quality Assurance](./qa/)
Test runner for a fixed set of tasks.

#### Example
```
target/release/qa
```

### [Printer](./printer/)
Binary to read task and solution files and print resulting map

#### Example
```
target/release/printer -- < some_task.json
```

### [Simulator](./simulator/)
Binary for running a simulation from a task/solution file

#### Example
```
target/release/simulator -- < some_task.json
```

### [Model](./model/)
Model instances for task/solution files and basic building objects

## Test

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

A JSON object that contains the tasks basic information and given objects (may also include
non-landscape objects. In that case the task is split into task description and solution).
