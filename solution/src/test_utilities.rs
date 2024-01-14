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
        vt1: VehicleTypeId::from("vt1"),
        vt2: VehicleTypeId::from("vt2"),
        depot1: DepotId::from("depot1"),
        depot2: DepotId::from("depot2"),
        depot3: DepotId::from("depot3"),
        // depot4: DepotId::from("depot4"),
        // depot5: DepotId::from("depot5"),
        trip12: NodeId::from("trip1-2"),
        trip23: NodeId::from("trip2-3"),
        trip34: NodeId::from("trip3-4"),
        trip45: NodeId::from("trip4-5"),
        trip45_fast: NodeId::from("trip4-5_fast"),
        trip51: NodeId::from("trip5-1"),
        trip31: NodeId::from("trip3-1"),
        trip14: NodeId::from("trip1-4"),
        start_depot1: NodeId::from("s_depot1"),
        end_depot1: NodeId::from("e_depot1"),
        start_depot2: NodeId::from("s_depot2"),
        end_depot2: NodeId::from("e_depot2"),
        start_depot3: NodeId::from("s_depot3"),
        end_depot3: NodeId::from("e_depot3"),
        start_depot4: NodeId::from("s_depot4"),
        end_depot4: NodeId::from("e_depot4"),
        start_depot5: NodeId::from("s_depot5"),
        // end_depot5: NodeId::from("e_depot5"),
    }
}
