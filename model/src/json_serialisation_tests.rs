use std::{fs::File, io::Read};

use time::{DateTime, Duration};

use crate::{
    base_types::{DepotId, Distance, Location, LocationId, NodeId, VehicleTypeId},
    json_serialisation::load_rolling_stock_problem_instance_from_json,
    locations::Locations,
    network::nodes::Node,
    vehicle_types::VehicleType,
};

//add a test that reads a json file
#[test]
fn test_load_from_json_no_null() {
    test_load_from_json("resources/small_test_input.json");
}
#[test]
fn test_load_from_json_with_null() {
    test_load_from_json("resources/small_test_input_with_null_values.json");
}

fn test_load_from_json(path: &str) {
    // ACT
    let mut file = File::open(path).unwrap();
    let mut input_data = String::new();
    file.read_to_string(&mut input_data).unwrap();
    let input_data: serde_json::Value = serde_json::from_str(&input_data).unwrap();

    let network = load_rolling_stock_problem_instance_from_json(input_data);
    let vehicle_types = network.vehicle_types();
    let config = network.config();

    // ASSERT
    let locations = network.locations();
    let loc0 = locations.get_location(LocationId::from(0)).unwrap();
    let loc2 = locations.get_location(LocationId::from(2)).unwrap();
    let loc3 = locations.get_location(LocationId::from(3)).unwrap();

    let vt0 = VehicleTypeId::from(0);
    let vt1 = VehicleTypeId::from(1);

    assert_eq!(
        *vehicle_types.get(vt0).unwrap(),
        VehicleType::new(vt0, String::from("IC"), 100, 50, Some(4),)
    );
    assert_eq!(
        *vehicle_types.get(vt1).unwrap(),
        VehicleType::new(vt1, String::from("Vehicle Type 1"), 80, 40, None,)
    );

    assert_eq!(loc0, Location::of(LocationId::from(0)));
    assert_eq!(loc2, Location::of(LocationId::from(2)));
    assert_eq!(loc3, Location::of(LocationId::from(3)));

    assert_eq!(locations.get_external_id_of_location(loc0).unwrap(), "Location 0");
    assert_eq!(locations.get_external_id_of_location(loc2).unwrap(), "Location 2");
    assert_eq!(locations.get_external_id_of_location(loc3).unwrap(), "Location 3");

    assert_eq!(network.all_nodes().count(), 10);

    assert_eq!(
        *network.node(NodeId::start_depot_from(1)),
        Node::create_start_depot_node(
            NodeId::start_depot_from(1),
            DepotId::from(1),
            loc0,
            String::from("Start Depot 1 (at 0)"),
        )
    );
    assert_eq!(
        *network.node(NodeId::end_depot_from(1)),
        Node::create_end_depot_node(
            NodeId::end_depot_from(1),
            DepotId::from(1),
            loc0,
            String::from("End Depot 1 (at 0)"),
        )
    );
    assert_eq!(
        network.capacity_of(DepotId::from(1), VehicleTypeId::from(0)),
        5
    );

    assert_eq!(
        network.capacity_of(DepotId::from(1), VehicleTypeId::from(1)),
        5
    );

    assert_eq!(
        *network.node(NodeId::start_depot_from(2)),
        Node::create_start_depot_node(
            NodeId::start_depot_from(2),
            DepotId::from(2),
            loc2,
            String::from("Start Depot 2 (at 2)"),
        )
    );
    assert_eq!(
        *network.node(NodeId::end_depot_from(2)),
        Node::create_end_depot_node(
            NodeId::end_depot_from(2),
            DepotId::from(2),
            loc2,
            String::from("End Depot 2 (at 2)"),
        )
    );
    assert_eq!(
        network.capacity_of(DepotId::from(2), VehicleTypeId::from(0)),
        0
    );
    assert_eq!(
        network.capacity_of(DepotId::from(2), VehicleTypeId::from(1)),
        500
    );

    assert_eq!(
        *network.node(NodeId::service_from(0, 0)),
        Node::create_service_trip_node(Node::create_service_trip(
            NodeId::service_from(0, 0),
            vt1,
            loc0,
            loc3,
            DateTime::new("2023-07-24T12:00:00"),
            DateTime::new("2023-07-24T12:30:00"),
            Distance::from_meter(600),
            50,
            40,
            String::from("Trip 0-0"),
        ))
    );

    assert_eq!(
        *network.node(NodeId::service_from(0, 1)),
        Node::create_service_trip_node(Node::create_service_trip(
            NodeId::service_from(0, 1),
            vt1,
            loc3,
            loc2,
            DateTime::new("2023-07-24T12:40:00"),
            DateTime::new("2023-07-24T13:10:00"),
            Distance::from_meter(400),
            100,
            90,
            String::from("Trip 0-1"),
        ))
    );

    assert_eq!(
        *network.node(NodeId::service_from(2, 0)),
        Node::create_service_trip_node(Node::create_service_trip(
            NodeId::service_from(2, 0),
            vt0,
            loc2,
            loc3,
            DateTime::new("2023-07-24T14:00:00"),
            DateTime::new("2023-07-24T16:00:00"),
            Distance::from_meter(2000),
            0,
            0,
            String::from("Service Trip 2"),
        ))
    );

    assert_eq!(
        *network.node(NodeId::maintenance_from(0)),
        Node::create_maintenance_node(Node::create_maintenance(
            NodeId::maintenance_from(0),
            loc0,
            DateTime::new("2023-07-24T6:00:00"),
            DateTime::new("2023-07-24T12:00:00"),
            String::from("Maintenance Slot 0"),
        ))
    );

    assert_eq!(
        *network.node(NodeId::maintenance_from(1)),
        Node::create_maintenance_node(Node::create_maintenance(
            NodeId::maintenance_from(1),
            loc0,
            DateTime::new("2023-07-24T12:00:00"),
            DateTime::new("2023-07-24T18:00:00"),
            String::from("Maintenance 1"),
        ))
    );

    assert_travel_time(loc0, loc0, 0, locations);
    assert_travel_time(loc0, loc2, 600, locations);
    assert_travel_time(loc0, loc3, 300, locations);
    assert_travel_time(loc2, loc0, 6000, locations);
    assert_travel_time(loc2, loc2, 0, locations);
    assert_travel_time(loc2, loc3, 400, locations);
    assert_travel_time(loc3, loc0, 3000, locations);
    assert_travel_time(loc3, loc2, 4000, locations);
    assert_travel_time(loc3, loc3, 0, locations);

    assert_travel_distance(loc0, loc0, 0, locations);
    assert_travel_distance(loc0, loc2, 1000, locations);
    assert_travel_distance(loc0, loc3, 500, locations);
    assert_travel_distance(loc2, loc0, 10000, locations);
    assert_travel_distance(loc2, loc2, 0, locations);
    assert_travel_distance(loc2, loc3, 700, locations);
    assert_travel_distance(loc3, loc0, 5000, locations);
    assert_travel_distance(loc3, loc2, 7000, locations);
    assert_travel_distance(loc3, loc3, 0, locations);

    assert!(!config.forbid_dead_head_trip);
    assert_eq!(config.day_limit_threshold, Duration::from_seconds(300));
    assert_eq!(config.shunting.minimal, Duration::from_seconds(120));
    assert_eq!(config.shunting.dead_head_trip, Duration::from_seconds(300));
    assert_eq!(config.shunting.coupling, Duration::from_seconds(600));
    assert_eq!(
        config.maintenance.maximal_distance,
        Distance::from_meter(30000000)
    );
    assert_eq!(config.costs.staff, 100);
    assert_eq!(config.costs.service_trip, 50);
    assert_eq!(config.costs.maintenance, 0);
    assert_eq!(config.costs.dead_head_trip, 500);
    assert_eq!(config.costs.idle, 20);
}

fn assert_travel_time(from: Location, to: Location, expected: u64, locations: &Locations) {
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
