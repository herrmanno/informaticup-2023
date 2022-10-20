use std::collections::{HashMap, VecDeque};

use model::{
    coord::neighbours,
    map::{Map, MapObject},
    object::{Object, ObjectCell},
    solution::Solution,
    task::{Product, Task},
};

/// Result of simulating a mpa
#[derive(Debug, PartialEq, Eq)]
pub struct SimulatorResult {
    /// The final score
    pub score: u32,
    /// The turn, the final score was reached
    pub turn: u32,
}

impl PartialOrd for SimulatorResult {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SimulatorResult {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering::*;

        let score_cmp = self.score.cmp(&other.score);
        let turn_cmp = self.turn.cmp(&other.turn);

        match (score_cmp, turn_cmp) {
            (Greater, _) => Greater,
            (Less, _) => Less,
            (Equal, Less) => Greater,
            (Equal, Greater) => Less,
            _ => Equal,
        }
    }
}

/// Runs a simulation of a task and a given solution map
pub fn simulate(task: &Task, map: &Map, quiet: bool) -> SimulatorResult {
    let products_by_type = task
        .products
        .iter()
        .map(|product| (product.subtype, product))
        .collect::<HashMap<u8, &Product>>();

    let mut score = 0;

    let mut resources: HashMap<usize, u32> = map
        .get_objects()
        .iter()
        .enumerate()
        .filter_map(|(index, obj)| match obj.object {
            Object::Deposit { width, height, .. } => {
                Some((index, width as u32 * height as u32 * 5))
            }
            _ => None,
        })
        .collect();

    let mut resource_distribution: Vec<Vec<u32>> =
        map.get_objects().iter().map(|_| vec![0; 10]).collect();

    let objects = map.get_objects();

    let mut best_turn = 0;
    for turn in 1..=task.turns {
        // START OF ROUND

        let mut queue = objects
            .iter()
            .enumerate()
            .filter(|(_, map_object)| matches!(map_object.object, Object::Factory { .. }))
            .collect::<VecDeque<(usize, &MapObject)>>();

        // try to *pull* resources at ingresses
        while let Some((index, object)) = queue.pop_front() {
            // skip mines - mines dont 'pull' their resources, because deposits push them
            // at the *end of the turn* into the mines
            if matches!(object.object, Object::Deposit { .. }) {
                continue;
            }

            let mut resources_incoming = vec![0; 10];

            for (x, y) in object.ingresses.iter() {
                for (nx, ny) in neighbours(*x, *y) {
                    if let Some(ObjectCell::Exgress {
                        index: index_incoming,
                        ..
                    }) = map.get_cell(nx, ny)
                    {
                        // move resources
                        let incoming_resources = resource_distribution[*index_incoming].clone();
                        for (resource_index, value) in
                            resource_distribution[index].iter_mut().enumerate()
                        {
                            *value += incoming_resources[resource_index];
                            resources_incoming[resource_index] +=
                                incoming_resources[resource_index];
                        }
                        resource_distribution[*index_incoming] = vec![0; 10];

                        // enqueue next object
                        queue.push_back((*index_incoming, &objects[*index_incoming]));
                    }
                }
            }

            let (x, y) = object.object.coords();

            if resources_incoming.iter().any(|value| *value > 0) && !quiet {
                println!(
                    "{} (start): ({},{}) accepts [{}], holds [{}]",
                    turn,
                    x,
                    y,
                    pretty_format_resources(&resources_incoming),
                    pretty_format_resources(&resource_distribution[index]),
                );
            }
        }

        // END OF ROUND

        let deposits = objects
            .iter()
            .enumerate()
            .filter(|(_, map_object)| matches!(map_object.object, Object::Deposit { .. }));

        for (index, map_object) in deposits {
            let resource_type = map_object
                .object
                .subtype()
                .expect("Invalid deposit: must have subtype")
                as usize;

            // let mut resources_outgoing = vec![0; 10];

            for (x, y) in map_object.exgresses.iter() {
                for (nx, ny) in neighbours(*x, *y) {
                    if let Some(ObjectCell::Ingress {
                        index: index_receiving,
                        ..
                    }) = map.get_cell(nx, ny)
                    {
                        let receiving_object = &objects[*index_receiving];

                        if let Object::Mine { .. } = receiving_object.object {
                            let amount = resources[&index].min(3);
                            resource_distribution[index][resource_type] += amount;
                            // resources_outgoing[resource_type] += amount;
                            if let Some(r) = resources.get_mut(&index) {
                                *r -= amount;
                            }

                            let coords = map_object.object.coords();

                            if amount > 0 && !quiet {
                                println!(
                                    "{} (end): ({}, {}) takes [{}:{}], [{}:{}] available",
                                    turn,
                                    coords.0,
                                    coords.1,
                                    resource_type,
                                    amount,
                                    resource_type,
                                    resources.get(&index).unwrap()
                                );
                            }
                        }
                    }
                }
            }
        }

        let factories = objects
            .iter()
            .enumerate()
            .filter(|(_, map_object)| matches!(map_object.object, Object::Factory { .. }));

        for (index, object) in factories {
            if let Object::Factory { subtype, .. } = &objects[index].object {
                let factory_resources = &mut resource_distribution[index];
                if let Some(&product) = products_by_type.get(subtype) {
                    let can_produce = product.resources.iter().enumerate().all(
                        |(resource_index, resource_amount)| {
                            factory_resources[resource_index] >= *resource_amount
                        },
                    );

                    if can_produce {
                        score += product.points;
                        for (resource_index, amount) in product.resources.iter().enumerate() {
                            factory_resources[resource_index] -= amount;
                        }

                        let (x, y) = object.object.coords();

                        if !quiet {
                            println!(
                                "{} (end): ({}, {}) produces {} ({} points)",
                                turn, x, y, subtype, product.points
                            );
                        }

                        best_turn = turn;
                    }
                } else {
                    panic!(
                        "no product for subtype {} known but a factory exists",
                        subtype
                    );
                }
            }
        }
    }

    SimulatorResult {
        score,
        turn: best_turn,
    }
}

pub fn generate_map(task: &Task, solution: &Solution) -> Map {
    let mut objects = Vec::with_capacity(task.objects.len() + solution.0.len());
    objects.extend(task.objects.clone().into_iter().map(Object::from));
    objects.extend(solution.0.iter().cloned().map(Object::from));

    Map::new(task.width, task.height, objects)
}

fn pretty_format_resources(resources: &[u32]) -> String {
    resources
        .iter()
        .enumerate()
        .filter(|(_, &value)| value > 0)
        .map(|(index, value)| format!("{}:{}", index, value))
        .reduce(|a, b| format!("{}, {}", a, b))
        .unwrap_or_else(|| "".to_string())
}
