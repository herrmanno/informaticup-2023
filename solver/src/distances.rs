//! Utilities for calculating shortest path between points

use std::{
    collections::VecDeque,
    hash::{Hash, Hasher},
    sync::{Arc, Mutex},
};

use fxhash::FxHashMap as HashMap;
use fxhash::FxHashSet as HashSet;
use lazy_static::lazy_static;
use model::{
    coord::{neighbours, Point},
    map::Map,
    object::Object,
};

/// Map from (hash(map), hash(deposits)) => distance map
type DistanceCache = HashMap<(u64, u64), Arc<HashMap<Point, u32>>>;

lazy_static! {
    static ref DISTANCES_CACHE: Mutex<DistanceCache> = Default::default();
}

/// Create a map of shortest distances to given deposits from all empty points on map
///
/// Returns map as Arc because it may be read from a cache
pub(crate) fn get_distances(map: &Map, deposits: &[Object]) -> Arc<HashMap<Point, u32>> {
    let map_hash = {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        map.hash(&mut hasher);
        hasher.finish()
    };
    let deposits_hash = {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        deposits.hash(&mut hasher);
        hasher.finish()
    };

    let mut cache = DISTANCES_CACHE.lock().unwrap();
    let distances = cache
        .entry((map_hash, deposits_hash))
        .or_insert_with(|| Arc::new(create_distances(map, deposits)));

    Arc::clone(distances)
}

/// Create a map of shortest distances to given deposits from all reachable points on map
fn create_distances(map: &Map, deposits: &[Object]) -> HashMap<Point, u32> {
    let mut distances: HashMap<Point, u32> = HashMap::default();
    let mut queue: VecDeque<(u32, Point)> = VecDeque::new();
    let mut visited: HashSet<Point> = HashSet::default();

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
