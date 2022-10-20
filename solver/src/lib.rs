mod path;

use std::{
    collections::{HashMap, HashSet, VecDeque},
    rc::Rc,
};

use model::{
    coord::{neighbours, Point},
    map::Map,
    object::{Coord, Object, ObjectCell, ObjectType, Subtype},
    task::{Product, Task},
};

use path::{Path, PathID};
use rand::{
    distributions::WeightedIndex, prelude::Distribution, seq::SliceRandom, thread_rng, Rng,
};
use simulator::{simulate, SimulatorResult};

/// Number of whole iterations
const NUM_ITERATIONS: u32 = 500;

/// Number of times a factory location is tried.
/// If no location can be found a whole new iteration starts
const NUM_MAX_FACTORY_PLACEMENTS: u32 = 100;

/// Number of pre-calculated paths per factory (position) and resource type
const NUM_PATHS_PER_FACTORY_AND_RESOURCE: u32 = 100;

/// Number of path combinations to try during one iteration
const NUM_PATH_COMBINING_ITERATIONS: u32 = 1000;

pub fn solve<'a, 'b>(task: &'a Task, original_map: &'b mut Map) -> &'b Map {
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
    // let mut available_factory_locations = possible_factory_locations;

    let mut best_factory_positions_by_factory_subtype: HashMap<
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

    println!("{}", original_map);

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

    'iterate: for _ in 0..NUM_ITERATIONS {
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

            continue 'iterate;
        }

        // construct factory -> deposit paths

        let mut paths_from_factory_to_resource: HashMap<(Subtype, Subtype), Vec<Path>> =
            HashMap::new();

        // construct shortest paths from factories to deposits

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

            for (resource_index, &amount) in product.resources.iter().enumerate() {
                if amount == 0 {
                    continue;
                }

                let shortest_paths = build_shortest_paths_from_factory_to_deposit(
                    NUM_PATHS_PER_FACTORY_AND_RESOURCE,
                    factory,
                    resource_index,
                    &map,
                    &mut rng,
                );

                if !shortest_paths.is_empty() {
                    paths_from_factory_to_resource
                        .insert((subtype, resource_index as Subtype), shortest_paths);
                } else {
                    // No path from factory to resource could be found -> forbid factory position

                    let current_factory_position = factory.coords();
                    let factory_positions = best_factory_positions_by_factory_subtype
                        .get_mut(&subtype)
                        .unwrap();
                    let current_factory_position_index = factory_positions
                        .1
                        .iter()
                        .enumerate()
                        .find_map(|(index, position)| {
                            if *position == current_factory_position {
                                Some(index)
                            } else {
                                None
                            }
                        })
                        .unwrap();

                    factory_positions
                        .0
                        .update_weights(&vec![(current_factory_position_index, &0f32)][..])
                        .unwrap();
                    continue 'iterate;
                }
            }
        }

        // chose path combinations

        'combining_paths: for _ in 0..NUM_PATH_COMBINING_ITERATIONS {
            // TODO: shuffle factory_indices
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

                // TODO: sort by: distance to deposit (start with furthest) OR importance of resource type
                for (resource_index, &amount) in product.resources.iter().enumerate() {
                    if amount == 0 {
                        continue;
                    }

                    let shortest_paths =
                        &paths_from_factory_to_resource[&(subtype, resource_index as Subtype)];
                    // TODO: pick better paths with higher probability
                    let path = &shortest_paths[rng.gen_range(0..shortest_paths.len())];

                    if map.try_insert_objects(path.into()).is_err() {
                        continue 'combining_paths;
                    }
                }
            }

            break 'combining_paths;
        }

        // FIXME: build additional path in descending product priority

        let map_score = simulate(task, &map, true);

        best_solution = if let Some((result, best_map)) = best_solution {
            if map_score > result {
                println!("{:?}", map_score);
                println!("{}", map);
                Some((map_score, map))
            } else {
                Some((result, best_map))
            }
        } else {
            Some((map_score, map))
        };
    }

    original_map
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

/// Constructs the shortest path from a factory to a deposit of subtype `resource_index`
fn build_shortest_paths_from_factory_to_deposit<R: Rng + ?Sized>(
    num_paths: u32,
    factory: &Object,
    resource_index: usize,
    map: &Map,
    rng: &mut R,
) -> Vec<Path> {
    let mut i = 0;
    let mut paths = Vec::with_capacity(num_paths as usize);
    let mut paths_so_far: HashSet<PathID> = HashSet::new();
    let mut queue: VecDeque<Rc<Path>> = VecDeque::new();

    let mut ingresses = factory.ingresses();
    ingresses.shuffle(rng);
    let path = Path::from_starting_points(ingresses);
    queue.push_front(Rc::new(path));

    // TODO: sort queue by current distance to possible target
    'bfs: while let Some(path) = queue.pop_front() {
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
                for mine_subtype in 2..=3 {
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
                                    i += 1;
                                }

                                if i == num_paths {
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
                    match map
                        .can_insert_object(&conveyor)
                        .and_then(|_| Path::append(conveyor, &path))
                    {
                        Ok(path) => queue.push_back(Rc::new(path)),
                        Err(_e) => {}
                    }
                }

                for combiner_subtype in (0..=3).rev() {
                    let combiner =
                        Object::combiner_with_subtype_and_exgress_at(combiner_subtype, (nx, ny));
                    match map
                        .can_insert_object(&combiner)
                        .and_then(|_| Path::append(combiner, &path))
                    {
                        Ok(path) => queue.push_back(Rc::new(path)),
                        Err(_e) => {}
                    }
                }
            }
        }
    }

    paths.shrink_to_fit();
    paths
}
