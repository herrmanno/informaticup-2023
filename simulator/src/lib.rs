use std::{cell::RefCell, collections::VecDeque};

use fxhash::FxHashMap as HashMap;
use fxhash::FxHashSet as HashSet;

use model::map::new_map;
use model::{
    coord::neighbours,
    map::Maplike,
    object::Object,
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
pub fn simulate(task: &Task, map: &impl Maplike, quiet: bool) -> SimulatorResult {
    let products_by_type = task
        .products
        .iter()
        .map(|product| (product.subtype, product))
        .collect::<HashMap<u8, &Product>>();

    let mut score = 0;

    // Map from deposit to its resources
    let mut resources: HashMap<&Object, u32> = map
        .get_objects()
        .filter_map(|obj| match obj {
            Object::Deposit { width, height, .. } => {
                Some((obj, *width as u32 * *height as u32 * 5))
            }
            _ => None,
        })
        .collect();

    // Map from objectID to amount of resources that object currently holds
    let mut resource_distribution: HashMap<&Object, RefCell<Vec<u32>>> = map
        .get_objects()
        .map(|obj| (obj, RefCell::new(vec![0; 8])))
        .collect();

    let mut best_turn = 0;
    for turn in 1..=task.turns {
        // START OF ROUND

        let mut queue = map
            .get_objects()
            .filter(|object| matches!(object, &Object::Factory { .. }))
            .collect::<VecDeque<&Object>>();

        // try to *pull* resources at ingresses
        while let Some(object) = queue.pop_front() {
            // skip mines - mines dont 'pull' their resources, because deposits push them
            // at the *end of the turn* into the mines
            if matches!(object, Object::Deposit { .. }) {
                continue;
            }

            let mut resources_incoming = vec![0; 8];

            for (x, y) in object.ingresses().iter() {
                for (nx, ny) in neighbours(*x, *y) {
                    if let Some(outgoing_object) = map.get_object_with_exgress_at(nx, ny) {
                        // move resources
                        for (resource_index, value) in resource_distribution
                            .get(object)
                            .unwrap()
                            .borrow_mut()
                            .iter_mut()
                            .enumerate()
                        {
                            let outgoing_resource = &mut resource_distribution
                                .get(outgoing_object)
                                .unwrap()
                                .borrow_mut()[resource_index];

                            let amount = match object {
                                Object::Mine { .. } => (*outgoing_resource).min(3),
                                _ => *outgoing_resource,
                            };

                            *value += amount;
                            *outgoing_resource -= amount;
                            resources_incoming[resource_index] += amount;
                        }

                        // enqueue next object
                        queue.push_back(outgoing_object);
                    }
                }
            }

            let (x, y) = object.coords();

            if resources_incoming.iter().any(|value| *value > 0) && !quiet {
                println!(
                    "{} (start): ({}, {}) accepts [{}], holds [{}]",
                    turn,
                    x,
                    y,
                    pretty_format_resources(&resources_incoming),
                    pretty_format_resources(&resource_distribution[object].borrow()),
                );
            }
        }

        // END OF ROUND

        let deposits = map
            .get_objects()
            .filter(|object| matches!(object, Object::Deposit { .. }));

        for deposit in deposits {
            let resource_type = deposit
                .subtype()
                .expect("Invalid deposit: must have subtype")
                as usize;

            let mut visited_cells = HashSet::default();

            for (x, y) in deposit.exgresses().iter() {
                for (nx, ny) in neighbours(*x, *y) {
                    if visited_cells.contains(&(nx, ny)) {
                        continue;
                    }

                    visited_cells.insert((nx, ny));

                    if let Some(Object::Mine { .. }) = map.get_object_with_ingress_at(nx, ny) {
                        let amount = resources[deposit].min(3);
                        let deposits_resources = resource_distribution.get_mut(deposit).unwrap();
                        deposits_resources.borrow_mut()[resource_type] += amount;

                        if let Some(r) = resources.get_mut(deposit) {
                            *r -= amount;
                        }

                        let coords = deposit.coords();

                        if amount > 0 && !quiet {
                            println!(
                                "{} (end): ({}, {}) takes [{}x{}], [{}x{}] available",
                                turn,
                                coords.0,
                                coords.1,
                                amount,
                                resource_type,
                                resources.get(deposit).unwrap(),
                                resource_type,
                            );
                        }
                    }
                }
            }
        }

        let factories = map
            .get_objects()
            .filter(|object| matches!(object, Object::Factory { .. }));

        for factory in factories {
            if let Object::Factory { subtype, .. } = factory {
                let factory_resources = resource_distribution.get_mut(factory).unwrap();
                if let Some(&product) = products_by_type.get(subtype) {
                    'produce_loop: loop {
                        let can_produce = product.resources.iter().enumerate().all(
                            |(resource_index, resource_amount)| {
                                factory_resources.borrow_mut()[resource_index] >= *resource_amount
                            },
                        );

                        if can_produce {
                            score += product.points;
                            for (resource_index, amount) in product.resources.iter().enumerate() {
                                factory_resources.borrow_mut()[resource_index] -= amount;
                            }

                            let (x, y) = factory.coords();

                            if !quiet {
                                println!(
                                    "{} (end): ({}, {}) produces {} ({} points)",
                                    turn, x, y, subtype, product.points
                                );
                            }

                            best_turn = turn;
                        } else {
                            break 'produce_loop;
                        }
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

pub fn generate_map(task: &Task, solution: &Solution) -> impl Maplike {
    let mut objects = Vec::with_capacity(task.objects.len() + solution.0.len());
    objects.extend(task.objects.clone().into_iter().map(Object::from));
    objects.extend(solution.0.iter().cloned().map(Object::from));

    new_map(task.width, task.height, objects)
}

fn pretty_format_resources(resources: &[u32]) -> String {
    resources
        .iter()
        .enumerate()
        .filter(|(_, &value)| value > 0)
        .map(|(index, value)| format!("{}x{}", value, index))
        .reduce(|a, b| format!("{}, {}", a, b))
        .unwrap_or_else(|| "".to_string())
}
