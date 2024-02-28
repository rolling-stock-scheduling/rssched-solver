use std::{fs::File, io::Read, sync::Arc};

use model::{
    base_types::{DepotId, NodeId, VehicleTypeId},
    config::Config,
    json_serialisation::load_rolling_stock_problem_instance_from_json,
    network::Network,
    vehicle_types::VehicleTypes,
};

pub(crate) struct TestData {
    pub(crate) vehicle_types: Arc<VehicleTypes>,
    pub(crate) network: Arc<Network>,
    pub(crate) config: Arc<Config>,
    pub(crate) vt1: VehicleTypeId,
    pub(crate) vt2: VehicleTypeId,
    pub(crate) depot1: DepotId,
    pub(crate) depot2: DepotId,
    pub(crate) depot3: DepotId,
    // pub(crate) depot4: DepotId,
    // pub(crate) depot5: DepotId,
    pub(crate) trip12: NodeId,
    pub(crate) trip23: NodeId,
    pub(crate) trip34: NodeId,
    pub(crate) trip45: NodeId,
    pub(crate) trip45_fast: NodeId,
    pub(crate) trip51: NodeId,
    pub(crate) trip31: NodeId,
    pub(crate) trip14: NodeId,
    pub(crate) start_depot1: NodeId,
    pub(crate) end_depot1: NodeId,
    pub(crate) start_depot2: NodeId,
    pub(crate) end_depot2: NodeId,
    pub(crate) start_depot3: NodeId,
    pub(crate) end_depot3: NodeId,
    pub(crate) start_depot4: NodeId,
    pub(crate) end_depot4: NodeId,
    pub(crate) start_depot5: NodeId,
    // pub(crate) end_depot5: NodeId,
}

pub(crate) fn init_test_data() -> TestData {
    // load file from json
    let path = "resources/test_instance.json";

    let mut file = File::open(path).unwrap();
    let mut input_data = String::new();
    file.read_to_string(&mut input_data).unwrap();
    let input_data: serde_json::Value = serde_json::from_str(&input_data).unwrap();
    let (vehicle_types, network, config) =
        load_rolling_stock_problem_instance_from_json(input_data);
    TestData {
        vehicle_types,
        network,
        config,
        vt1: VehicleTypeId::from(1),
        vt2: VehicleTypeId::from(2),
        depot1: DepotId::from(1),
        depot2: DepotId::from(2),
        depot3: DepotId::from(3),
        // depot4: DepotId::from(4),
        // depot5: DepotId::from(5),
        trip12: NodeId::service_from(12, 0),
        trip23: NodeId::service_from(23, 0),
        trip34: NodeId::service_from(34, 0),
        trip45: NodeId::service_from(45, 0),
        trip45_fast: NodeId::service_from(450, 0),
        trip51: NodeId::service_from(51, 0),
        trip31: NodeId::service_from(31, 0),
        trip14: NodeId::service_from(14, 0),
        start_depot1: NodeId::start_depot_from(1),
        end_depot1: NodeId::end_depot_from(1),
        start_depot2: NodeId::start_depot_from(2),
        end_depot2: NodeId::end_depot_from(2),
        start_depot3: NodeId::start_depot_from(3),
        end_depot3: NodeId::end_depot_from(3),
        start_depot4: NodeId::start_depot_from(4),
        end_depot4: NodeId::end_depot_from(4),
        start_depot5: NodeId::start_depot_from(5),
        // end_depot5: NodeId::end_depot_from(5),
    }
}
