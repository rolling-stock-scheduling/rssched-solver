use itertools::assert_equal;
use model::base_types::{DepotId, Distance, NodeId, VehicleId, VehicleTypeId};

use crate::{
    path::Path,
    segment::Segment,
    test_utilities::{init_test_data, TestData},
    Schedule,
};

fn default_schedule(d: &TestData) -> Schedule {
    let mut schedule =
        Schedule::empty(d.vehicle_types.clone(), d.network.clone(), d.config.clone());
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
        .unwrap();

    schedule = schedule
        .spawn_vehicle_for_path(
            d.vt2,
            vec![d.start_depot2, d.trip31, d.trip14, d.end_depot1],
        )
        .unwrap();

    schedule = schedule
        .spawn_vehicle_for_path(
            d.vt2,
            vec![d.start_depot1, d.trip12, d.trip23, d.trip31, d.end_depot2],
        )
        .unwrap();

    schedule
}

#[test]
fn basic_methods_test() {
    // ARRANGE
    let d = init_test_data();
    let veh000 = VehicleId::from("veh000");
    let veh001 = VehicleId::from("veh001");
    let veh002 = VehicleId::from("veh002");

    // ACT
    let schedule = default_schedule(&d);

    // ASSERT
    assert_eq!(schedule.number_of_vehicles(), 3);
    assert_equal(
        schedule.vehicles_iter(),
        [veh000, veh001, veh002].iter().cloned(),
    );
    assert!(schedule.is_vehicle(veh000));
    assert!(schedule.is_vehicle(veh001));
    assert!(schedule.is_vehicle(veh002));
    assert!(!schedule.is_vehicle(VehicleId::from("veh003")));

    assert_eq!(schedule.get_vehicle(veh000).unwrap().id(), veh000);
    assert_eq!(schedule.get_vehicle(veh000).unwrap().type_id(), d.vt1);

    assert_eq!(schedule.get_vehicle(veh001).unwrap().id(), veh001);
    assert_eq!(schedule.get_vehicle(veh001).unwrap().type_id(), d.vt2);

    assert_eq!(schedule.get_vehicle(veh002).unwrap().id(), veh002);
    assert_eq!(schedule.get_vehicle(veh002).unwrap().type_id(), d.vt2);

    assert!(schedule.get_vehicle(VehicleId::from("veh004")).is_err());

    assert_eq!(schedule.vehicle_type_of(veh000), d.vt1);
    assert_eq!(schedule.vehicle_type_of(veh001), d.vt2);
    assert_eq!(schedule.vehicle_type_of(veh002), d.vt2);

    assert!(!schedule.is_dummy(veh000));
    assert!(!schedule.is_dummy(veh001));
    assert!(!schedule.is_dummy(veh002));
    assert!(!schedule.is_dummy(VehicleId::from("veh003")));

    assert_eq!(schedule.number_of_dummy_tours(), 0);
    assert!(schedule.dummy_iter().next().is_none());

    assert_eq!(schedule.tour_of(veh000).unwrap().len(), 7);
    assert_eq!(schedule.tour_of(veh001).unwrap().len(), 4);
    assert_eq!(schedule.tour_of(veh002).unwrap().len(), 5);

    assert_eq!(
        schedule.train_formation_of(d.trip12).ids(),
        vec![veh000, veh002]
    );
    assert_eq!(
        schedule.train_formation_of(d.trip23).ids(),
        vec![veh000, veh002]
    );
    assert_eq!(schedule.train_formation_of(d.trip34).ids(), vec![veh000]);
    assert_eq!(schedule.train_formation_of(d.trip45).ids(), vec![veh000]);
    assert_eq!(schedule.train_formation_of(d.trip51).ids(), vec![veh000]);
    assert_eq!(
        schedule.train_formation_of(d.trip31).ids(),
        vec![veh001, veh002]
    );
    assert_eq!(schedule.train_formation_of(d.trip14).ids(), vec![veh001]);

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
    assert_eq!(schedule.depot_balance(DepotId::from("depot3"), d.vt1), 0);

    assert_eq!(schedule.total_depot_balance_violation(), 2);

    assert!(!schedule.can_depot_spawn_vehicle(d.start_depot1, d.vt1));
    assert!(!schedule.can_depot_spawn_vehicle(d.start_depot1, d.vt2));
    assert!(!schedule.can_depot_spawn_vehicle(d.start_depot2, d.vt1));
    assert!(!schedule.can_depot_spawn_vehicle(d.start_depot2, d.vt2));
    assert!(!schedule.can_depot_spawn_vehicle(NodeId::from("s_depot4"), d.vt1));
    assert!(schedule.can_depot_spawn_vehicle(NodeId::from("s_depot4"), d.vt2));
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
fn spawn_vehicle_for_path_without_depots_test() {
    // ARRANGE
    let d = init_test_data();
    let schedule = default_schedule(&d);

    // ACT
    let new_schedule = schedule
        .spawn_vehicle_for_path(
            d.vt1,
            vec![d.trip12, d.trip23, d.trip34, d.trip45, d.trip51],
        )
        .unwrap();

    // vehicle spawns from depot3 as depot1 and 2 are full.
    // de-spawns at depot1 as it is the closest.

    // ASSERT
    assert_eq!(new_schedule.number_of_vehicles(), 4);
    assert_equal(
        new_schedule
            .tour_of(VehicleId::from("veh003"))
            .unwrap()
            .all_nodes_iter(),
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
fn delete_vehicle_success_test() {
    // ARRANGE
    let d = init_test_data();
    let schedule = default_schedule(&d);
    let veh000 = VehicleId::from("veh000");
    let veh002 = VehicleId::from("veh002");

    // ACT
    let new_schedule = schedule.delete_vehicle(veh000).unwrap();

    // ASSERT
    assert_eq!(new_schedule.number_of_vehicles(), 2);
    assert!(new_schedule.get_vehicle(veh000).is_err());
    assert!(new_schedule.tour_of(veh000).is_err());
    assert_equal(
        new_schedule.train_formation_of(d.trip12).ids(),
        [veh002].iter().cloned(),
    );
    assert_equal(
        new_schedule.train_formation_of(d.trip23).ids(),
        [veh002].iter().cloned(),
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

    new_schedule.verify_consistency();
}

#[test]
fn delete_vehicle_failure_test() {
    // ARRANGE
    let d = init_test_data();
    let schedule = default_schedule(&d);
    let veh003 = VehicleId::from("veh003");

    // ACT
    let new_schedule = schedule.delete_vehicle(veh003);

    // ASSERT
    assert!(new_schedule.is_err());
}

#[test]
fn add_path_to_vehicle_tour_with_conflict_test() {
    // ARRANGE
    let d = init_test_data();
    let schedule = default_schedule(&d);
    let veh001 = VehicleId::from("veh001");

    // ACT
    let new_schedule = schedule
        .add_path_to_vehicle_tour(
            veh001,
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
            .tour_of(veh001)
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
    let veh001 = VehicleId::from("veh001");

    // ACT
    let new_schedule = schedule
        .add_path_to_vehicle_tour(
            veh001,
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
            .tour_of(veh001)
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
    let veh001 = VehicleId::from("veh001");

    // ACT
    let new_schedule = schedule.add_path_to_vehicle_tour(
        veh001,
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
    let veh000 = VehicleId::from("veh000");
    let veh002 = VehicleId::from("veh002");
    let segment = Segment::new(d.trip45, d.trip51);

    // ACT
    let new_schedule = schedule.fit_reassign(segment, veh000, veh002).unwrap();

    // ASSERT
    assert_eq!(new_schedule.number_of_vehicles(), 3);
    assert_equal(
        new_schedule.tour_of(veh000).unwrap().all_nodes_iter(),
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
        new_schedule.tour_of(veh002).unwrap().all_nodes_iter(),
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
}

#[test]
fn fit_reassign_with_split_test() {
    // ARRANGE
    let d = init_test_data();
    let schedule = default_schedule(&d)
        .spawn_vehicle_for_path(d.vt2, vec![d.trip31])
        .unwrap();
    let veh000 = VehicleId::from("veh000");
    let veh003 = VehicleId::from("veh003");
    let segment = Segment::new(d.trip12, d.trip51);

    // ACT
    let new_schedule = schedule.fit_reassign(segment, veh000, veh003).unwrap();

    // ASSERT
    assert_eq!(new_schedule.number_of_vehicles(), 4);
    assert_equal(
        new_schedule.tour_of(veh000).unwrap().all_nodes_iter(),
        [d.start_depot1, d.trip34, d.trip45, d.end_depot2]
            .iter()
            .cloned(),
    );

    assert_equal(
        new_schedule.tour_of(veh003).unwrap().all_nodes_iter(),
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
}

#[test]
fn fit_reassign_fits_but_cannot_be_removed() {
    // ARRANGE
    let d = init_test_data();
    let schedule = default_schedule(&d)
        .spawn_vehicle_for_path(d.vt2, vec![d.trip14, d.trip45_fast, d.trip51])
        .unwrap()
        .spawn_vehicle_for_path(d.vt2, vec![d.trip34, d.trip51])
        .unwrap();
    let veh003 = VehicleId::from("veh003");
    let veh004 = VehicleId::from("veh004");
    let segment = Segment::new(d.trip45_fast, d.trip45_fast);

    // ACT
    let new_schedule = schedule.fit_reassign(segment, veh003, veh004).unwrap();
    // even though the segment fits, it cannot be removed from veh003 as the dead_head_trip is
    // slower than trip45_fast.
    // Also note that depot1 is full, so veh004 spawns from depot3.
    // As depot3 is full then, veh004 despawns at depot4.

    // ASSERT
    assert_equal(
        new_schedule.tour_of(veh003).unwrap().all_nodes_iter(),
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
        new_schedule.tour_of(veh004).unwrap().all_nodes_iter(),
        [d.start_depot4, d.trip34, d.trip51, d.end_depot1]
            .iter()
            .cloned(),
    );

    // TODO: Continue with override_reassign
}
