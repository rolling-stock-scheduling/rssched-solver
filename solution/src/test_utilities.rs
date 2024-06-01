use std::{fs::File, io::Read, sync::Arc};

use model::{
    base_types::{DepotIdx, NodeIdx, VehicleTypeIdx},
    json_serialisation::load_rolling_stock_problem_instance_from_json,
    network::Network,
};

use crate::Schedule;

pub struct TestData {
    pub network: Arc<Network>,
    pub vt1: VehicleTypeIdx,
    pub vt2: VehicleTypeIdx,
    pub depot1: DepotIdx,
    pub depot2: DepotIdx,
    pub depot3: DepotIdx,
    pub trip12: NodeIdx,
    pub trip23: NodeIdx,
    pub trip34: NodeIdx,
    pub trip45: NodeIdx,
    pub trip45_fast: NodeIdx,
    pub trip51: NodeIdx,
    pub trip31: NodeIdx,
    pub trip14: NodeIdx,
    pub start_depot1: NodeIdx,
    pub end_depot1: NodeIdx,
    pub start_depot2: NodeIdx,
    pub end_depot2: NodeIdx,
    pub start_depot3: NodeIdx,
    pub end_depot3: NodeIdx,
    pub start_depot4: NodeIdx,
    pub end_depot4: NodeIdx,
    pub start_depot5: NodeIdx,
    pub end_depot5: NodeIdx,
    pub start_overflow_depot: NodeIdx,
    pub end_overflow_depot: NodeIdx,
}

pub fn init_test_data() -> TestData {
    // load file from json
    let path = "resources/test_instance.json";

    let mut file = File::open(path).unwrap();
    let mut input_data = String::new();
    file.read_to_string(&mut input_data).unwrap();
    let input_data: serde_json::Value = serde_json::from_str(&input_data).unwrap();
    let network = load_rolling_stock_problem_instance_from_json(input_data);
    TestData {
        network,
        vt1: VehicleTypeIdx::from(0),
        vt2: VehicleTypeIdx::from(1),
        start_depot1: NodeIdx::start_depot_from(0),
        end_depot1: NodeIdx::end_depot_from(1),
        start_depot2: NodeIdx::start_depot_from(2),
        end_depot2: NodeIdx::end_depot_from(3),
        start_depot3: NodeIdx::start_depot_from(4),
        end_depot3: NodeIdx::end_depot_from(5),
        start_depot4: NodeIdx::start_depot_from(6),
        end_depot4: NodeIdx::end_depot_from(7),
        start_depot5: NodeIdx::start_depot_from(8),
        end_depot5: NodeIdx::end_depot_from(9),
        start_overflow_depot: NodeIdx::start_depot_from(10),
        end_overflow_depot: NodeIdx::end_depot_from(11),
        trip12: NodeIdx::service_from(12),
        trip23: NodeIdx::service_from(13),
        trip34: NodeIdx::service_from(14),
        trip45: NodeIdx::service_from(15),
        trip45_fast: NodeIdx::service_from(16),
        trip51: NodeIdx::service_from(17),
        trip31: NodeIdx::service_from(18),
        trip14: NodeIdx::service_from(19),
        depot1: DepotIdx::from(0),
        depot2: DepotIdx::from(1),
        depot3: DepotIdx::from(2),
        // depot4: DepotIdx::from(3),
        // depot5: DepotIdx::from(4),
    }
}

pub fn default_schedule(d: &TestData) -> Schedule {
    let mut schedule = Schedule::empty(d.network.clone());

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
            d.vt1,
            vec![d.start_depot2, d.trip31, d.trip14, d.end_depot1],
        )
        .unwrap()
        .0;

    // veh00002
    schedule = schedule
        .spawn_vehicle_for_path(
            d.vt1,
            vec![d.start_depot1, d.trip12, d.trip23, d.trip31, d.end_depot2],
        )
        .unwrap()
        .0;

    schedule
}
