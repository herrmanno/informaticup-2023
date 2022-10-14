# InformatiCup 2023

## Packages

The project consists of multiple packages:

### [Printer](./printer/)
Binary to read task and solution files and print resulting map

#### Example
```
cargo run -p printer -- --cli inputs/custom001.cli.json
```

### [Simulation](./simulation/)
Binary for running a simulation from a task/solution file

#### Example
```
cargo run -p simulator -- --cli simulator/inputs/test1.json
```

### [Model](./model/)
Model instances for task/solution files and basic building objects