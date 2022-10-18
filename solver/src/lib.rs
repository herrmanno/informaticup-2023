use std::collections::{HashMap, HashSet, VecDeque};

use model::{
    coord::{neighbours, Coord},
    map::{Map, MapObject},
    object::{Object, ObjectCell, ObjectType},
    task::Task,
};

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

    let mut map = original_map.clone();
    let possible_factory_locations = find_possible_factory_positions(&map);
    let mut available_factory_locations = possible_factory_locations;

    println!("{}", original_map);

    // start iterating

    let mut factory_indices = Vec::new();

    // place factories

    for product in task.products.iter() {
        let interesting_deposits = &deposits_by_product[&product.subtype];

        let best_location =
            available_factory_locations
                .iter()
                .enumerate()
                .min_by_key(|(_, position)| {
                    // TODO: weight deposit (resource types) by importance for product
                    let distances = interesting_deposits
                        .iter()
                        .map(|deposit| {
                            let (x, y) = position;
                            let (dx, dy) = deposit.coords();
                            // TODO: use path distance instead of manhattan distance (see task 004)
                            (x - dx).abs() + (y - dy).abs()
                        })
                        .collect::<Vec<i32>>();

                    let sum = distances.iter().sum::<i32>();
                    let mean_distance = sum / distances.len() as i32;
                    let deviation = distances
                        .iter()
                        .map(|&i| (i - mean_distance).abs())
                        .sum::<i32>();

                    sum + deviation
                });

        // TODO: check that for each required resource type, a deposit of such type is
        // reachable (simple path finding) from this factory location
        if let Some((location_index, &(x, y))) = best_location {
            available_factory_locations.swap_remove(location_index);
            let factory = Object::Factory {
                x,
                y,
                subtype: product.subtype,
            };
            match map.insert_object(factory) {
                Ok(factory_index) => factory_indices.push(factory_index),
                Err(e) => panic!("{}", e),
            }

            let min_x = if x > 4 { x - 4 } else { 0 };
            let min_y = if y > 4 { y - 4 } else { 0 };
            available_factory_locations.retain(|coord| {
                !((min_x..x + 5).contains(&coord.0) && (min_y..y + 5).contains(&coord.1))
            });
        } else {
            panic!(
                "Cannot place factory for product {}. No position available.",
                product.subtype
            );
        }
    }

    // construct factory -> deposit paths

    for factory_index in factory_indices {
        let factory = map.get_objects()[factory_index].clone();
        let subtype = factory.object.subtype().unwrap();
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

            println!(
                "Finding shortest path from factory at {:?} to deposit of type {}",
                factory.object.coords(),
                resource_index
            );
            if let Some((_path, new_map)) =
                build_shortest_path_from_factory_to_deposit(&factory, resource_index, &map)
            {
                map = new_map;
                println!("{}", map);
            } else {
                println!(
                    "WARN: Could not find path from factory at {:?} to deposit of type {}",
                    factory.object.coords(),
                    resource_index
                )
            }
        }
    }

    println!("{}", map);
    /*
       STEPS
       1. for every product, that has deposits on board (in descending order by possability to produce goods)
           1. try to find a place for its factory, from
              where a path between the factory and the
              deposits exist
       2. for every factory
           for every deposit of factories product
               try to find the shortest valid path from factory [1] to deposit
               [1] from all factory ingresses and open ends of combiners already connected to the factory
           if there exist no path:
               then:
                   if this factory is already connected to another deposit:
                       remove that path (and forbid it) and take another path to that deposit
                   else: try to replace the factory
    */
    original_map
}

/// Finds all locations, at which a 5x5 factory could be legally placed
fn find_possible_factory_positions(map: &Map) -> Vec<Coord> {
    let width = map.width();
    let height = map.height();

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

/// Vector of object indices
type Path = Vec<usize>;

/// Constructs the shortest path from a factory to a deposit of subtype `resource_index`
fn build_shortest_path_from_factory_to_deposit(
    factory: &MapObject,
    resource_index: usize,
    map: &Map,
) -> Option<(Path, Map)> {
    /// (distance, path, current position, map) TODO: optimize: don't clone list for each state
    type SearchState = (u32, Vec<usize>, Coord, Map);
    let mut queue: VecDeque<SearchState> = VecDeque::new();

    // TODO: add all ingresses from already connected components to the start points
    for start_point in factory.ingresses.iter() {
        queue.push_back((0, vec![], *start_point, map.clone()))
    }

    let mut visited = HashSet::new();

    // TODO: sort queue by current distance to possible target
    while let Some((distance, path, (x, y), map)) = queue.pop_front() {
        if visited.contains(&(x, y)) {
            continue;
        }

        visited.insert((x, y));

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
                        Some(ObjectCell::Exgress { index, .. }) => {
                            let obj = &map.get_objects()[*index];
                            obj.object.kind() == ObjectType::Deposit
                                && obj.object.subtype() == Some(resource_index as u8)
                        }
                        _ => false,
                    });

                if mine_reaches_deposit {
                    let mut map = map.clone();
                    match map.insert_object(mine) {
                        Ok(mine_object_index) => {
                            let mut path = path;
                            path.push(mine_object_index);
                            return Some((path, map));
                        }
                        Err(_e) => {}
                    }
                }
            }

            for conveyor_subtype in (0..=7).rev() {
                let conveyor =
                    Object::conveyor_with_subtype_and_exgress_at(conveyor_subtype, (nx, ny));
                let mut map = map.clone();
                let conveyor_ingress = conveyor.ingress().unwrap();
                match map.insert_object(conveyor) {
                    Ok(conveyor_object_index) => {
                        let mut path = path.clone();
                        path.push(conveyor_object_index);
                        queue.push_back((distance + 1, path, conveyor_ingress, map))
                    }
                    Err(_e) => {}
                }
            }

            for combiner_subtype in (0..=3).rev() {
                let combiner =
                    Object::combiner_with_subtype_and_exgress_at(combiner_subtype, (nx, ny));
                let mut map = map.clone();
                let combiner_ingresses = combiner.ingresses();
                match map.insert_object(combiner) {
                    Ok(conveyor_object_index) => {
                        let mut path = path.clone();
                        path.push(conveyor_object_index);
                        for combiner_ingress in combiner_ingresses {
                            queue.push_back((
                                distance + 1,
                                path.clone(),
                                combiner_ingress,
                                map.clone(),
                            ))
                        }
                    }
                    Err(_e) => {}
                }
            }
        }
    }

    None
}

// fn create_shortest_path_to_deposits(map: &Map, deposit_subtypes: impl Iterator<Item = u8>) -> HashMap<u8, HashMap<Coord, u32>> {
//     let mut distances = HashMap::new();
//     for deposit_subtype in deposit_subtypes {
//         distances.insert(deposit_subtype, create_shortest_path_to_deposit(map, deposit_subtype));
//     }

//     distances
// }

// fn create_shortest_path_to_deposit(map: &Map, deposit_subtype: u8) -> HashMap<(u32,u32), u32> {
//     fn shortest_path(coord: Coord, deposit_subtype: u8, distances: &mut HashMap<Coord, u32>, map: &Map) -> u32 {
//         println!("Checking shortest path from {:?} to deposit {}", coord, deposit_subtype);

//         let (x,y) = coord;
//         let neighbours = neighbours(x, y);

//         if neighbours.iter().any(|(x,y)| match map.get_cell(*x, *y) {
//             Some(ObjectCell::Exgress { index }) => {
//                 let obj = &map.get_objects()[*index].object;
//                 obj.kind() == ObjectType::Deposit && Some(deposit_subtype) == obj.subtype()
//             },
//             _ => false
//         }) {
//             distances.insert(coord, 0);
//             0
//         } else {
//             let distance = neighbours.iter()
//                 .map(|(x,y)| {
//                     shortest_path((*x,*y), deposit_subtype, distances, map)
//                 })
//                 .min()
//                 .unwrap_or(u32::MAX);

//             distances.insert(coord, distance);

//             distance
//         }
//     }

//     let free_cells = {
//         let mut v = vec![];
//         for y in 0..map.height() {
//             for x in 0..map.width() {
//                 if map.get_cell(x, y).is_none() {
//                     v.push((x, y));
//                 }
//             }
//         }
//         v
//     };

//     let mut distances = HashMap::new();
//     for coord in free_cells {
//         shortest_path(coord, deposit_subtype, &mut distances, map);
//     }

//     distances
// }
