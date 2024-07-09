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

use std::{fs::File, io::Read};

use rapid_time::{DateTime, Duration};

use crate::{
    base_types::{DepotIdx, Distance, Location, LocationIdx, NodeIdx, VehicleTypeIdx},
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
    let loc0 = locations.get(LocationIdx::from(0)).unwrap();
    let loc2 = locations.get(LocationIdx::from(1)).unwrap();
    let loc3 = locations.get(LocationIdx::from(2)).unwrap();

    let vt0 = VehicleTypeIdx::from(0);
    let vt1 = VehicleTypeIdx::from(1);

    assert_eq!(
        *vehicle_types.get(vt0).unwrap(),
        VehicleType::new(vt0, String::from("IC"), 100, 50, Some(4),)
    );
    assert_eq!(
        *vehicle_types.get(vt1).unwrap(),
        VehicleType::new(vt1, String::from("IR"), 80, 40, None,)
    );

    assert_eq!(loc0, Location::of(LocationIdx::from(0)));
    assert_eq!(loc2, Location::of(LocationIdx::from(1)));
    assert_eq!(loc3, Location::of(LocationIdx::from(2)));

    assert_eq!(locations.get_id(loc0).unwrap(), "ZH");
    assert_eq!(locations.get_id(loc2).unwrap(), "BN");
    assert_eq!(locations.get_id(loc3).unwrap(), "LU");

    assert_eq!(network.all_nodes().count(), 12);

    network.all_nodes().for_each(|node| {
        println!("{:?}", node);
    });

    assert_eq!(
        *network.node(NodeIdx::start_depot_from(0)),
        Node::create_start_depot_node(0, String::from("s_depot_ZH"), DepotIdx::from(0), loc0,)
    );
    assert_eq!(
        *network.node(NodeIdx::end_depot_from(1)),
        Node::create_end_depot_node(1, String::from("e_depot_ZH"), DepotIdx::from(0), loc0,)
    );
    assert_eq!(
        network.capacity_of(DepotIdx::from(0), VehicleTypeIdx::from(0)),
        5
    );

    assert_eq!(
        network.capacity_of(DepotIdx::from(0), VehicleTypeIdx::from(1)),
        5
    );

    assert_eq!(
        *network.node(NodeIdx::start_depot_from(2)),
        Node::create_start_depot_node(2, String::from("s_depot_BN"), DepotIdx::from(1), loc2,)
    );
    assert_eq!(
        *network.node(NodeIdx::end_depot_from(3)),
        Node::create_end_depot_node(3, String::from("e_depot_BN"), DepotIdx::from(1), loc2,)
    );
    assert_eq!(
        network.capacity_of(DepotIdx::from(1), VehicleTypeIdx::from(0)),
        500
    );
    assert_eq!(
        network.capacity_of(DepotIdx::from(1), VehicleTypeIdx::from(1)),
        0
    );

    assert_eq!(
        *network.node(NodeIdx::service_from(6)),
        Node::create_service_trip_node(
            6,
            Node::create_service_trip(
                String::from("trip_1a_seg_0"),
                vt0,
                loc2,
                loc3,
                DateTime::new("2023-07-24T12:00:00"),
                DateTime::new("2023-07-24T14:00:00"),
                Distance::from_meter(2000),
                80,
                80,
                None
            )
        )
    );

    assert_eq!(
        *network.node(NodeIdx::service_from(7)),
        Node::create_service_trip_node(
            7,
            Node::create_service_trip(
                String::from("trip_1b_seg_0"),
                vt0,
                loc2,
                loc3,
                DateTime::new("2023-07-24T14:00:00"),
                DateTime::new("2023-07-24T16:00:00"),
                Distance::from_meter(2000),
                1,
                0,
                None
            )
        )
    );

    assert_eq!(
        *network.node(NodeIdx::service_from(8)),
        Node::create_service_trip_node(
            8,
            Node::create_service_trip(
                String::from("trip_0_seg_0"),
                vt1,
                loc0,
                loc3,
                DateTime::new("2023-07-24T12:00:00"),
                DateTime::new("2023-07-24T12:30:00"),
                Distance::from_meter(600),
                50,
                40,
                Some(1)
            )
        )
    );

    assert_eq!(
        *network.node(NodeIdx::maintenance_from(10)),
        Node::create_maintenance_node(
            10,
            Node::create_maintenance(
                String::from("maintenance_slot_0"),
                loc0,
                DateTime::new("2023-07-24T6:00:00"),
                DateTime::new("2023-07-24T12:00:00"),
                2
            )
        )
    );

    assert_eq!(
        *network.node(NodeIdx::maintenance_from(11)),
        Node::create_maintenance_node(
            11,
            Node::create_maintenance(
                String::from("maintenance_slot_1"),
                loc2,
                DateTime::new("2023-07-24T12:00:00"),
                DateTime::new("2023-07-24T18:00:00"),
                1
            )
        )
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
