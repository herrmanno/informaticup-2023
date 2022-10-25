# InformatiCup 2023

Solver for the [2023 InformatiCup challenge „Profit“](https://github.com/informatiCup/informatiCup2023)

## Packages

The project consists of multiple packages:

### [Solver](./solver/)
Solver for puzzles

#### Example
```
cargo run -p solver -- < some_task.json
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

## Input formats

### Task file

A JSON object that contains the tasks basic information and given objects (deposits and obstacles).

### Cli file

A JSON array that contains two values.
1. A task object as described above
2. An array of objects that are part of the solution
