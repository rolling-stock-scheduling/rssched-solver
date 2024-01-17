use itertools::assert_equal;
use model::base_types::{Distance, VehicleId, VehicleTypeId};

use crate::{
    path::Path,
    segment::Segment,
    test_utilities::{init_test_data, TestData},
    Schedule,
};

fn default_schedule(d: &TestData) -> Schedule {
    let mut schedule =
        Schedule::empty(d.vehicle_types.clone(), d.network.clone(), d.config.clone());

    // veh00000
    schedule = schedule
        .spawn_vehicle_for_path(
            d.vt1,
            vec![
                d.start_depot1,
                d.trip12,
                d.trip23,
                d.trip34,
                d.trip45,
                d.trip51,
                d.end_depot2,
            ],
        )
        .unwrap()
        .0;

    // veh00001
    schedule = schedule
        .spawn_vehicle_for_path(
            d.vt2,
            vec![d.start_depot2, d.trip31, d.trip14, d.end_depot1],
        )
        .unwrap()
        .0;

    // veh00002
    schedule = schedule
        .spawn_vehicle_for_path(
            d.vt2,
            vec![d.start_depot1, d.trip12, d.trip23, d.trip31, d.end_depot2],
        )
        .unwrap()
        .0;

    schedule
}

#[test]
fn basic_methods_test() {
    // ARRANGE
    let d = init_test_data();
    let veh0 = VehicleId::from("veh00000");
    let veh1 = VehicleId::from("veh00001");
    let veh2 = VehicleId::from("veh00002");
    let veh3 = VehicleId::from("veh00003");
    let veh4 = VehicleId::from("veh00004");

    // ACT
    let schedule = default_schedule(&d);

    // ASSERT
    assert_eq!(schedule.number_of_vehicles(), 3);
    assert_equal(schedule.vehicles_iter(), [veh0, veh1, veh2].iter().cloned());
    assert!(schedule.is_vehicle(veh0));
    assert!(schedule.is_vehicle(veh1));
    assert!(schedule.is_vehicle(veh2));
    assert!(!schedule.is_vehicle(veh3));

    assert_eq!(schedule.get_vehicle(veh0).unwrap().id(), veh0);
    assert_eq!(schedule.get_vehicle(veh0).unwrap().type_id(), d.vt1);

    assert_eq!(schedule.get_vehicle(veh1).unwrap().id(), veh1);
    assert_eq!(schedule.get_vehicle(veh1).unwrap().type_id(), d.vt2);

    assert_eq!(schedule.get_vehicle(veh2).unwrap().id(), veh2);
    assert_eq!(schedule.get_vehicle(veh2).unwrap().type_id(), d.vt2);

    assert!(schedule.get_vehicle(veh4).is_err());

    assert_eq!(schedule.vehicle_type_of(veh0), d.vt1);
    assert_eq!(schedule.vehicle_type_of(veh1), d.vt2);
    assert_eq!(schedule.vehicle_type_of(veh2), d.vt2);

    assert!(!schedule.is_dummy(veh0));
    assert!(!schedule.is_dummy(veh1));
    assert!(!schedule.is_dummy(veh2));
    assert!(!schedule.is_dummy(veh3));

    assert_eq!(schedule.number_of_dummy_tours(), 0);
    assert!(schedule.dummy_iter().next().is_none());

    assert_eq!(schedule.tour_of(veh0).unwrap().len(), 7);
    assert_eq!(schedule.tour_of(veh1).unwrap().len(), 4);
    assert_eq!(schedule.tour_of(veh2).unwrap().len(), 5);

    assert_eq!(
        schedule.train_formation_of(d.trip12).ids(),
        vec![veh0, veh2]
    );
    assert_eq!(
        schedule.train_formation_of(d.trip23).ids(),
        vec![veh0, veh2]
    );
    assert_eq!(schedule.train_formation_of(d.trip34).ids(), vec![veh0]);
    assert_eq!(schedule.train_formation_of(d.trip45).ids(), vec![veh0]);
    assert_eq!(schedule.train_formation_of(d.trip51).ids(), vec![veh0]);
    assert_eq!(
        schedule.train_formation_of(d.trip31).ids(),
        vec![veh1, veh2]
    );
    assert_eq!(schedule.train_formation_of(d.trip14).ids(), vec![veh1]);

    assert_eq!(
        schedule.number_of_vehicles_of_same_type_spawned_at(d.depot1, d.vt1),
        1
    );
    assert_eq!(
        schedule.number_of_vehicles_of_same_type_spawned_at(d.depot1, d.vt2),
        1
    );
    assert_eq!(
        schedule.number_of_vehicles_of_same_type_spawned_at(d.depot2, d.vt1),
        0
    );
    assert_eq!(
        schedule.number_of_vehicles_of_same_type_spawned_at(d.depot2, d.vt2),
        1
    );

    assert_eq!(schedule.depot_balance(d.depot1, d.vt1), 1);
    assert_eq!(schedule.depot_balance(d.depot1, d.vt2), 0);
    assert_eq!(schedule.depot_balance(d.depot2, d.vt1), -1);
    assert_eq!(schedule.depot_balance(d.depot2, d.vt2), 0);
    assert_eq!(schedule.depot_balance(d.depot3, d.vt1), 0);

    assert_eq!(schedule.total_depot_balance_violation(), 2);

    assert!(!schedule.can_depot_spawn_vehicle(d.start_depot1, d.vt1));
    assert!(!schedule.can_depot_spawn_vehicle(d.start_depot1, d.vt2));
    assert!(!schedule.can_depot_spawn_vehicle(d.start_depot2, d.vt1));
    assert!(!schedule.can_depot_spawn_vehicle(d.start_depot2, d.vt2));
    assert!(!schedule.can_depot_spawn_vehicle(d.start_depot4, d.vt1));
    assert!(schedule.can_depot_spawn_vehicle(d.start_depot4, d.vt2));
    assert!(!schedule.can_depot_spawn_vehicle(d.start_depot2, VehicleTypeId::from("vt3")));

    assert!(!schedule.reduces_spawning_at_depot_violation(d.vt1, d.depot1));
    assert!(!schedule.reduces_spawning_at_depot_violation(d.vt2, d.depot1));
    assert!(schedule.reduces_spawning_at_depot_violation(d.vt1, d.depot2));
    assert!(!schedule.reduces_spawning_at_depot_violation(d.vt2, d.depot2));

    assert!(schedule.reduces_despawning_at_depot_violation(d.vt1, d.depot1));
    assert!(!schedule.reduces_despawning_at_depot_violation(d.vt2, d.depot1));
    assert!(!schedule.reduces_despawning_at_depot_violation(d.vt1, d.depot2));
    assert!(!schedule.reduces_despawning_at_depot_violation(d.vt2, d.depot2));

    assert_eq!(schedule.number_of_unserved_passengers(), 130);

    assert!(schedule.is_fully_covered(d.trip12));
    assert!(schedule.is_fully_covered(d.trip23));
    assert!(!schedule.is_fully_covered(d.trip34));
    assert!(!schedule.is_fully_covered(d.trip45));
    assert!(!schedule.is_fully_covered(d.trip51));
    assert!(schedule.is_fully_covered(d.trip31));
    assert!(!schedule.is_fully_covered(d.trip14));

    assert_eq!(schedule.seat_distance_traveled(), 1630000 + 3640000); //service trips + dead_head_trips
    assert_eq!(
        schedule.total_dead_head_distance(),
        Distance::from_km(12.0 + 23.0 + 41.0 + 12.0)
    );

    schedule.verify_consistency();
}

#[test]
fn scheduling_ordering_test() {
    // ARRANGE
    let d = init_test_data();
    let veh1 = VehicleId::from("veh00001");
    let veh2 = VehicleId::from("veh00002");
    let schedule_default = default_schedule(&d);
    let schedule_four_vehicles = schedule_default
        .spawn_vehicle_for_path(d.vt2, vec![d.trip12, d.trip23, d.trip31])
        .unwrap()
        .0;
    let schedule_two_vehicles = schedule_default.replace_vehicle_by_dummy(veh2).unwrap();
    let schedule_long_tour = schedule_default
        .add_path_to_vehicle_tour(
            veh1,
            Path::new(vec![d.trip51], d.network.clone())
                .unwrap()
                .unwrap(),
        )
        .unwrap();
    let schedule_short_tour = schedule_two_vehicles
        .spawn_vehicle_for_path(d.vt2, vec![d.trip31])
        .unwrap()
        .0;
    let schedule_copy = schedule_default.clone();

    // ASSERT
    assert!(schedule_two_vehicles < schedule_short_tour);
    assert!(schedule_short_tour < schedule_default);
    assert!(schedule_default == schedule_copy);
    assert!(schedule_copy < schedule_long_tour);
    assert!(schedule_long_tour < schedule_four_vehicles);
}

#[test]
fn spawn_vehicle_to_repalce_dummy_tour_test() {
    // ARRANGE
    let d = init_test_data();
    let veh0 = VehicleId::from("veh00000");
    let veh4 = VehicleId::from("veh00004");
    let schedule = default_schedule(&d).replace_vehicle_by_dummy(veh0).unwrap();
    let dummy3 = VehicleId::from("dummy00003");

    // ACT
    let (new_schedule, new_vehicle) = schedule
        .spawn_vehicle_to_replace_dummy_tour(dummy3, d.vt1)
        .unwrap();

    // ASSERT
    assert_eq!(new_schedule.number_of_vehicles(), 3);
    assert_eq!(new_schedule.number_of_dummy_tours(), 0);

    assert_eq!(new_vehicle, veh4);

    assert_equal(
        new_schedule.tour_of(veh4).unwrap().all_nodes_iter(),
        [
            d.start_depot1,
            d.trip12,
            d.trip23,
            d.trip34,
            d.trip45,
            d.trip51,
            d.end_depot1,
        ]
        .iter()
        .cloned(),
    );
    new_schedule.verify_consistency();
}

#[test]
fn spawn_vehicle_to_repalce_dummy_tour_failure_test() {
    // ARRANGE
    let d = init_test_data();
    let veh0 = VehicleId::from("veh00000");
    // create dummy by using override_reassign:
    let schedule = default_schedule(&d);

    // ACT
    let result = schedule.spawn_vehicle_to_replace_dummy_tour(veh0, d.vt1);

    // ASSERT
    assert!(result.is_err());
}

#[test]
fn spawn_vehicle_for_path_without_depots_test() {
    // ARRANGE
    let d = init_test_data();
    let schedule = default_schedule(&d);
    let veh3 = VehicleId::from("veh00003");

    // ACT
    let (new_schedule, new_vehicle) = schedule
        .spawn_vehicle_for_path(
            d.vt1,
            vec![d.trip12, d.trip23, d.trip34, d.trip45, d.trip51],
        )
        .unwrap();

    // vehicle spawns from depot3 as depot1 and 2 are full.
    // de-spawns at depot1 as it is the closest.

    // ASSERT
    assert_eq!(new_schedule.number_of_vehicles(), 4);
    assert_eq!(new_vehicle, veh3);
    assert_equal(
        new_schedule.tour_of(veh3).unwrap().all_nodes_iter(),
        [
            d.start_depot3,
            d.trip12,
            d.trip23,
            d.trip34,
            d.trip45,
            d.trip51,
            d.end_depot1,
        ]
        .iter()
        .cloned(),
    );
    new_schedule.verify_consistency();
}

#[test]
fn spawning_too_many_vehicles_gives_err_test() {
    // ARRANGE
    let d = init_test_data();
    let schedule = default_schedule(&d);
    // depot1 has capacity 1 for vt1 which is occupied by veh0.
    // depot2 has capacity 0
    // depot3 has capacity 1
    // depot4 has capacity 0
    // depot5 has capacity 1
    // Hence, we can spawn 2 vehicles of vt1 before an error occurs.

    // ACT
    let result = schedule
        .spawn_vehicle_for_path(d.vt1, vec![d.trip12])
        .unwrap()
        .0
        .spawn_vehicle_for_path(d.vt1, vec![d.trip12])
        .unwrap()
        .0
        .spawn_vehicle_for_path(d.vt1, vec![d.trip12]);

    // ASSERT
    assert!(result.is_err());
}

#[test]
fn replace_vehicle_by_dummy_success_test() {
    // ARRANGE
    let d = init_test_data();
    let schedule = default_schedule(&d);
    let veh0 = VehicleId::from("veh00000");
    let veh2 = VehicleId::from("veh00002");
    let dummy3 = VehicleId::from("dummy00003");

    // ACT
    let new_schedule = schedule.replace_vehicle_by_dummy(veh0).unwrap();

    // ASSERT
    assert_eq!(new_schedule.number_of_vehicles(), 2);

    assert!(new_schedule.get_vehicle(veh0).is_err());
    assert!(new_schedule.tour_of(veh0).is_err());
    assert_equal(
        new_schedule.train_formation_of(d.trip12).ids(),
        [veh2].iter().cloned(),
    );
    assert_equal(
        new_schedule.train_formation_of(d.trip23).ids(),
        [veh2].iter().cloned(),
    );
    assert!(new_schedule.train_formation_of(d.trip34).ids().is_empty(),);
    assert!(new_schedule.train_formation_of(d.trip45).ids().is_empty(),);
    assert!(new_schedule.train_formation_of(d.trip51).ids().is_empty(),);

    assert!(!new_schedule.is_fully_covered(d.trip12));
    assert!(!new_schedule.is_fully_covered(d.trip23));

    assert_eq!(new_schedule.depot_balance(d.depot1, d.vt1), 0);
    assert_eq!(new_schedule.depot_balance(d.depot2, d.vt1), 0);

    assert_eq!(new_schedule.total_depot_balance_violation(), 0);

    assert!(new_schedule.can_depot_spawn_vehicle(d.start_depot1, d.vt1));

    assert_eq!(new_schedule.number_of_dummy_tours(), 1);

    assert_equal(
        new_schedule.tour_of(dummy3).unwrap().all_nodes_iter(),
        [d.trip12, d.trip23, d.trip34, d.trip45, d.trip51]
            .iter()
            .cloned(),
    );

    new_schedule.verify_consistency();
}

#[test]
fn replace_vehicle_by_dummy_failure_test() {
    // ARRANGE
    let d = init_test_data();
    let schedule = default_schedule(&d);
    let veh3 = VehicleId::from("veh00003");

    // ACT
    let new_schedule = schedule.replace_vehicle_by_dummy(veh3);

    // ASSERT
    assert!(new_schedule.is_err());
}

#[test]
fn add_path_to_vehicle_tour_with_conflict_test() {
    // ARRANGE
    let d = init_test_data();
    let schedule = default_schedule(&d);
    let veh1 = VehicleId::from("veh00001");

    // ACT
    let new_schedule = schedule
        .add_path_to_vehicle_tour(
            veh1,
            Path::new(vec![d.trip51, d.end_depot3], d.network.clone())
                .unwrap()
                .unwrap(),
        )
        .unwrap();
    // trip14 and end_depot1 are removed

    // ASSERT
    assert_eq!(new_schedule.number_of_vehicles(), 3);
    assert_equal(
        new_schedule
            .tour_of(veh1)
            .unwrap()
            .all_nodes_iter()
            .collect::<Vec<_>>(),
        [d.start_depot2, d.trip31, d.trip51, d.end_depot3]
            .iter()
            .cloned(),
    );
    new_schedule.verify_consistency();
}

#[test]
fn add_path_to_vehicle_tour_with_same_start_depot_test() {
    // ARRANGE
    let d = init_test_data();
    let schedule = default_schedule(&d);
    let veh1 = VehicleId::from("veh00001");

    // ACT
    let new_schedule = schedule
        .add_path_to_vehicle_tour(
            veh1,
            Path::new(vec![d.start_depot2, d.trip12, d.trip23], d.network.clone())
                .unwrap()
                .unwrap(),
        )
        .unwrap();
    // start_depot2 is replaced with start_depot2. So everthing is fine even though start_depot2
    // was full.

    // ASSERT
    assert_eq!(new_schedule.number_of_vehicles(), 3);
    assert_equal(
        new_schedule
            .tour_of(veh1)
            .unwrap()
            .all_nodes_iter()
            .collect::<Vec<_>>(),
        [
            d.start_depot2,
            d.trip12,
            d.trip23,
            d.trip31,
            d.trip14,
            d.end_depot1,
        ]
        .iter()
        .cloned(),
    );
    assert_eq!(
        new_schedule.number_of_vehicles_of_same_type_spawned_at(d.depot1, d.vt2),
        1
    );

    new_schedule.verify_consistency();
}

#[test]
fn add_path_to_vehicle_tour_with_full_start_depot_test() {
    // ARRANGE
    let d = init_test_data();
    let schedule = default_schedule(&d);
    let veh1 = VehicleId::from("veh00001");

    // ACT
    let new_schedule = schedule.add_path_to_vehicle_tour(
        veh1,
        Path::new(vec![d.start_depot1, d.trip12, d.trip23], d.network.clone())
            .unwrap()
            .unwrap(),
    );
    // start_depot2 is replaced with start_depot1. But start_depot1 is full. -> Error.

    // ASSERT
    assert!(new_schedule.is_err());
}

#[test]
fn fit_reassign_test() {
    // ARRANGE
    let d = init_test_data();
    let schedule = default_schedule(&d);
    let veh0 = VehicleId::from("veh00000");
    let veh2 = VehicleId::from("veh00002");
    let segment = Segment::new(d.trip45, d.trip51);

    // ACT
    let new_schedule = schedule.fit_reassign(segment, veh0, veh2).unwrap();

    // ASSERT
    assert_eq!(new_schedule.number_of_vehicles(), 3);
    assert_equal(
        new_schedule.tour_of(veh0).unwrap().all_nodes_iter(),
        [
            d.start_depot1,
            d.trip12,
            d.trip23,
            d.trip34,
            d.trip45,
            d.end_depot2,
        ]
        .iter()
        .cloned(),
    );
    assert_equal(
        new_schedule.tour_of(veh2).unwrap().all_nodes_iter(),
        [
            d.start_depot1,
            d.trip12,
            d.trip23,
            d.trip31,
            d.trip51,
            d.end_depot2,
        ]
        .iter()
        .cloned(),
    );
    new_schedule.verify_consistency();
}

#[test]
fn fit_reassign_with_split_test() {
    // ARRANGE
    let d = init_test_data();
    let schedule = default_schedule(&d)
        .spawn_vehicle_for_path(d.vt2, vec![d.trip31])
        .unwrap()
        .0;
    let veh0 = VehicleId::from("veh00000");
    let veh3 = VehicleId::from("veh00003");
    let segment = Segment::new(d.trip12, d.trip51);

    // ACT
    let new_schedule = schedule.fit_reassign(segment, veh0, veh3).unwrap();

    // ASSERT
    assert_eq!(new_schedule.number_of_vehicles(), 4);
    assert_equal(
        new_schedule.tour_of(veh0).unwrap().all_nodes_iter(),
        [d.start_depot1, d.trip34, d.trip45, d.end_depot2]
            .iter()
            .cloned(),
    );

    assert_equal(
        new_schedule.tour_of(veh3).unwrap().all_nodes_iter(),
        [
            d.start_depot3,
            d.trip12,
            d.trip23,
            d.trip31,
            d.trip51,
            d.end_depot1,
        ]
        .iter()
        .cloned(),
    );
    new_schedule.verify_consistency();
}

#[test]
fn fit_reassign_move_full_tour_test() {
    // ARRANGE
    let d = init_test_data();
    let schedule = default_schedule(&d)
        .spawn_vehicle_for_path(d.vt2, vec![d.trip45_fast, d.trip51])
        .unwrap()
        .0;
    let veh2 = VehicleId::from("veh00002");
    let veh3 = VehicleId::from("veh00003");

    let segment = Segment::new(d.trip12, d.trip31);

    // ACT
    let new_schedule = schedule.fit_reassign(segment, veh2, veh3).unwrap();

    // ASSERT
    assert_eq!(new_schedule.number_of_vehicles(), 3);
    assert!(new_schedule.get_vehicle(veh2).is_err());

    assert_equal(
        new_schedule.tour_of(veh3).unwrap().all_nodes_iter(),
        [
            d.start_depot4,
            d.trip12,
            d.trip23,
            d.trip31,
            d.trip45_fast,
            d.trip51,
            d.end_depot1,
        ]
        .iter()
        .cloned(),
    );
    new_schedule.verify_consistency();
}

#[test]
fn fit_reassign_fits_but_cannot_be_removed_test() {
    // ARRANGE
    let d = init_test_data();
    let schedule = default_schedule(&d)
        .spawn_vehicle_for_path(d.vt2, vec![d.trip14, d.trip45_fast, d.trip51])
        .unwrap()
        .0
        .spawn_vehicle_for_path(d.vt2, vec![d.trip34, d.trip51])
        .unwrap()
        .0;
    let veh3 = VehicleId::from("veh00003");
    let veh4 = VehicleId::from("veh00004");
    let segment = Segment::new(d.trip45_fast, d.trip45_fast);

    // ACT
    let new_schedule = schedule.fit_reassign(segment, veh3, veh4).unwrap();
    // even though the segment fits, it cannot be removed from veh3 as the dead_head_trip is
    // slower than trip45_fast.
    // Also note that depot1 is full, so veh4 spawns from depot3.
    // As depot3 is full then, veh4 despawns at depot4.

    // ASSERT
    assert_equal(
        new_schedule.tour_of(veh3).unwrap().all_nodes_iter(),
        [
            d.start_depot3,
            d.trip14,
            d.trip45_fast,
            d.trip51,
            d.end_depot1,
        ]
        .iter()
        .cloned(),
    );

    assert_equal(
        new_schedule.tour_of(veh4).unwrap().all_nodes_iter(),
        [d.start_depot4, d.trip34, d.trip51, d.end_depot1]
            .iter()
            .cloned(),
    );
    new_schedule.verify_consistency();
}

#[test]
fn fit_reassign_move_start_depot_test() {
    // ARRANGE
    let d = init_test_data();
    let schedule = default_schedule(&d);
    let veh1 = VehicleId::from("veh00001");
    let veh2 = VehicleId::from("veh00002");
    let segment = Segment::new(d.start_depot1, d.trip31);

    // ACT
    let new_schedule = schedule.fit_reassign(segment, veh2, veh1).unwrap();
    // start_depot1 of veh2 is not moved as it would conflict with start_depot2 of veh1.

    // ASSERT
    assert_equal(
        new_schedule.tour_of(veh1).unwrap().all_nodes_iter(),
        [
            d.start_depot2,
            d.trip12,
            d.trip23,
            d.trip31,
            d.trip14,
            d.end_depot1,
        ]
        .iter()
        .cloned(),
    );

    assert_equal(
        new_schedule.tour_of(veh2).unwrap().all_nodes_iter(),
        [d.start_depot1, d.trip31, d.end_depot2].iter().cloned(),
    );
    new_schedule.verify_consistency();
}

#[test]
fn fit_reassign_move_end_depot_test() {
    // ARRANGE
    let d = init_test_data();
    let schedule = default_schedule(&d);
    let veh1 = VehicleId::from("veh00001");
    let veh2 = VehicleId::from("veh00002");
    let segment = Segment::new(d.trip31, d.end_depot1);

    // ACT
    let new_schedule = schedule.fit_reassign(segment, veh1, veh2).unwrap();
    // end_depot1 of veh1 is not moved as it would conflict with end_depot2 of veh2.

    // ASSERT
    assert_equal(
        new_schedule.tour_of(veh1).unwrap().all_nodes_iter(),
        [d.start_depot2, d.trip31, d.end_depot1].iter().cloned(),
    );

    assert_equal(
        new_schedule.tour_of(veh2).unwrap().all_nodes_iter(),
        [
            d.start_depot1,
            d.trip12,
            d.trip23,
            d.trip31,
            d.trip14,
            d.end_depot2,
        ]
        .iter()
        .cloned(),
    );
    new_schedule.verify_consistency();
}

#[test]
fn fit_reassign_dummy_provider_test() {
    // ARRANGE
    let d = init_test_data();
    let veh0 = VehicleId::from("veh00000");
    let veh2 = VehicleId::from("veh00002");
    let dummy3 = VehicleId::from("dummy00003");
    let schedule = default_schedule(&d).replace_vehicle_by_dummy(veh0).unwrap();
    let segment = Segment::new(d.trip45, d.trip51);

    // ACT
    let new_schedule = schedule.fit_reassign(segment, dummy3, veh2).unwrap();

    // ASSERT
    assert_eq!(new_schedule.number_of_vehicles(), 2);
    assert_eq!(new_schedule.number_of_dummy_tours(), 1);
    assert_equal(
        new_schedule.tour_of(dummy3).unwrap().all_nodes_iter(),
        [d.trip12, d.trip23, d.trip34, d.trip45].iter().cloned(),
    );
    assert_equal(
        new_schedule.tour_of(veh2).unwrap().all_nodes_iter(),
        [
            d.start_depot1,
            d.trip12,
            d.trip23,
            d.trip31,
            d.trip51,
            d.end_depot2,
        ]
        .iter()
        .cloned(),
    );
    new_schedule.verify_consistency();
}

#[test]
fn fit_reassign_dummy_receiver_test() {
    // ARRANGE
    let d = init_test_data();
    let veh0 = VehicleId::from("veh00000");
    let veh2 = VehicleId::from("veh00002");
    let dummy3 = VehicleId::from("dummy00003");
    let schedule = default_schedule(&d).replace_vehicle_by_dummy(veh2).unwrap();
    let segment = Segment::new(d.trip45, d.trip51);

    // ACT
    let new_schedule = schedule.fit_reassign(segment, veh0, dummy3).unwrap();

    // ASSERT
    assert_eq!(new_schedule.number_of_vehicles(), 2);
    assert_eq!(new_schedule.number_of_dummy_tours(), 1);
    assert_equal(
        new_schedule.tour_of(veh0).unwrap().all_nodes_iter(),
        [
            d.start_depot1,
            d.trip12,
            d.trip23,
            d.trip34,
            d.trip45,
            d.end_depot2,
        ]
        .iter()
        .cloned(),
    );
    assert_equal(
        new_schedule.tour_of(dummy3).unwrap().all_nodes_iter(),
        [d.trip12, d.trip23, d.trip31, d.trip51].iter().cloned(),
    );
    new_schedule.verify_consistency();
}

#[test]
fn fit_reassign_dummy_provider_and_receiver_test() {
    // ARRANGE
    let d = init_test_data();
    let veh0 = VehicleId::from("veh00000");
    let veh2 = VehicleId::from("veh00002");
    let dummy3 = VehicleId::from("dummy00003");
    let dummy4 = VehicleId::from("dummy00004");
    let schedule = default_schedule(&d)
        .replace_vehicle_by_dummy(veh0)
        .unwrap()
        .replace_vehicle_by_dummy(veh2)
        .unwrap();
    let segment = Segment::new(d.trip45, d.trip51);

    // ACT
    let new_schedule = schedule.fit_reassign(segment, dummy3, dummy4).unwrap();

    // ASSERT
    assert_eq!(new_schedule.number_of_vehicles(), 1);
    assert_eq!(new_schedule.number_of_dummy_tours(), 2);
    assert_equal(
        new_schedule.tour_of(dummy3).unwrap().all_nodes_iter(),
        [d.trip12, d.trip23, d.trip34, d.trip45].iter().cloned(),
    );
    assert_equal(
        new_schedule.tour_of(dummy4).unwrap().all_nodes_iter(),
        [d.trip12, d.trip23, d.trip31, d.trip51].iter().cloned(),
    );
    new_schedule.verify_consistency();
}

#[test]
fn override_reassign_test() {
    // ARRANGE
    let d = init_test_data();
    let schedule = default_schedule(&d);
    let veh0 = VehicleId::from("veh00000");
    let veh2 = VehicleId::from("veh00002");
    let segment = Segment::new(d.trip45, d.trip51);

    // ACT
    let (new_schedule, dummy_opt) = schedule.override_reassign(segment, veh0, veh2).unwrap();

    // ASSERT
    assert_eq!(new_schedule.number_of_vehicles(), 3);
    assert_equal(
        new_schedule.tour_of(veh0).unwrap().all_nodes_iter(),
        [d.start_depot1, d.trip12, d.trip23, d.trip34, d.end_depot2]
            .iter()
            .cloned(),
    );
    assert_equal(
        new_schedule.tour_of(veh2).unwrap().all_nodes_iter(),
        [
            d.start_depot1,
            d.trip12,
            d.trip23,
            d.trip45,
            d.trip51,
            d.end_depot2,
        ]
        .iter()
        .cloned(),
    );

    assert!(dummy_opt.is_some());
    let dummy3 = dummy_opt.unwrap();
    assert_eq!(dummy3, VehicleId::from("dummy00003"));

    assert_equal(
        new_schedule.tour_of(dummy3).unwrap().all_nodes_iter(),
        [d.trip31].iter().cloned(),
    );
    new_schedule.verify_consistency();
}

#[test]
fn override_reassign_move_full_tour_test() {
    // ARRANGE
    let d = init_test_data();
    let schedule = default_schedule(&d)
        .spawn_vehicle_for_path(d.vt2, vec![d.trip31])
        .unwrap()
        .0;
    let veh0 = VehicleId::from("veh00000");
    let veh3 = VehicleId::from("veh00003");
    let segment = Segment::new(d.trip12, d.trip51);

    // ACT
    let (new_schedule, dummy_opt) = schedule.override_reassign(segment, veh0, veh3).unwrap();

    // ASSERT
    assert_eq!(new_schedule.number_of_vehicles(), 3);

    assert!(new_schedule.get_vehicle(veh0).is_err());

    assert_equal(
        new_schedule.tour_of(veh3).unwrap().all_nodes_iter(),
        [
            d.start_depot3,
            d.trip12,
            d.trip23,
            d.trip34,
            d.trip45,
            d.trip51,
            d.end_depot1,
        ]
        .iter()
        .cloned(),
    );

    assert!(dummy_opt.is_some());
    let dummy4 = dummy_opt.unwrap();
    assert_eq!(dummy4, VehicleId::from("dummy00004"));

    assert_equal(
        new_schedule.tour_of(dummy4).unwrap().all_nodes_iter(),
        [d.trip31].iter().cloned(),
    );
    new_schedule.verify_consistency();
}

#[test]
fn override_reassign_fits_but_cannot_be_removed_test() {
    // ARRANGE
    let d = init_test_data();
    let schedule = default_schedule(&d)
        .spawn_vehicle_for_path(d.vt2, vec![d.trip14, d.trip45_fast, d.trip51])
        .unwrap()
        .0
        .spawn_vehicle_for_path(d.vt2, vec![d.trip34, d.trip51])
        .unwrap()
        .0;
    let veh3 = VehicleId::from("veh00003");
    let veh4 = VehicleId::from("veh00004");
    let segment = Segment::new(d.trip45_fast, d.trip45_fast);

    // ACT
    let result = schedule.override_reassign(segment, veh3, veh4);
    // even though the segment fits, it cannot be removed from veh3 as the dead_head_trip is
    // slower than trip45_fast. -> Error.

    // ASSERT
    assert!(result.is_err());
}

#[test]
fn override_reassign_no_new_dummy_test() {
    // ARRANGE
    let d = init_test_data();
    let schedule = default_schedule(&d)
        .spawn_vehicle_for_path(d.vt2, vec![d.trip45_fast, d.trip51])
        .unwrap()
        .0;
    let veh2 = VehicleId::from("veh00002");
    let veh3 = VehicleId::from("veh00003");
    let segment = Segment::new(d.trip23, d.trip31);

    // ACT
    let (new_schedule, dummy_opt) = schedule.override_reassign(segment, veh2, veh3).unwrap();

    // ASSERT
    assert_eq!(new_schedule.number_of_vehicles(), 4);
    assert!(dummy_opt.is_none());

    assert_equal(
        new_schedule.tour_of(veh2).unwrap().all_nodes_iter(),
        [d.start_depot1, d.trip12, d.end_depot2].iter().cloned(),
    );

    assert_equal(
        new_schedule.tour_of(veh3).unwrap().all_nodes_iter(),
        [
            d.start_depot4,
            d.trip23,
            d.trip31,
            d.trip45_fast,
            d.trip51,
            d.end_depot1,
        ]
        .iter()
        .cloned(),
    );
}

#[test]
fn override_reassign_move_all_non_depots() {
    // ARRANGE
    let d = init_test_data();
    let schedule = default_schedule(&d);
    let veh1 = VehicleId::from("veh00001");
    let veh2 = VehicleId::from("veh00002");
    let segment = Segment::new(d.trip12, d.trip31);

    // ACT
    let (new_schedule, dummy_opt) = schedule.override_reassign(segment, veh2, veh1).unwrap();

    // ASSERT
    assert_eq!(new_schedule.number_of_vehicles(), 2);
    assert_equal(
        new_schedule.tour_of(veh1).unwrap().all_nodes_iter(),
        [
            d.start_depot2,
            d.trip12,
            d.trip23,
            d.trip31,
            d.trip14,
            d.end_depot1,
        ]
        .iter()
        .cloned(),
    );

    assert!(new_schedule.get_vehicle(veh2).is_err());

    assert!(dummy_opt.is_some());

    assert_equal(
        new_schedule
            .tour_of(dummy_opt.unwrap())
            .unwrap()
            .all_nodes_iter(),
        [d.trip31].iter().cloned(),
    );

    new_schedule.verify_consistency();
}

#[test]
fn override_reassign_move_start_depot_with_remaining_trip_test() {
    // ARRANGE
    let d = init_test_data();
    let schedule = default_schedule(&d);
    let veh1 = VehicleId::from("veh00001");
    let veh2 = VehicleId::from("veh00002");
    let segment = Segment::new(d.start_depot1, d.trip23);

    // ACT
    let result = schedule.override_reassign(segment, veh2, veh1);

    // ASSERT
    assert!(result.is_err());
}

#[test]
fn override_reassign_move_start_depot_no_remaining_trip_test() {
    // ARRANGE
    let d = init_test_data();
    let schedule = default_schedule(&d);
    let veh1 = VehicleId::from("veh00001");
    let veh2 = VehicleId::from("veh00002");
    let segment = Segment::new(d.start_depot1, d.trip31);

    // ACT
    let (new_schedule, dummy_opt) = schedule.override_reassign(segment, veh2, veh1).unwrap();

    // ASSERT
    assert_eq!(new_schedule.number_of_vehicles(), 2);
    assert_equal(
        new_schedule.tour_of(veh1).unwrap().all_nodes_iter(),
        [
            d.start_depot1,
            d.trip12,
            d.trip23,
            d.trip31,
            d.trip14,
            d.end_depot1,
        ]
        .iter()
        .cloned(),
    );

    assert!(new_schedule.get_vehicle(veh2).is_err());

    assert!(dummy_opt.is_some());

    assert_equal(
        new_schedule
            .tour_of(dummy_opt.unwrap())
            .unwrap()
            .all_nodes_iter(),
        [d.trip31].iter().cloned(),
    );

    new_schedule.verify_consistency();
}

#[test]
fn override_reassign_move_end_depot_with_remaining_trip_test() {
    // ARRANGE
    let d = init_test_data();
    let schedule = default_schedule(&d);
    let veh1 = VehicleId::from("veh00001");
    let veh2 = VehicleId::from("veh00002");
    let segment = Segment::new(d.trip14, d.end_depot1);

    // ACT
    let result = schedule.override_reassign(segment, veh1, veh2);

    // ASSERT
    assert!(result.is_err());
}

#[test]
fn override_reassign_move_end_depot_no_remaining_trip_test() {
    // ARRANGE
    let d = init_test_data();
    let schedule = default_schedule(&d);
    let veh1 = VehicleId::from("veh00001");
    let veh2 = VehicleId::from("veh00002");
    let segment = Segment::new(d.trip31, d.end_depot1);

    // ACT
    let (new_schedule, dummy_opt) = schedule.override_reassign(segment, veh1, veh2).unwrap();

    // ASSERT
    assert_eq!(new_schedule.number_of_vehicles(), 2);

    assert!(new_schedule.get_vehicle(veh1).is_err());

    assert_equal(
        new_schedule.tour_of(veh2).unwrap().all_nodes_iter(),
        [
            d.start_depot1,
            d.trip12,
            d.trip23,
            d.trip31,
            d.trip14,
            d.end_depot1,
        ]
        .iter()
        .cloned(),
    );

    assert!(dummy_opt.is_some());

    assert_equal(
        new_schedule
            .tour_of(dummy_opt.unwrap())
            .unwrap()
            .all_nodes_iter(),
        [d.trip31].iter().cloned(),
    );

    new_schedule.verify_consistency();
}

#[test]
fn override_reassign_dummy_provider_test() {
    // ARRANGE
    let d = init_test_data();
    let veh0 = VehicleId::from("veh00000");
    let veh2 = VehicleId::from("veh00002");
    let dummy3 = VehicleId::from("dummy00003");
    let schedule = default_schedule(&d).replace_vehicle_by_dummy(veh0).unwrap();
    let segment = Segment::new(d.trip45, d.trip51);

    // ACT
    let (new_schedule, dummy_opt) = schedule.override_reassign(segment, dummy3, veh2).unwrap();

    // ASSERT
    assert_eq!(new_schedule.number_of_vehicles(), 2);
    assert_eq!(new_schedule.number_of_dummy_tours(), 2);
    assert_equal(
        new_schedule.tour_of(dummy3).unwrap().all_nodes_iter(),
        [d.trip12, d.trip23, d.trip34].iter().cloned(),
    );
    assert_equal(
        new_schedule.tour_of(veh2).unwrap().all_nodes_iter(),
        [
            d.start_depot1,
            d.trip12,
            d.trip23,
            d.trip45,
            d.trip51,
            d.end_depot2,
        ]
        .iter()
        .cloned(),
    );

    assert!(dummy_opt.is_some());
    let dummy4 = dummy_opt.unwrap();
    assert_eq!(dummy4, VehicleId::from("dummy00004"));

    assert_equal(
        new_schedule.tour_of(dummy4).unwrap().all_nodes_iter(),
        [d.trip31].iter().cloned(),
    );
    new_schedule.verify_consistency();
}

#[test]
fn override_reassign_dummy_receiver_test() {
    // ARRANGE
    let d = init_test_data();
    let veh0 = VehicleId::from("veh00000");
    let veh2 = VehicleId::from("veh00002");
    let dummy3 = VehicleId::from("dummy00003");
    let schedule = default_schedule(&d).replace_vehicle_by_dummy(veh2).unwrap();
    let segment = Segment::new(d.trip45, d.trip51);

    // ACT
    let (new_schedule, dummy_opt) = schedule.override_reassign(segment, veh0, dummy3).unwrap();

    // ASSERT
    assert_eq!(new_schedule.number_of_vehicles(), 2);
    assert_eq!(new_schedule.number_of_dummy_tours(), 2);
    assert_equal(
        new_schedule.tour_of(veh0).unwrap().all_nodes_iter(),
        [d.start_depot1, d.trip12, d.trip23, d.trip34, d.end_depot2]
            .iter()
            .cloned(),
    );
    assert_equal(
        new_schedule.tour_of(dummy3).unwrap().all_nodes_iter(),
        [d.trip12, d.trip23, d.trip45, d.trip51].iter().cloned(),
    );

    assert!(dummy_opt.is_some());
    let dummy4 = dummy_opt.unwrap();
    assert_eq!(dummy4, VehicleId::from("dummy00004"));

    assert_equal(
        new_schedule.tour_of(dummy4).unwrap().all_nodes_iter(),
        [d.trip31].iter().cloned(),
    );
    new_schedule.verify_consistency();
}

#[test]
fn override_reassign_dummy_provider_and_receiver_test() {
    // ARRANGE
    let d = init_test_data();
    let veh0 = VehicleId::from("veh00000");
    let veh2 = VehicleId::from("veh00002");
    let dummy3 = VehicleId::from("dummy00003");
    let dummy4 = VehicleId::from("dummy00004");
    let schedule = default_schedule(&d)
        .replace_vehicle_by_dummy(veh0)
        .unwrap()
        .replace_vehicle_by_dummy(veh2)
        .unwrap();
    let segment = Segment::new(d.trip45, d.trip51);

    // ACT
    let (new_schedule, dummy_opt) = schedule.override_reassign(segment, dummy3, dummy4).unwrap();

    // ASSERT
    assert_eq!(new_schedule.number_of_vehicles(), 1);
    assert_eq!(new_schedule.number_of_dummy_tours(), 3);
    assert_equal(
        new_schedule.tour_of(dummy3).unwrap().all_nodes_iter(),
        [d.trip12, d.trip23, d.trip34].iter().cloned(),
    );
    assert_equal(
        new_schedule.tour_of(dummy4).unwrap().all_nodes_iter(),
        [d.trip12, d.trip23, d.trip45, d.trip51].iter().cloned(),
    );

    assert!(dummy_opt.is_some());
    let dummy5 = dummy_opt.unwrap();
    assert_eq!(dummy5, VehicleId::from("dummy00005"));

    assert_equal(
        new_schedule.tour_of(dummy5).unwrap().all_nodes_iter(),
        [d.trip31].iter().cloned(),
    );
    new_schedule.verify_consistency();
}

#[test]
fn improve_depots_test() {
    // ARRANGE
    let d = init_test_data();
    let schedule = Schedule::empty(d.vehicle_types.clone(), d.network.clone(), d.config.clone())
        .spawn_vehicle_for_path(
            d.vt1,
            vec![d.start_depot3, d.trip23, d.trip34, d.end_depot2],
        )
        .unwrap()
        .0
        .spawn_vehicle_for_path(
            d.vt1,
            vec![d.start_depot5, d.trip12, d.trip23, d.end_depot1],
        )
        .unwrap()
        .0;
    let veh0 = VehicleId::from("veh00000");
    let veh1 = VehicleId::from("veh00001");

    // ACT
    let new_schedule = schedule.improve_depots(Some(vec![veh0]));
    // veh0 is moved from depot3 to depot1
    let new_schedule2 = new_schedule.improve_depots(None);
    // veh0 is moved from depot3 to depot1
    // veh1 is moved from depot5 to depot3 (as depot1 is full)

    // ASSERT
    assert_equal(
        new_schedule.tour_of(veh0).unwrap().all_nodes_iter(),
        [d.start_depot1, d.trip23, d.trip34, d.end_depot4]
            .iter()
            .cloned(),
    );
    assert_equal(
        new_schedule.tour_of(veh1).unwrap().all_nodes_iter(),
        [d.start_depot5, d.trip12, d.trip23, d.end_depot1]
            .iter()
            .cloned(),
    );
    new_schedule.verify_consistency();

    assert_equal(
        new_schedule2.tour_of(veh0).unwrap().all_nodes_iter(),
        [d.start_depot1, d.trip23, d.trip34, d.end_depot4]
            .iter()
            .cloned(),
    );
    assert_equal(
        new_schedule2.tour_of(veh1).unwrap().all_nodes_iter(),
        [d.start_depot3, d.trip12, d.trip23, d.end_depot3]
            .iter()
            .cloned(),
    );
    new_schedule2.verify_consistency();
}

#[test]
fn reassign_end_depots_greedily_test() {
    // ARRANGE
    let d = init_test_data();
    let schedule = Schedule::empty(d.vehicle_types.clone(), d.network.clone(), d.config.clone())
        .spawn_vehicle_for_path(
            d.vt1,
            vec![d.start_depot3, d.trip23, d.trip34, d.end_depot2],
        )
        .unwrap()
        .0
        .spawn_vehicle_for_path(
            d.vt2,
            vec![d.start_depot3, d.trip12, d.trip23, d.end_depot1],
        )
        .unwrap()
        .0;
    let veh0 = VehicleId::from("veh00000");
    let veh1 = VehicleId::from("veh00001");

    // ACT
    let new_schedule = schedule.reassign_end_depots_greedily().unwrap();

    // ASSERT
    assert_equal(
        new_schedule.tour_of(veh0).unwrap().all_nodes_iter(),
        [d.start_depot3, d.trip23, d.trip34, d.end_depot4]
            .iter()
            .cloned(),
    );
    assert_equal(
        new_schedule.tour_of(veh1).unwrap().all_nodes_iter(),
        [d.start_depot3, d.trip12, d.trip23, d.end_depot3]
            .iter()
            .cloned(),
    );
    new_schedule.verify_consistency();
}
