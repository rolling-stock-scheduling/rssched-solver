use criterion::{criterion_group, criterion_main, Criterion};
use model::base_types::VehicleIdx;
use solution::{
    path::Path,
    test_utilities::{default_schedule, init_test_data},
};

pub fn schedule_modification_benchmarks(c: &mut Criterion) {
    let d = init_test_data();
    let schedule = default_schedule(&d);
    c.bench_function("spawn_vehicle_for_path", |b| {
        b.iter(|| {
            schedule
                .spawn_vehicle_for_path(
                    d.vt1,
                    vec![d.start_depot3, d.trip12, d.trip23, d.trip31, d.end_depot2],
                )
                .unwrap();
        })
    });

    let veh0 = VehicleIdx::vehicle_from(0);
    c.bench_function("replace_vehicle_by_dummy", |b| {
        b.iter(|| {
            schedule.replace_vehicle_by_dummy(veh0).unwrap();
        })
    });

    c.bench_function("add_path_to_vehicle_tour", |b| {
        b.iter(|| {
            schedule
                .add_path_to_vehicle_tour(
                    veh0,
                    Path::new(vec![d.trip12, d.trip23, d.trip31], d.network.clone())
                        .unwrap()
                        .unwrap(),
                )
                .unwrap();
        })
    });

    // TODO implement benchmarks for all modifications, using a small instance and a large instance
}

criterion_group!(benches, schedule_modification_benchmarks);
criterion_main!(benches);
