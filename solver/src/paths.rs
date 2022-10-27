use std::{
    cell::RefCell,
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, HashSet, VecDeque},
    rc::Rc,
    time::{Duration, Instant},
};

use crate::path::{Path, PathID};
use model::{
    coord::{neighbours, Point},
    map::Map,
    object::{Object, ObjectCell, ObjectType},
};
use rand::Rng;

/// Max time to search for the next path
const MAX_SEARCH_TIME_IN_MILLIS: u64 = 2000;

//TODO: investigate if 100 (current value) is large enough for succesful pathfinding on big maps.
/// Max partial paths to look at without improvement (of distance to target) before search cancellation
const MAX_STEPS_WITHOUT_IMPROVEMENT: usize = 100;

pub(crate) struct Paths<T> {
    distances_to_deposits: HashMap<Point, u32>,
    paths_so_far: HashSet<PathID>,
    queue: BinaryHeap<Reverse<(u32, u32, Rc<Path>)>>,
    resource_index: u8,
    map: Map, //TODO: borrow, instaed of own
    rng: Rc<RefCell<T>>,
}

impl<T: Rng> Paths<T> {
    pub fn new(
        start_points: &[Point],
        resource_index: u8,
        deposits: &[Object],
        map: &Map,
        rng: Rc<RefCell<T>>,
    ) -> Self {
        let distances_to_deposits = build_distance_map_from_deposits(map, deposits);

        let min_distance_to_deposits = |points: &[Point]| {
            points
                .iter()
                .filter_map(|point| distances_to_deposits.get(point))
                .min()
                .cloned()
                .unwrap_or(u32::MAX)
        };

        let paths_so_far: HashSet<PathID> = HashSet::new();
        let mut queue: BinaryHeap<Reverse<(u32, u32, Rc<Path>)>> = BinaryHeap::new();

        for &ingress in start_points {
            let path = Path::from_starting_points(vec![ingress]);
            let distance = min_distance_to_deposits(&neighbours(ingress.0, ingress.1));
            queue.push(Reverse((distance, 0, Rc::new(path))))
        }

        Paths {
            distances_to_deposits,
            paths_so_far,
            queue,
            resource_index,
            map: map.clone(),
            rng,
        }
    }

    #[allow(dead_code)] //TODO: remove
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    #[allow(dead_code)] //TODO: remove
    pub fn clear(&mut self) {
        self.queue.clear();
    }
}

/*
    TODO: check if using a kind of 'Map(Reference)' as state is faster than using paths.

    Background:
    On every step the whole path must be inserted into a map to check it is valid. When using
    a kind of Map, with a partial path already inserted, one must only check the path's new
    segment for validity.

*/

impl<T: Rng> Iterator for Paths<T> {
    type Item = Path;

    /* TODO: Abort search if no great improvement can be found over some time

        Example:
        00000000001111111111222222222
        01234567890123456789012345678
        00 --------XXXXXXXXXXXXX--------
        01 -000000-XXXX+++++XXXX-222222-
        02 -000000-XXXX+111+XXXX-222222-
        03 -000000-XXXX+111+XXXX-222222-
        04 -000000-XXXX+111+XXXX-222222-
        05 -000000-XXXX+++++XXXX-222222-
        06 -000000-XXXX.....XXXX-222222-
        07 -000000-XXXX.....XXXX-222222-
        08 --------XXXX.....XXXX--------
        09 ........XXXX.....XXXX........
        10 ........XXXX..X..XXXX........
        11 ..............X..............
        12 ........XXXX..X..XXXX........
        13 ........XXXX+++++XXXX........
        14 --------XXXX+000+XXXX--------
        15 -111111-XXXX+000+XXXX-333333-
        16 -111111-XXXX+000+XXXX-333333-
        17 -111111-XXXX+++++XXXX-333333-
        18 -111111-XXXX.....XXXX-333333-
        19 -111111-XXXX.....XXXX-333333-
        20 -111111-XXXX.....XXXX-333333-
        21 -111111-XXXX.....XXXX-333333-
        22 --------XXXXXXXXXXXXX--------

        When trying fo find a path from factory 0 to deposit 0 or 1, there is no valid path.
        Nevertheless, many path in the direction of deposit 2/3 will be tried.

    */
    fn next(&mut self) -> Option<Self::Item> {
        let Paths {
            distances_to_deposits,
            paths_so_far,
            queue,
            resource_index,
            map,
            ref rng,
        } = self;

        let min_distance_to_deposits = |points: &[Point]| {
            points
                .iter()
                .filter_map(|point| distances_to_deposits.get(point))
                .min()
                .cloned()
                .unwrap_or(u32::MAX)
                .saturating_add(rng.borrow_mut().gen_range(0..=10)) // TODO: use randomness in a smarter way
        };

        let timer = Instant::now();

        let mut i: usize = 0;
        let mut min_distance: Option<(u32, usize)> = None;
        while let Some(Reverse((path_distance, path_length, path))) = queue.pop() {
            i += 1;

            if timer.elapsed() > Duration::from_millis(MAX_SEARCH_TIME_IN_MILLIS) {
                return None;
            }

            min_distance = match min_distance {
                None => Some((path_distance, i)),
                Some((dist, _)) if path_distance < dist => Some((path_distance, i)),
                Some((_, j)) if i - j < MAX_STEPS_WITHOUT_IMPROVEMENT => min_distance,
                _ => {
                    return None;
                }
            };

            // TODO: smarter way to kick paths, that can not reach target
            if path_distance > 200 {
                continue;
            }

            for (x, y) in path.heads() {
                /*  LOGIC
                    1. try if target is reached if a mine is placed
                    2. try using long conveyor
                    3. try using short conveyor
                    4. try using combiner
                */

                let free_neighbours = neighbours(x, y)
                    .into_iter()
                    .filter(|(x, y)| map.is_empty_at(*x, *y))
                    .collect::<Vec<Point>>();

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
                                        && obj.subtype() == Some(*resource_index)
                                }
                                _ => false,
                            });

                        if mine_reaches_deposit {
                            let new_path = Path::append_unchecked(mine, &path);
                            match map.can_insert_objects(new_path.objects().collect()) {
                                Ok(_) => {
                                    let new_path_id = new_path.id();
                                    if !paths_so_far.contains(&new_path_id) {
                                        paths_so_far.insert(new_path_id);

                                        // Try to reuse path ?!
                                        queue.push(Reverse((path_distance, path_length, path)));

                                        return Some(new_path);
                                    }
                                }
                                Err(_e) => {}
                            }
                        }
                    }

                    for conveyor_subtype in (0..=7).rev() {
                        let conveyor = Object::conveyor_with_subtype_and_exgress_at(
                            conveyor_subtype,
                            (nx, ny),
                        );
                        let ingress = conveyor.ingress().unwrap();
                        let new_path = Path::append_unchecked(conveyor, &path);
                        match map.can_insert_objects(new_path.objects().collect()) {
                            Ok(_) => {
                                let distance = min_distance_to_deposits(&[ingress]);
                                queue.push(Reverse((distance, path_length, Rc::new(new_path))));
                            }
                            Err(_e) => {}
                        }
                    }

                    for combiner_subtype in 0..=3 {
                        let combiner = Object::combiner_with_subtype_and_exgress_at(
                            combiner_subtype,
                            (nx, ny),
                        );
                        let ingresses = combiner.ingresses();
                        let new_path = Path::append_unchecked(combiner, &path);
                        match map.can_insert_objects(new_path.objects().collect()) {
                            Ok(_) => {
                                let distance = min_distance_to_deposits(&ingresses);
                                queue.push(Reverse((distance, path_length, Rc::new(new_path))));
                            }
                            Err(_e) => {}
                        }
                    }
                }
            }
        }

        None
    }
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
