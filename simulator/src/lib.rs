use std::collections::{HashMap, VecDeque};

use model::{
    coord::neighbours,
    map::Map,
    object::{Object, ObjectCell, ObjectID},
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

    // Map from deposit to its resources
    let mut resources: HashMap<ObjectID, u32> = map
        .get_objects()
        .filter_map(|obj| match obj {
            Object::Deposit { width, height, .. } => {
                Some((obj.id(), *width as u32 * *height as u32 * 5))
            }
            _ => None,
        })
        .collect();

    // Map from objectID to amount of resources that object currently holds
    let mut resource_distribution: HashMap<ObjectID, Vec<u32>> = map
        .get_objects()
        .map(|obj| (obj.id(), vec![0; 10]))
        .collect();

    let objects: HashMap<ObjectID, &Object> =
        map.get_objects().map(|obj| (obj.id(), obj)).collect();

    let mut best_turn = 0;
    for turn in 1..=task.turns {
        // START OF ROUND

        let mut queue = objects
            .iter()
            .filter(|(_, object)| matches!(*object, Object::Factory { .. }))
            .map(|(id, object)| (*id, *object))
            .collect::<VecDeque<(ObjectID, &Object)>>();

        // try to *pull* resources at ingresses
        while let Some((object_id, object)) = queue.pop_front() {
            // skip mines - mines dont 'pull' their resources, because deposits push them
            // at the *end of the turn* into the mines
            if matches!(object, Object::Deposit { .. }) {
                continue;
            }

            let mut resources_incoming = vec![0; 10];

            for (x, y) in object.ingresses().iter() {
                for (nx, ny) in neighbours(*x, *y) {
                    if let Some(ObjectCell::Exgress {
                        id: id_incoming, ..
                    }) = map.get_cell(nx, ny)
                    {
                        // move resources
                        let incoming_resources = resource_distribution[id_incoming].clone();
                        for (resource_index, value) in resource_distribution
                            .get_mut(&object_id)
                            .unwrap()
                            .iter_mut()
                            .enumerate()
                        {
                            *value += incoming_resources[resource_index];
                            resources_incoming[resource_index] +=
                                incoming_resources[resource_index];
                        }
                        resource_distribution.insert(*id_incoming, vec![0; 10]);

                        // enqueue next object
                        queue.push_back((*id_incoming, objects[id_incoming]));
                    }
                }
            }

            let (x, y) = object.coords();

            if resources_incoming.iter().any(|value| *value > 0) && !quiet {
                println!(
                    "{} (start): ({},{}) accepts [{}], holds [{}]",
                    turn,
                    x,
                    y,
                    pretty_format_resources(&resources_incoming),
                    pretty_format_resources(&resource_distribution[&object_id]),
                );
            }
        }

        // END OF ROUND

        let deposits = objects
            .iter()
            .filter(|(_, object)| matches!(object, Object::Deposit { .. }));

        for (deposit_id, deposit) in deposits {
            let resource_type = deposit
                .subtype()
                .expect("Invalid deposit: must have subtype")
                as usize;

            // let mut resources_outgoing = vec![0; 10];

            for (x, y) in deposit.exgresses().iter() {
                for (nx, ny) in neighbours(*x, *y) {
                    if let Some(ObjectCell::Ingress {
                        id: id_receiving, ..
                    }) = map.get_cell(nx, ny)
                    {
                        let receiving_object = &objects[id_receiving];

                        if let Object::Mine { .. } = receiving_object {
                            let amount = resources[deposit_id].min(3);
                            // resource_distribution[deposit_id][resource_type] += amount;
                            let deposits_resources =
                                resource_distribution.get_mut(deposit_id).unwrap();
                            deposits_resources[resource_type] += amount;

                            if let Some(r) = resources.get_mut(deposit_id) {
                                *r -= amount;
                            }

                            let coords = deposit.coords();

                            if amount > 0 && !quiet {
                                println!(
                                    "{} (end): ({}, {}) takes [{}:{}], [{}:{}] available",
                                    turn,
                                    coords.0,
                                    coords.1,
                                    resource_type,
                                    amount,
                                    resource_type,
                                    resources.get(deposit_id).unwrap()
                                );
                            }
                        }
                    }
                }
            }
        }

        let factories = objects
            .iter()
            .filter(|(_, object)| matches!(object, Object::Factory { .. }));

        for (factory_id, object) in factories {
            if let Object::Factory { subtype, .. } = object {
                let factory_resources = resource_distribution.get_mut(factory_id).unwrap();
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

                        let (x, y) = object.coords();

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
