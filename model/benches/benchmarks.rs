use criterion::{criterion_group, criterion_main, Criterion};
use model::{
    map::{new_map, Maplike},
    object::Object,
};

fn map_can_insert_object(c: &mut Criterion) {
    let map = new_map(100, 100, vec![]);

    let objects = vec![
        Object::Mine {
            x: 10,
            y: 10,
            subtype: 0,
        },
        Object::Conveyor {
            x: 50,
            y: 50,
            subtype: 0,
        },
        Object::Combiner {
            x: 75,
            y: 75,
            subtype: 0,
        },
        Object::Factory {
            x: 60,
            y: 20,
            subtype: 0,
        },
    ];
    let objects = objects.iter().cycle().take(101).collect::<Vec<&Object>>();

    c.bench_function("map.can_insert_object(..)", |b| {
        b.iter(|| {
            for object in objects.iter() {
                map.can_insert_object(object).unwrap();
            }
        })
    });
}

criterion_group!(map_benches, map_can_insert_object);
criterion_main!(map_benches);
