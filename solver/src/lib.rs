mod path;

use std::{
    collections::{HashMap, HashSet, VecDeque},
    rc::Rc,
};

use model::{
    coord::{neighbours, Coord},
    map::{Map, MapObject},
    object::{Object, ObjectCell, ObjectType},
    task::Task,
};

use path::Path;

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
            if let Some(path) =
                build_shortest_path_from_factory_to_deposit(&factory, resource_index, &map)
            {
                // TODO: store additional ingresses of path for next path finding from same factory
                for object in Into::<Vec<Object>>::into(path) {
                    // TODO: may insert unchecked, because we already know all parts are legal
                    if let Err(e) = map.insert_object(object) {
                        unreachable!("Error while inserting path object onto map: '{}'", e)
                    }
                }
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

/// Constructs the shortest path from a factory to a deposit of subtype `resource_index`
fn build_shortest_path_from_factory_to_deposit(
    factory: &MapObject,
    resource_index: usize,
    map: &Map,
) -> Option<Path> {
    let mut queue: VecDeque<Rc<Path>> = VecDeque::new();

    let path = Path::from_starting_points(factory.ingresses.clone());
    queue.push_front(Rc::new(path));

    let mut visited = HashSet::new();

    // TODO: sort queue by current distance to possible target
    while let Some(path) = queue.pop_front() {
        for (x, y) in path.heads() {
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
                        match map
                            .can_insert_object(&mine)
                            .and_then(|_| Path::append(mine, &path))
                        {
                            Ok(path) => {
                                // return Some((&path).into());
                                return Some(path);
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

    None
}
