use std::collections::HashMap;

use time::{DateTime, Duration};

use crate::{
    base_types::{Distance, Location, LocationId, NodeId, StationSide, VehicleTypeId},
    json_serialisation::load_rolling_stock_problem_instance_from_json,
    locations::Locations,
    network::nodes::Node,
    vehicle_types::VehicleType,
};

//add a test that reads a json file
#[test]
fn test_load_from_json() {
    let (locations, vehicle_types, network, config) =
        load_rolling_stock_problem_instance_from_json("resources/small_test_input.json");

    let loc1 = locations.get_location(LocationId::from("loc1"));
    let loc2 = locations.get_location(LocationId::from("loc2"));
    let loc3 = locations.get_location(LocationId::from("loc3"));

    assert_eq!(
        *vehicle_types.get(VehicleTypeId::from("vt1")).unwrap(),
        VehicleType::new(
            VehicleTypeId::from("vt1"),
            String::from("Vehicle Type 1"),
            50,
            100,
            10,
        )
    );
    assert_eq!(
        *vehicle_types.get(VehicleTypeId::from("vt2")).unwrap(),
        VehicleType::new(
            VehicleTypeId::from("vt2"),
            String::from("Vehicle Type 2"),
            40,
            80,
            8,
        )
    );

    assert_eq!(loc1, Location::of(LocationId::from("loc1")));
    assert_eq!(loc2, Location::of(LocationId::from("loc2")));
    assert_eq!(loc3, Location::of(LocationId::from("loc3")));

    assert_eq!(network.all_nodes().count(), 6);
    assert_eq!(
        *network.node(NodeId::from("start_depot1")),
        Node::create_start_depot_node(
            NodeId::from("start_depot1"),
            loc1,
            HashMap::from([
                (VehicleTypeId::from("vt1"), Some(7)),
                (VehicleTypeId::from("vt2"), Some(5))
            ]),
            String::from("start_depot(depot1,loc1)"),
        )
    );
    assert_eq!(
        *network.node(NodeId::from("end_depot1")),
        Node::create_end_depot_node(
            NodeId::from("end_depot1"),
            loc1,
            HashMap::from([
                (VehicleTypeId::from("vt1"), Some(7)),
                (VehicleTypeId::from("vt2"), Some(5))
            ]),
            String::from("end_depot(depot1,loc1)"),
        )
    );
    assert_eq!(
        *network.node(NodeId::from("start_depot2")),
        Node::create_start_depot_node(
            NodeId::from("start_depot2"),
            loc2,
            HashMap::from([(VehicleTypeId::from("vt2"), Some(8))]),
            String::from("start_depot(depot2,loc2)"),
        )
    );
    assert_eq!(
        *network.node(NodeId::from("end_depot2")),
        Node::create_end_depot_node(
            NodeId::from("end_depot2"),
            loc2,
            HashMap::from([(VehicleTypeId::from("vt2"), Some(8))]),
            String::from("end_depot(depot2,loc2)"),
        )
    );

    assert_eq!(
        *network.node(NodeId::from("trip1")),
        Node::create_service_node(
            NodeId::from("trip1"),
            loc1,
            loc2,
            DateTime::new("2023-07-24T11:59:41"),
            DateTime::new("2023-07-24T12:59:41"),
            StationSide::Front,
            StationSide::Front,
            Distance::from_meter(1000),
            50,
            String::from("Trip 1"),
        )
    );

    assert_eq!(
        *network.node(NodeId::from("trip2")),
        Node::create_service_node(
            NodeId::from("trip2"),
            loc2,
            loc3,
            DateTime::new("2023-07-24T11:59:41"),
            DateTime::new("2023-07-24T13:59:41"),
            StationSide::Front,
            StationSide::Front,
            Distance::from_meter(2000),
            80,
            String::from("Trip 2"),
        )
    );

    assert_travel_time(loc1, loc1, 0, &locations);
    assert_travel_time(loc1, loc2, 600, &locations);
    assert_travel_time(loc1, loc3, 300, &locations);
    assert_travel_time(loc2, loc1, 6000, &locations);
    assert_travel_time(loc2, loc2, 0, &locations);
    assert_travel_time(loc2, loc3, 400, &locations);
    assert_travel_time(loc3, loc1, 3000, &locations);
    assert_travel_time(loc3, loc2, 4000, &locations);
    assert_travel_time(loc3, loc3, 0, &locations);

    assert_travel_distance(loc1, loc1, 0, &locations);
    assert_travel_distance(loc1, loc2, 1000, &locations);
    assert_travel_distance(loc1, loc3, 500, &locations);
    assert_travel_distance(loc2, loc1, 10000, &locations);
    assert_travel_distance(loc2, loc2, 0, &locations);
    assert_travel_distance(loc2, loc3, 700, &locations);
    assert_travel_distance(loc3, loc1, 5000, &locations);
    assert_travel_distance(loc3, loc2, 7000, &locations);
    assert_travel_distance(loc3, loc3, 0, &locations);

    assert_eq!(
        config.durations_between_activities.minimal,
        Duration::from_seconds(600)
    );
    assert_eq!(
        config.durations_between_activities.dead_head_trip,
        Duration::from_seconds(300)
    );
    assert_eq!(
        config.default_maximal_formation_length,
        Distance::from_meter(20)
    );
}

fn assert_travel_time(from: Location, to: Location, expected: u32, locations: &Locations) {
    assert_eq!(
        locations.travel_time(from, to),
        Duration::from_seconds(expected),
        "Travel time from {} to {} should be {}",
        from,
        to,
        expected
    );
}

fn assert_travel_distance(from: Location, to: Location, expected: u64, locations: &Locations) {
    assert_eq!(
        locations.distance(from, to),
        Distance::from_meter(expected),
        "Travel distance from {} to {} should be {}",
        from,
        to,
        expected
    );
}
