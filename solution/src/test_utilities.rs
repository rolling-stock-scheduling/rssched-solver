use std::{fs::File, io::Read, sync::Arc};

use model::{
    base_types::{DepotIdx, NodeIdx, VehicleTypeIdx},
    json_serialisation::load_rolling_stock_problem_instance_from_json,
    network::Network,
};

pub(crate) struct TestData {
    pub(crate) network: Arc<Network>,
    pub(crate) vt1: VehicleTypeIdx,
    pub(crate) vt2: VehicleTypeIdx,
    pub(crate) depot1: DepotIdx,
    pub(crate) depot2: DepotIdx,
    pub(crate) depot3: DepotIdx,
    // pub(crate) depot4: DepotId,
    // pub(crate) depot5: DepotId,
    pub(crate) trip12: NodeIdx,
    pub(crate) trip23: NodeIdx,
    pub(crate) trip34: NodeIdx,
    pub(crate) trip45: NodeIdx,
    pub(crate) trip45_fast: NodeIdx,
    pub(crate) trip51: NodeIdx,
    pub(crate) trip31: NodeIdx,
    pub(crate) trip14: NodeIdx,
    pub(crate) start_depot1: NodeIdx,
    pub(crate) end_depot1: NodeIdx,
    pub(crate) start_depot2: NodeIdx,
    pub(crate) end_depot2: NodeIdx,
    pub(crate) start_depot3: NodeIdx,
    pub(crate) end_depot3: NodeIdx,
    pub(crate) start_depot4: NodeIdx,
    pub(crate) end_depot4: NodeIdx,
    pub(crate) start_depot5: NodeIdx,
    // pub(crate) end_depot5: NodeId,
}

pub(crate) fn init_test_data() -> TestData {
    // load file from json
    let path = "resources/test_instance.json";

    let mut file = File::open(path).unwrap();
    let mut input_data = String::new();
    file.read_to_string(&mut input_data).unwrap();
    let input_data: serde_json::Value = serde_json::from_str(&input_data).unwrap();
    let network = load_rolling_stock_problem_instance_from_json(input_data);
    TestData {
        network,
        vt1: VehicleTypeIdx::from(1),
        vt2: VehicleTypeIdx::from(2),
        depot1: DepotIdx::from(1),
        depot2: DepotIdx::from(2),
        depot3: DepotIdx::from(3),
        // depot4: DepotId::from(4),
        // depot5: DepotId::from(5),
        trip12: NodeIdx::service_from(12, 0),
        trip23: NodeIdx::service_from(23, 0),
        trip34: NodeIdx::service_from(34, 0),
        trip45: NodeIdx::service_from(45, 0),
        trip45_fast: NodeIdx::service_from(450, 0),
        trip51: NodeIdx::service_from(51, 0),
        trip31: NodeIdx::service_from(31, 0),
        trip14: NodeIdx::service_from(14, 0),
        start_depot1: NodeIdx::start_depot_from(1),
        end_depot1: NodeIdx::end_depot_from(1),
        start_depot2: NodeIdx::start_depot_from(2),
        end_depot2: NodeIdx::end_depot_from(2),
        start_depot3: NodeIdx::start_depot_from(3),
        end_depot3: NodeIdx::end_depot_from(3),
        start_depot4: NodeIdx::start_depot_from(4),
        end_depot4: NodeIdx::end_depot_from(4),
        start_depot5: NodeIdx::start_depot_from(5),
        // end_depot5: NodeIdx::end_depot_from(5),
    }
}
