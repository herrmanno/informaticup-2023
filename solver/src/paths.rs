use std::{
    cell::RefCell,
    collections::BinaryHeap,
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};

use fxhash::FxHashMap as HashMap;
use fxhash::FxHashSet as HashSet;

use crate::distances::get_distances;
use crate::path::{Path, PathID};
use model::{
    coord::{neighbours, Point},
    map::Map,
    object::Object,
};
use rand::Rng;

/// Max time to search for the next path
const MAX_SEARCH_TIME_IN_MILLIS: u64 = 500;

/// Max partial paths to look at without improvement (of distance to target) before search cancellation
///
/// Is used to abort paths that probably overshoot and no longer can reach the target.
///
/// This approach evidently *may* lead to paths being aborted that may reach the target (soon), but
/// will increase overall performance by pruning bad paths early.
const MAX_STEPS_WITHOUT_IMPROVEMENT: usize = 10;

/// Max manhattan distance a path's head may have from the target
///
/// Used to discard paths that have gone wild and probably don't have a change to reach the target anymoe
///
/// This approach evidently *may* lead to paths being aborted that may reach the target (soon), but
/// will increase overall performance by pruning bad paths early.
// const MAX_PATH_DISTANCE: u32 = 100;

/// Max length (as number of objects) any path may reach
///
/// Used to discard paths that have gone wild and probably don't have a change to reach the target anymoe
///
/// This approach evidently *may* lead to paths being aborted that may reach the target (soon), but
/// will increase overall performance by pruning bad paths early.
// const MAX_PATH_LENGTH: u32 = 100;

struct PathSearchState {
    start_distance: u32,
    distance: u32,
    path_length: u32,
    path: Rc<Path>,
    map_ref: Arc<Map>,
}

impl PartialEq for PathSearchState {
    fn eq(&self, other: &Self) -> bool {
        self.path.eq(&other.path)
    }
}

impl Eq for PathSearchState {}

impl PartialOrd for PathSearchState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathSearchState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .distance
            .cmp(&self.distance)
            .then(other.path_length.cmp(&self.path_length))
    }
}

pub struct Paths<T> {
    distances_to_deposits: Arc<HashMap<Point, u32>>,
    paths_so_far: HashSet<PathID>,
    queue: BinaryHeap<PathSearchState>,
    rng: Rc<RefCell<T>>,
}

impl<T: Rng> Paths<T> {
    pub fn new(
        start_points: &[Point],
        deposits: &[Object],
        map: &Map,
        rng: Rc<RefCell<T>>,
    ) -> Self {
        let distances_to_deposits = get_distances(map, deposits);

        let min_distance_to_deposits = |points: &[Point]| {
            points
                .iter()
                .filter_map(|point| distances_to_deposits.get(point))
                .min()
                .cloned()
        };

        let paths_so_far: HashSet<PathID> = HashSet::default();

        let mut queue: BinaryHeap<PathSearchState> = BinaryHeap::new();

        let map_ref = Arc::new(map.clone());
        for &ingress in start_points {
            let path = Path::from_starting_points(vec![ingress]);
            let distance = min_distance_to_deposits(&neighbours(ingress.0, ingress.1));
            if let Some(distance) = distance {
                queue.push(PathSearchState {
                    start_distance: distance,
                    distance,
                    path_length: 0,
                    path: Rc::new(path),
                    map_ref: Arc::clone(&map_ref),
                });
            }
        }

        Paths {
            distances_to_deposits,
            paths_so_far,
            queue,
            rng,
        }
    }
}

impl<T: Rng> Iterator for Paths<T> {
    type Item = Path;

    fn next(&mut self) -> Option<Self::Item> {
        let Paths {
            distances_to_deposits,
            paths_so_far,
            queue,
            ref rng,
            ..
        } = self;

        let min_distance_to_deposits = |points: &[Point]| {
            points
                .iter()
                .filter_map(|point| distances_to_deposits.get(point))
                .min()
                .cloned()
                .map(|d| d.saturating_add(rng.borrow_mut().gen_range(0..=10))) // TODO: use randomness in a smarter way
        };

        let timer = Instant::now();

        let mut i: usize = 0;
        let mut min_distance: Option<(u32, usize)> = None;
        while let Some(PathSearchState {
            start_distance,
            distance: path_distance,
            path_length,
            path,
            map_ref,
        }) = queue.pop()
        {
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

            // TODO: investigate if dynamic path_{distance,length} bounds help early pruning
            let MAX_PATH_DISTANCE = 2 * start_distance;
            let MAX_PATH_LENGTH = ((start_distance / 3) + 100).max(500);

            // // TODO: smarter way to kick paths, that can not reach target
            if path_distance > MAX_PATH_DISTANCE || path_length > MAX_PATH_LENGTH {
                continue;
            }

            for (x, y) in path.heads() {
                /*  LOGIC
                    1. check if target is reached by placing a mine
                    2. try using long conveyor
                    3. try using short conveyor
                    4. try using combiner
                */

                let free_neighbours = neighbours(x, y)
                    .into_iter()
                    .filter(|(x, y)| map_ref.is_empty_at(*x, *y))
                    .collect::<Vec<Point>>();

                for (nx, ny) in free_neighbours {
                    for mine_subtype in 0..=3 {
                        let mine = Object::mine_with_subtype_and_exgress_at(mine_subtype, (nx, ny));
                        let mine_ingress = mine.ingress().unwrap();

                        let mine_reaches_deposit = distances_to_deposits
                            .get(&mine_ingress)
                            .cloned()
                            .unwrap_or(u32::MAX)
                            == 0;

                        if mine_reaches_deposit && map_ref.can_insert_object(&mine).is_ok() {
                            let new_path = Path::append(mine, &path);
                            let new_path_id = new_path.id();
                            if paths_so_far.insert(new_path_id) {
                                return Some(new_path);
                            }
                        }
                    }

                    for conveyor_subtype in (0..=7).rev() {
                        let conveyor = Object::conveyor_with_subtype_and_exgress_at(
                            conveyor_subtype,
                            (nx, ny),
                        );
                        let ingress = conveyor.ingress().unwrap();

                        if map_ref.can_insert_object(&conveyor).is_ok() {
                            let new_path = Path::append(conveyor.clone(), &path);
                            if let Some(distance) = min_distance_to_deposits(&[ingress]) {
                                let mut new_map_ref = Map::from_map(&map_ref);
                                new_map_ref.insert_object_unchecked(conveyor);

                                queue.push(PathSearchState {
                                    start_distance,
                                    distance,
                                    path_length,
                                    path: Rc::new(new_path),
                                    map_ref: Arc::new(new_map_ref),
                                })
                            }
                        }
                    }

                    for combiner_subtype in 0..=3 {
                        let combiner = Object::combiner_with_subtype_and_exgress_at(
                            combiner_subtype,
                            (nx, ny),
                        );
                        let ingresses = combiner.ingresses();

                        if map_ref.can_insert_object(&combiner).is_ok() {
                            let new_path = Path::append(combiner.clone(), &path);
                            if let Some(distance) = min_distance_to_deposits(&ingresses) {
                                let mut new_map_ref = Map::from_map(&map_ref);
                                new_map_ref.insert_object_unchecked(combiner);

                                queue.push(PathSearchState {
                                    start_distance,
                                    distance,
                                    path_length,
                                    path: Rc::new(new_path),
                                    map_ref: Arc::new(new_map_ref),
                                });
                            }
                        }
                    }
                }
            }
        }

        None
    }
}
