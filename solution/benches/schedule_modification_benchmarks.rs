// © 2023-2024 ETH Zurich
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// any later version.
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

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
