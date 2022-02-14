pub(crate) mod nodes;
use nodes::Node;


use crate::time::{Time,Duration};
use crate::distance::Distance;
use crate::locations::Locations;
use crate::units::{Units,UnitType};
use crate::base_types::{UnitId,NodeId,StationSide};

use std::collections::HashMap;
use std::fmt;

use std::iter::Iterator;

pub(crate) struct Network<'a> {
    nodes: HashMap<NodeId, Node>,
    service_nodes: Vec<NodeId>,
    maintenance_nodes: Vec<NodeId>,
    start_nodes: HashMap<UnitId,NodeId>,
    end_nodes: Vec<NodeId>,
    locations: &'a Locations
}

// static functions
impl<'a> Network<'a> {
    pub(crate) fn initialize(locations: &'a Locations, units: &Units, path_service: &str, path_maintenance: &str, path_endpoints: &str) -> Network<'a> {
        let mut nodes: HashMap<NodeId, Node> = HashMap::new();

        let mut service_nodes: Vec<NodeId> = Vec::new();
        let mut reader = csv::ReaderBuilder::new().delimiter(b';').from_path(path_service).expect("csv-file for loading service_trips not found");
        for result in reader.records() {
            let record = result.expect("Some recond cannot be read while reading service_trips");
            let _driving_day = record.get(0).unwrap();
            let _train_number = record.get(1).unwrap();
            let start_time = Time::new(record.get(2).unwrap());
            let start_location = locations.get_location(record.get(3).unwrap());
            let start_side = StationSide::from(record.get(4).unwrap());
            let end_time = Time::new(record.get(5).unwrap());
            let end_location = locations.get_location(record.get(6).unwrap());
            let end_side = StationSide::from(record.get(7).unwrap());
            let length =  Distance::from_km(record.get(8).unwrap().parse().unwrap());
            let _demand: u8 = record.get(9).unwrap().parse().unwrap();
            let id_string = &format!("ST:{}",(record.get(10).unwrap()));
            let id = NodeId::from(&id_string);

            let service_trip = Node::create_service_node(
                id,
                start_location,
                end_location,
                start_time,
                end_time,
                length,
                );
            nodes.insert(id,service_trip);
            service_nodes.push(id);
        }

        let mut maintenance_nodes: Vec<NodeId> = Vec::new();
        let mut reader = csv::ReaderBuilder::new().delimiter(b';').from_path(path_maintenance).expect("csv-file for loading maintenance_slots not found");
        for result in reader.records() {
            let record = result.expect("Some recond cannot be read while reading maintenance_slots");
            let id_string = format!("MS:{}",record.get(0).unwrap());
            let id = NodeId::from(&id_string);
            let location = locations.get_location(record.get(1).unwrap());
            let start_time = Time::new(record.get(2).unwrap());
            let end_time = Time::new(record.get(3).unwrap());

            let maintenance_slot = Node::create_maintenance_node(
                id,
                location,
                start_time,
                end_time,
                );
            nodes.insert(id,maintenance_slot);
            maintenance_nodes.push(id);
        }

        let mut start_nodes: HashMap<UnitId, NodeId> = HashMap::new();
        for unit in units.iter() {
            let unit_id = unit.get_id();
            let node_id_string = format!("SN:{}", unit_id);
            let node_id = NodeId::from(&node_id_string);
            let start_node = Node::create_start_node(node_id, unit_id, unit.get_start_location(), unit.get_start_time());
            nodes.insert(node_id,start_node);
            start_nodes.insert(unit_id,node_id);
        }

        let mut end_nodes: Vec<NodeId> = Vec::new();
        let mut reader = csv::ReaderBuilder::new().delimiter(b';').from_path(path_endpoints).expect("csv-file for loading end_points not found");
        for result in reader.records() {
            let record = result.expect("Some recond cannot be read while reading end_points");
            let id_string = format!("EP:{}",record.get(0).unwrap());
            let id = NodeId::from(&id_string);
            let unit_type = UnitType::Standard;
            let time = Time::new(record.get(1).unwrap());
            let location = locations.get_location(record.get(2).unwrap());
            let duration_till_maintenance = Duration::from_iso(record.get(3).unwrap());
            let dist_till_maintenance =  Distance::from_km(record.get(4).unwrap().parse().unwrap());

            let end_point = Node::create_end_node(
                id,
                unit_type,
                location,
                time,
                duration_till_maintenance,
                dist_till_maintenance
                );
            nodes.insert(id,end_point);
            end_nodes.push(id);
        }

        Network{nodes,service_nodes,maintenance_nodes,start_nodes,end_nodes,locations}
    }
}

// methods
impl<'a> Network<'a> {

    pub(crate) fn node(&self, id: NodeId) -> &Node {
        self.nodes.get(&id).unwrap()
    }

    /// return the number of nodes in the network.
    pub(crate) fn size(&self) -> usize {
        self.nodes.len()
    }

    pub(crate) fn service_nodes_ids(&self) -> impl Iterator<Item=NodeId> + '_ {
        self.service_nodes.iter().map(|&n| n)
    }

    pub(crate) fn maintenance_nodes_ids(&self) -> impl Iterator<Item=NodeId> + '_ {
        self.maintenance_nodes.iter().map(|&n| n)

    }

    pub(crate) fn start_nodes_ids(&self) -> impl Iterator<Item=NodeId> + '_ {
        self.start_nodes.values().map(|&n| n)

    }

    pub(crate) fn end_nodes_ids(&self) -> impl Iterator<Item=NodeId> + '_ {
        self.end_nodes.iter().map(|&n| n)

    }

    pub(crate) fn start_node_id_of(&self, unit_id: UnitId) -> NodeId {
        self.nodes.get(self.start_nodes.get(&unit_id).unwrap()).unwrap().id()
    }

    /// returns True iff node1 can reach node2
    pub(crate) fn can_reach(&self, node1: NodeId, node2: NodeId) -> bool {
        let n1 = self.nodes.get(&node1).unwrap();
        let n2 = self.nodes.get(&node2).unwrap();
        n1.end_time() + self.locations.travel_time(n1.end_location(), n2.start_location()) < n2.start_time()
    }

    pub(crate) fn all_successors(&self, node: NodeId) -> impl Iterator<Item=NodeId> + '_ {
        self.all_nodes_ids().filter(move |&n| self.can_reach(node,n))
        // self.all_nodes_iter().filter(|&other| self.can_reach(node, other)).map(|&n| n)
    }

    pub(crate) fn all_predecessors(&self, node: NodeId) -> impl Iterator<Item=NodeId> + '_ {
        self.all_nodes_ids().filter(move |&n| self.can_reach(n, node))
    }

    pub(crate) fn all_nodes_ids(&self) -> impl Iterator<Item=NodeId> + '_ {
        self.nodes.values().map(|n| n.id())
    }
}

impl<'a> fmt::Display for Network<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"** network with {} nodes:\n", self.size())?;
        for (i,v) in self.nodes.values().enumerate() {
            write!(f,"\t{}: {}\n", i, v)?;
        }
        Ok(())
    }
}
