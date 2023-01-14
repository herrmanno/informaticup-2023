use std::{
    cell::RefCell,
    collections::VecDeque,
    ops::DerefMut,
    rc::Rc,
    time::{Duration, Instant},
};

use fxhash::FxHashMap as HashMap;

use common::debug;
use model::{
    coord::Point,
    map::Map,
    object::{Coord, Object, ObjectCell, ObjectID, Subtype},
    task::{Product, Task},
};

use crate::{path::Path, paths::Paths};
use rand::{distributions::WeightedIndex, prelude::Distribution, seq::SliceRandom, Rng};
use simulator::{simulate, SimulatorResult};

/// Number of times a factory location is tried.
/// If no location can be found a whole new iteration starts
const NUM_MAX_FACTORY_PLACEMENTS: u32 = 100;

/// Chance that a single factory will be skipped during placement
const PROBABILITY_FACTORY_SKIP: (u32, u32) = (1, 10);

/// Number of paths to try (calculate) per factory and resource type
const NUM_PATHS_PER_FACTORY_AND_RESOURCE: u32 = 15;

/// Number of additional paths to try (calculate) per factory and resource type
const NUM_ADDITION_PATHS_PER_FACTORY_AND_RESOURCE: u32 = 10;

/// Number of path combinations to try during one iteration
const NUM_PATH_COMBINING_ITERATIONS: u32 = 10;

/// Max number of BFS states to process when finding a path
#[allow(dead_code)] //TODO: remove
const NUM_MAX_PATH_FINDING_STEPS: u32 = 100_000;

#[derive(Clone)]
pub struct Solver<'a, T> {
    task: &'a Task,
    original_map: &'a Map,
    deposits_by_type: HashMap<Subtype, Vec<Object>>,
    products: Vec<Product>,
    best_factory_positions_by_factory_subtype: HashMap<Subtype, (WeightedIndex<f32>, Vec<Point>)>,
    rng: Rc<RefCell<T>>,
    max_iteration_time: Duration,
    #[allow(unused)] //only used if feature 'stats' is active
    num_solutions: usize,
}

impl<'a, T> Solver<'a, T> {
    #[cfg(feature = "stats")]
    /// Returns the total number of solutions produced so far
    pub fn get_num_solutions(&self) -> usize {
        self.num_solutions
    }
}

impl<'a, T: Rng> Solver<'a, T> {
    pub fn new(
        task: &'a Task,
        map: &'a Map,
        rng: Rc<RefCell<T>>,
        max_iteration_time: Duration,
    ) -> Solver<'a, T> {
        let deposits_by_type: HashMap<u8, Vec<Object>> = {
            let mut deposits: HashMap<u8, Vec<Object>> = HashMap::default();
            task.objects
                .iter()
                .cloned()
                .map(Object::from)
                .for_each(|obj| {
                    if let Object::Deposit { subtype, .. } = obj {
                        deposits.entry(subtype).or_default().push(obj)
                    }
                });

            deposits
        };

        let deposits_by_product: HashMap<u8, Vec<Object>> = {
            let mut deposits: HashMap<u8, Vec<Object>> = HashMap::default();
            task.products.iter().for_each(|product| {
                product
                    .resources
                    .iter()
                    .enumerate()
                    .filter(|&(_, &amount)| amount > 0)
                    .flat_map(|(resource_index, _)| {
                        deposits_by_type[&(resource_index as u8)].iter()
                    })
                    .cloned()
                    .for_each(|deposit_object| {
                        deposits
                            .entry(product.subtype)
                            .or_default()
                            .push(deposit_object);
                    });
            });

            deposits
        };

        let possible_factory_locations = find_possible_factory_positions(map);

        // FIXME: use (try) distance map for choosing best factory positions
        let best_factory_positions_by_factory_subtype: HashMap<
            Subtype,
            (WeightedIndex<f32>, Vec<Point>),
        > = task
            .products
            .iter()
            .map(|product| {
                let factory_type = product.subtype;
                let deposits = &deposits_by_product[&factory_type];
                let (probabilities, best_positions) =
                    sort_to_best_positions_by_deposits(&possible_factory_locations, deposits);
                (factory_type, (probabilities, best_positions))
            })
            .collect();

        let products: Vec<Product> = task.products.to_vec();

        Solver {
            task,
            original_map: map,
            deposits_by_type,
            products,
            best_factory_positions_by_factory_subtype,
            rng,
            max_iteration_time,
            num_solutions: 0,
        }
    }
}

impl<'a, T: Rng> Iterator for Solver<'a, T> {
    type Item = (SimulatorResult, Map);

    fn next(&mut self) -> Option<Self::Item> {
        let Solver {
            task,
            original_map,
            deposits_by_type,
            products,
            best_factory_positions_by_factory_subtype,
            ref rng,
            max_iteration_time,
            ..
        } = self;

        let time_start = Instant::now();

        debug!("{}", original_map);

        // start iterating

        let mut best_solution: Option<(SimulatorResult, Map)> = None;

        #[allow(unused_variables)]
        'iterate: for n_iteration in 1.. {
            if time_start.elapsed() > *max_iteration_time {
                return None;
            }

            debug!("Starting iteration #{}", n_iteration);

            let mut map = original_map.clone();

            // place factories

            let mut factory_ids = Vec::new();

            // Shuffle products to place factories in different order/priority each iteration
            products.shuffle(rng.borrow_mut().deref_mut());

            'factory_placement: for product in products.iter() {
                // skip a factory with some probability to try solutions where not all factories are used
                if (**rng)
                    .borrow_mut()
                    .gen_ratio(PROBABILITY_FACTORY_SKIP.0, PROBABILITY_FACTORY_SKIP.1)
                {
                    continue 'factory_placement;
                }

                let factory_type = product.subtype;
                let (factory_location_distribution, factory_locations) =
                    &best_factory_positions_by_factory_subtype[&factory_type];

                for _ in 0..NUM_MAX_FACTORY_PLACEMENTS {
                    let factory_location = factory_locations
                        [factory_location_distribution.sample(rng.borrow_mut().deref_mut())];

                    // TODO: check that for each required resource type, a deposit of such type is
                    // reachable (simple path finding) from this factory location
                    let factory = Object::Factory {
                        x: factory_location.0,
                        y: factory_location.1,
                        subtype: product.subtype,
                    };
                    let factory_id = factory.id();

                    if map.insert_object(factory).is_ok() {
                        // TODO: update factory_positions weights, so that conflicting positions can not be picked anymore
                        factory_ids.push(factory_id);
                        continue 'factory_placement;
                    }
                }

                // TODO: disallow already set factories
                continue 'iterate;
            }

            if factory_ids.is_empty() {
                continue 'iterate;
            }

            debug!("Factories placed");
            debug!("{}", map);

            // construct factory -> deposit paths

            // chose path combinations

            // Map from factory subtype => (map of resource type => built path)
            let mut built_paths_by_factory: HashMap<Subtype, HashMap<Subtype, Path>> =
                HashMap::default();

            #[allow(unused_variables)]
            'combining_paths: for n_combining_paths in 0..NUM_PATH_COMBINING_ITERATIONS {
                debug!("Combining paths #{}", n_combining_paths);

                factory_ids.shuffle(rng.borrow_mut().deref_mut());

                for &factory_id in factory_ids.iter() {
                    let factory = map.get_object(factory_id).clone(); //clone, so 'map' is borrowed for the scope of the loop
                    let subtype = factory.subtype().unwrap();
                    let product = task // TODO: use lookup table
                        .products
                        .iter()
                        .find(|product| product.subtype == subtype)
                        .unwrap_or_else(|| {
                            panic!(
                                "No product found for subtype {} but a factory is present",
                                subtype
                            )
                        });

                    let mut resources: VecDeque<Subtype> = product
                        .resources
                        .iter()
                        .enumerate()
                        .filter_map(|(index, amount)| {
                            if *amount > 0 {
                                Some(index as Subtype)
                            } else {
                                None
                            }
                        })
                        .collect();

                    resources
                        .make_contiguous()
                        .shuffle(rng.borrow_mut().deref_mut());

                    let mut processed_resources: VecDeque<Subtype> = VecDeque::new();

                    let mut paths_by_resource: HashMap<Subtype, Option<Paths<T>>> =
                        resources.iter().map(|resource| (*resource, None)).collect();

                    let mut built_paths_by_resource: HashMap<Subtype, Path> = HashMap::default();

                    'path_building: while let Some(resource) = resources.pop_front() {
                        debug!(
                            "Try to find path from factory {} to resource {}",
                            factory.subtype().unwrap(),
                            resource
                        );

                        /* LOGIC
                        1a. If no path to resource built yet:
                            - Built and store paths for resource, based on already built paths
                            - Choose first valid of such paths
                        1b. Else:
                            - Choose the next valid path from prebuilt paths
                        2. Build and store the choosen path
                        3a. If no path can be choosen:
                            - push back resource and also push top of 'done' stack
                        3b. Else:
                            - pop resource and push it onto 'done' stack
                        */

                        let available_paths = paths_by_resource
                            .entry(resource)
                            .and_modify(|paths| {
                                if paths.is_none() {
                                    let start_points = {
                                        let mut start_points = factory.ingresses().to_vec();
                                        for path in built_paths_by_resource.values() {
                                            for ingress in path.all_ingresses() {
                                                start_points.push(ingress);
                                            }
                                        }
                                        start_points
                                    };
                                    *paths = Some(Paths::new(
                                        &start_points,
                                        &deposits_by_type[&resource],
                                        &map, //FIXME: pre-built deposit_distance map once and pass it here because 'map' does not change during loop
                                        Rc::clone(&self.rng),
                                    ));
                                }
                            })
                            .or_default();

                        // FIXME: 'paths_tried' should be remembered for this resource
                        if let Some(available_paths) = available_paths {
                            for (paths_tried, path) in available_paths.by_ref().enumerate() {
                                if paths_tried as u32 > NUM_PATHS_PER_FACTORY_AND_RESOURCE {
                                    break; // go to backtrack
                                }

                                if map
                                    .try_insert_objects(path.objects().cloned().collect())
                                    .is_ok()
                                {
                                    built_paths_by_resource.insert(resource, path);
                                    processed_resources.push_back(resource);
                                    continue 'path_building;
                                }
                            }
                        }

                        // backtrack
                        *available_paths = None;
                        built_paths_by_resource.remove(&resource);

                        resources.push_front(resource);
                        if let Some(prior_resource) = processed_resources.pop_back() {
                            resources.push_front(prior_resource);
                        } else {
                            continue 'combining_paths;
                        }
                    }

                    built_paths_by_factory.insert(subtype, built_paths_by_resource);

                    debug!("Initial paths built");
                    debug!("{}", map);
                }

                // map = work_map;
                break 'combining_paths;
            }

            if built_paths_by_factory.is_empty() {
                debug!("Could not build initial paths");
                continue 'iterate;
            }

            // Prepare weights for building additional paths

            let mut factory_resource_pairs: Vec<(ObjectID, Subtype)> = Vec::new();
            let mut factory_resource_weights_raw: Vec<u32> = Vec::new();
            for &factory_id in factory_ids.iter() {
                let factory = map.get_object(factory_id);
                let subtype = factory.subtype().unwrap();
                let product = &products[subtype as usize];
                for (resource_index, resource_amount) in product
                    .resources
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, amount)| {
                        if *amount > 0 {
                            Some((idx, amount))
                        } else {
                            None
                        }
                    })
                {
                    let key = (factory_id, resource_index as Subtype);
                    factory_resource_pairs.push(key);
                    let weight = resource_amount * product.points;
                    factory_resource_weights_raw.push(weight);
                }
            }
            let mut factory_resource_weights =
                WeightedIndex::new(factory_resource_weights_raw.clone())
                    .expect("Cannot built (factory,resource) weights");

            debug!("Building additional paths");

            // TODO: investigate optimal number of failed tries per factory/resource tuple
            let max_additional_path_failures = factory_ids.len() * 10;
            let mut additional_path_failures = 0;
            'additional_paths: loop {
                let factory_resource_pair_index =
                    factory_resource_weights.sample(rng.borrow_mut().deref_mut());

                let (factory_id, resource_index) =
                    factory_resource_pairs[factory_resource_pair_index];
                let factory = map.get_object(factory_id);

                debug!(
                    "Try building path from factory {} to resource {}",
                    factory.subtype().unwrap(),
                    resource_index
                );

                let built_paths_by_resource = built_paths_by_factory
                    .entry(factory.subtype().unwrap())
                    .or_default();
                let start_points = {
                    let mut start_points = factory.ingresses();
                    for path in built_paths_by_resource.values() {
                        for ingress in path.all_ingresses() {
                            start_points.push(ingress);
                        }
                    }
                    start_points
                };

                #[allow(unused_variables)]
                let mut i = 1;
                for path in Paths::new(
                    &start_points,
                    &deposits_by_type[&resource_index],
                    &map, //FIXME: prebuilt 'deposit_distance_map' here and pass it to Paths
                    Rc::clone(&self.rng),
                )
                .take(NUM_ADDITION_PATHS_PER_FACTORY_AND_RESOURCE as usize)
                {
                    debug!("Checking path #{}", i);
                    i += 1;
                    if map
                        .try_insert_objects(path.objects().cloned().collect())
                        .is_ok()
                    {
                        built_paths_by_resource.insert(resource_index, path);
                        debug!("{}", map);
                        continue 'additional_paths;
                    }
                }

                // Reduce weight of current factory,resource tuple
                let new_weight = &mut factory_resource_weights_raw[factory_resource_pair_index];
                *new_weight /= 2; //TODO: try to use some kind of exponential backoff
                let _ = factory_resource_weights
                    .update_weights(&[(factory_resource_pair_index, new_weight)]);

                additional_path_failures += 1;

                if additional_path_failures > max_additional_path_failures {
                    break 'additional_paths;
                }
            }

            debug!("Additional paths built");
            debug!("{}", map);

            let map_score = simulate(task, &map, true);

            #[cfg(feature = "stats")]
            {
                self.num_solutions += 1;
            }

            if let Some((result, _)) = &best_solution {
                if map_score > *result {
                    debug!("{:?}", map_score);
                    debug!("{}", map);
                    best_solution = Some((map_score, map));
                    return best_solution;
                }
            } else if map_score.score > 0 {
                debug!("{:?}", map_score);
                debug!("{}", map);
                best_solution = Some((map_score, map));
                return best_solution;
            };
        }

        None
    }
}

/// Finds all locations, at which a 5x5 factory could be legally placed
fn find_possible_factory_positions(map: &Map) -> Vec<Point> {
    let width = map.width() as Coord;
    let height = map.height() as Coord;

    let free_cells = {
        let mut v = vec![];
        for y in 0..height {
            for x in 0..width {
                if map.get_cell(x, y).is_none() {
                    v.push((x, y));
                }
            }
        }
        v
    };

    let mut positions = Vec::new();

    'lopp_cells: for (x, y) in free_cells {
        if x + 4 >= width || y + 4 >= height {
            continue 'lopp_cells;
        }

        let min_x = if x == 0 { 0 } else { x - 1 };
        let min_y = if y == 0 { 0 } else { y - 1 };
        for dx in x..x + 5 {
            for dy in y..y + 5 {
                if let Some(ObjectCell::Inner { .. }) = map.get_cell(dx, dy) {
                    continue 'lopp_cells;
                }
            }
        }
        for dx in min_x..=x + 5 {
            for dy in [min_y, y + 5] {
                if let Some(ObjectCell::Exgress { .. }) = map.get_cell(dx, dy) {
                    continue 'lopp_cells;
                }
            }
        }

        for dy in min_y..=y + 5 {
            for dx in [min_x, x + 5] {
                if let Some(ObjectCell::Exgress { .. }) = map.get_cell(dx, dy) {
                    continue 'lopp_cells;
                }
            }
        }

        positions.push((x, y))
    }

    positions
}

fn sort_to_best_positions_by_deposits(
    positions: &[Point],
    deposits: &[Object],
) -> (WeightedIndex<f32>, Vec<Point>) {
    let mut positions_with_distances: Vec<(i32, &Point)> = positions
        .iter()
        .map(|position| {
            // TODO: weight deposit (resource types) by importance for product
            let distances = deposits
                .iter()
                .map(|deposit| {
                    let (x, y) = position;
                    let (dx, dy) = deposit.coords();
                    // TODO: use path distance instead of manhattan distance (see task 004)
                    (x - dx).abs() as i32 + (y - dy).abs() as i32
                })
                .collect::<Vec<i32>>();

            let sum = distances.iter().sum::<i32>();
            let mean_distance = sum / distances.len() as i32;
            let deviation = distances
                .iter()
                .map(|&i| (i - mean_distance).abs())
                .sum::<i32>();

            let distance = sum + deviation;

            (distance, position)
        })
        .collect();

    positions_with_distances.sort_unstable_by_key(|(distance, _)| *distance);

    let probabilites: Vec<f32> = positions_with_distances
        .iter()
        .map(|(distance, _)| 1f32 / (*distance).max(1) as f32)
        .collect();

    let weights =
        WeightedIndex::new(probabilites).expect("Cannot build weights from factory locations");

    let positions: Vec<Point> = positions_with_distances
        .into_iter()
        .map(|(_, position)| position)
        .cloned()
        .collect();

    (weights, positions)
}
