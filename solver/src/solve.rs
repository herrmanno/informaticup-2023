use std::collections::{HashMap, VecDeque};

use common::debug;
use model::{
    coord::Point,
    map::Map,
    object::{Coord, Object, ObjectCell, Subtype},
    task::{Product, Task},
};

use crate::{path::Path, paths::Paths};
use rand::{distributions::WeightedIndex, prelude::Distribution, seq::SliceRandom, thread_rng};
use simulator::{simulate, SimulatorResult};

/// Number of whole iterations
const NUM_ITERATIONS: u32 = 50;

/// Number of times a factory location is tried.
/// If no location can be found a whole new iteration starts
const NUM_MAX_FACTORY_PLACEMENTS: u32 = 100;

/// Number of pre-calculated paths per factory (position) and resource type
const NUM_PATHS_PER_FACTORY_AND_RESOURCE: u32 = 15;

/// Number of path combinations to try during one iteration
const NUM_PATH_COMBINING_ITERATIONS: u32 = 10;

/// Max number of BFS states to process when finding a path
const NUM_MAX_PATH_FINDING_STEPS: u32 = 100_000;

pub fn solve<'a, 'b>(task: &'a Task, original_map: &'b mut Map) -> Option<(SimulatorResult, Map)> {
    let initial_object_count = original_map.get_objects().count();

    // prepare helper state that is useful for remaining algorithm
    let deposits_by_type: HashMap<u8, Vec<Object>> = {
        let mut deposits: HashMap<u8, Vec<Object>> = HashMap::new();
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
        let mut deposits: HashMap<u8, Vec<Object>> = HashMap::new();
        task.products.iter().for_each(|product| {
            product
                .resources
                .iter()
                .enumerate()
                .filter(|&(_, &amount)| amount > 0)
                .flat_map(|(resource_index, _)| deposits_by_type[&(resource_index as u8)].iter())
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

    let possible_factory_locations = find_possible_factory_positions(original_map);

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

    let mut products: Vec<Product> = task.products.to_vec();

    debug!("{}", original_map);

    /*
       IDEA
       ================

       Before iterating:
       - construct a list of best factory locations for each factory (=product) type needed

       For each iteration:
       - for each factory type:
           - pick a factory position with probability equal to its 'value' in list of best positions,
             where value means distance to all deposits. So if there are three possible positions
             p_0, p_1, p_2, with distances 20, 30, 50 respectively, the probability to pick
              p_0 = (20 + 30 + 50) / 20
              p_1 = (20 + 30 + 50) / 30
              p_2 = (20 + 30 + 50) / 50
              normalised.
       - place factory combination on a tabu list
       - for each factory f:
           - for each resource type r:
               - paths_f_r := create iterator of shortest paths from factory to resource
       - do n times:
           - for each factory f:
               - for each resource type r:
                   - pick `path` from paths_f_r with index between 0..n (with descending probability?)
                   - place `path`
                       - if failure continue `do n times`-loop
           - store result
           - (try to generate even more paths)
    */

    let mut rng = thread_rng();

    // start iterating

    let mut best_solution: Option<(SimulatorResult, Map)> = None;

    'iterate: for n_iteration in 1..=NUM_ITERATIONS {
        debug!("Starting iteration #{}", n_iteration);

        let mut map = original_map.clone();

        // place factories

        let mut factory_ids = Vec::new();

        // Shuffle products to place factories in different order/priority each iteration
        products.shuffle(&mut rng);

        'factory_placement: for product in products.iter() {
            let factory_type = product.subtype;
            let (factory_location_distribution, factory_locations) =
                &best_factory_positions_by_factory_subtype[&factory_type];

            for _ in 0..NUM_MAX_FACTORY_PLACEMENTS {
                let factory_location =
                    factory_locations[factory_location_distribution.sample(&mut rng)];

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

        debug!("Factories placed");
        debug!("{}", map);

        // construct factory -> deposit paths

        // chose path combinations

        'combining_paths: for n_combining_paths in 0..NUM_PATH_COMBINING_ITERATIONS {
            debug!("Combining paths #{}", n_combining_paths);

            let mut work_map = map.clone();

            factory_ids.shuffle(&mut rng);

            for &factory_id in factory_ids.iter() {
                let factory = map.get_object(factory_id);
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

                resources.make_contiguous().shuffle(&mut rng);

                let mut processed_resources: VecDeque<Subtype> = VecDeque::new();

                let mut paths_by_resource: HashMap<Subtype, Paths> = resources
                    .iter()
                    .map(|resource| (*resource, Paths::default()))
                    .collect();

                let mut built_paths_by_resource: HashMap<Subtype, Path> = HashMap::new();

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
                            if paths.is_empty() {
                                let start_points = {
                                    let mut start_points = factory.ingresses().to_vec();
                                    for path in built_paths_by_resource.values() {
                                        for ingress in path.all_ingresses() {
                                            start_points.push(ingress);
                                        }
                                    }
                                    start_points
                                };
                                *paths = Paths::new(
                                    &start_points,
                                    resource,
                                    &deposits_by_type[&resource],
                                    &map,
                                );
                            }
                        })
                        .or_default();

                    // FIXME: 'paths_tried' should be remembered for this resource
                    for (paths_tried, path) in available_paths.by_ref().enumerate() {
                        if paths_tried as u32 > NUM_PATHS_PER_FACTORY_AND_RESOURCE {
                            break; // go to backtrack
                        }

                        if work_map
                            .try_insert_objects(path.objects().cloned().collect())
                            .is_ok()
                        {
                            built_paths_by_resource.insert(resource, path);
                            // debug!("{}", work_map);
                            processed_resources.push_back(resource);
                            continue 'path_building;
                        }
                    }

                    // backtrack
                    available_paths.clear();
                    built_paths_by_resource.remove(&resource);

                    resources.push_front(resource);
                    if let Some(prior_resource) = processed_resources.pop_back() {
                        resources.push_front(prior_resource);
                    } else {
                        continue 'combining_paths;
                    }
                }

                debug!("{}", map);
            }

            map = work_map;
            break 'combining_paths;
        }

        // FIXME: build additional path in descending product priority

        let map_score = simulate(task, &map, true);

        best_solution = if let Some((result, best_map)) = best_solution {
            if map_score > result {
                debug!("{:?}", map_score);
                debug!("{}", map);
                Some((map_score, map))
            } else {
                Some((result, best_map))
            }
        } else {
            debug!("{:?}", map_score);
            debug!("{}", map);
            Some((map_score, map))
        };
    }

    best_solution
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

/*
/// Constructs the shortest path from a factory to a deposit of subtype `resource_index`
fn build_shortest_paths_from_factory_to_deposit<R: Rng + ?Sized>(
    num_paths: u32,
    start_points: &[Point],
    resource_index: usize,
    deposits: &[Object],
    map: &Map,
    _rng: &mut R, //TODO: use (at least a little bit) of randomness when finding paths
) -> Vec<Path> {
    let mut num_found_paths = 0;

    let distances_to_deposits = build_distance_map_from_deposits(map, deposits);

    let min_distance_to_deposits = |points: &[Point]| {
        points
            .iter()
            .filter_map(|point| distances_to_deposits.get(point))
            .min()
            .cloned()
            .unwrap_or(u32::MAX)
    };

    let mut paths = Vec::with_capacity(num_paths as usize);
    let mut paths_so_far: HashSet<PathID> = HashSet::new();
    let mut queue: BinaryHeap<Reverse<(u32, u32, Rc<Path>)>> = BinaryHeap::new();

    for &ingress in start_points {
        let path = Path::from_starting_points(vec![ingress]);
        let distance = min_distance_to_deposits(&[ingress]);
        queue.push(Reverse((distance, 0, Rc::new(path))))
    }

    let mut num_iterations = 0;
    'bfs: while let Some(Reverse((_, path_length, path))) = queue.pop() {
        if num_iterations >= NUM_MAX_PATH_FINDING_STEPS {
            break 'bfs;
        }
        num_iterations += 1;

        for (x, y) in path.heads() {
            /*  LOGIC
                1. try if target is reached if a mine is placed
                2. try using long conveyor
                3. try using short conveyor
                4. try using combiner
            */

            let free_neighbours = neighbours(x, y)
                .into_iter()
                .filter(|(x, y)| map.get_cell(*x, *y).is_none());

            for (nx, ny) in free_neighbours {
                for mine_subtype in 0..=3 {
                    let mine = Object::mine_with_subtype_and_exgress_at(mine_subtype, (nx, ny));
                    let mine_ingress = mine.ingress().unwrap();
                    let mine_reaches_deposit = neighbours(mine_ingress.0, mine_ingress.1)
                        .into_iter()
                        .any(|(x, y)| match map.get_cell(x, y) {
                            Some(ObjectCell::Exgress { id, .. }) => {
                                let obj = map.get_object(*id);
                                obj.kind() == ObjectType::Deposit
                                    && obj.subtype() == Some(resource_index as u8)
                            }
                            _ => false,
                        });

                    if mine_reaches_deposit {
                        match map
                            .can_insert_object(&mine)
                            .and_then(|_| Path::append(mine, &path))
                        {
                            Ok(new_path) => {
                                let new_path_id = new_path.id();
                                if !paths_so_far.contains(&new_path_id) {
                                    paths_so_far.insert(new_path_id);
                                    paths.push(new_path);
                                    num_found_paths += 1;
                                }

                                if num_found_paths == num_paths {
                                    break 'bfs;
                                }
                            }
                            Err(_e) => {}
                        }
                    }
                }

                for conveyor_subtype in (0..=7).rev() {
                    let conveyor =
                        Object::conveyor_with_subtype_and_exgress_at(conveyor_subtype, (nx, ny));
                    let ingress = conveyor.ingress().unwrap();
                    match map
                        .can_insert_object(&conveyor)
                        .and_then(|_| Path::append(conveyor, &path))
                    {
                        Ok(new_path) => {
                            let distance = min_distance_to_deposits(&[ingress]);
                            queue.push(Reverse((distance, path_length, Rc::new(new_path))));
                        }
                        Err(_e) => {}
                    }
                }

                for combiner_subtype in 0..=3 {
                    let combiner =
                        Object::combiner_with_subtype_and_exgress_at(combiner_subtype, (nx, ny));
                    let ingresses = combiner.ingresses();
                    match map
                        .can_insert_object(&combiner)
                        .and_then(|_| Path::append(combiner, &path))
                    {
                        Ok(new_path) => {
                            let distance = min_distance_to_deposits(&ingresses);
                            queue.push(Reverse((distance, path_length, Rc::new(new_path))));
                        }
                        Err(_e) => {}
                    }
                }
            }
        }
    }

    paths.shrink_to_fit();
    paths
}

/// Create a map of shortest distances to given deposits from all reachable points on map
fn build_distance_map_from_deposits(map: &Map, deposits: &[Object]) -> HashMap<Point, u32> {
    let mut distances: HashMap<Point, u32> = HashMap::new();
    let mut queue: VecDeque<(u32, Point)> = VecDeque::new();
    let mut visited: HashSet<Point> = HashSet::new();

    for deposit in deposits {
        for exgress in deposit.exgresses() {
            for position in neighbours(exgress.0, exgress.1) {
                if !visited.contains(&position) {
                    visited.insert(position);
                    if map.is_empty_at(position.0, position.1) {
                        queue.push_back((0, position));
                    }
                }
            }
        }
    }

    while let Some((distance, (x, y))) = queue.pop_front() {
        distances
            .entry((x, y))
            .and_modify(|old_distance| *old_distance = (*old_distance).min(distance))
            .or_insert(distance);
        for position in neighbours(x, y) {
            if !visited.contains(&position) {
                visited.insert(position);
                if map.is_empty_at(position.0, position.1) {
                    queue.push_back((distance + 1, position));
                }
            }
        }
    }

    distances
}

*/
