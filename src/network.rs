pub(crate) mod nodes;
use nodes::Node;

pub(crate) mod demand;
use demand::Demand;

use crate::time::{Time,Duration};
use crate::distance::Distance;
use crate::locations::Locations;
use crate::units::{Units,UnitType};
use crate::base_types::{UnitId,NodeId,StationSide};

use std::collections::HashMap;
use std::fmt;

use std::iter::Iterator;

use std::rc::Rc;

pub(crate) struct Network {
    nodes: HashMap<NodeId, Node>,

    // nodes are by default sorted by start_time
    service_nodes: Vec<NodeId>,
    maintenance_nodes: Vec<NodeId>,
    start_nodes: HashMap<UnitId,NodeId>,
    end_nodes: Vec<NodeId>,
    nodes_sorted_by_start: Vec<NodeId>,
    nodes_sorted_by_end: Vec<NodeId>,

    // for convenience
    loc: Rc<Locations>
}

// methods
impl Network {

    pub(crate) fn node(&self, id: NodeId) -> &Node {
        self.nodes.get(&id).unwrap()
    }

    /// return the number of nodes in the network.
    pub(crate) fn size(&self) -> usize {
        self.nodes.len()
    }

    pub(crate) fn service_nodes(&self) -> impl Iterator<Item=NodeId> + '_ {
        self.service_nodes.iter().copied()
    }

    pub(crate) fn maintenance_nodes(&self) -> impl Iterator<Item=NodeId> + '_ {
        self.maintenance_nodes.iter().copied()
    }

    pub(crate) fn end_nodes(&self) -> impl Iterator<Item=NodeId> + '_ {
        self.end_nodes.iter().copied()
    }

    pub(crate) fn start_node_of(&self, unit_id: UnitId) -> NodeId {
        self.nodes.get(self.start_nodes.get(&unit_id).unwrap()).unwrap().id()
    }

    /// returns True iff node1 can reach node2
    pub(crate) fn can_reach(&self, node1: NodeId, node2: NodeId) -> bool {
        let n1 = self.nodes.get(&node1).unwrap();
        let n2 = self.nodes.get(&node2).unwrap();
        n1.end_time() + self.loc.travel_time(n1.end_location(), n2.start_location()) < n2.start_time()
    }

    /// provides all nodes that are reachable from node (in increasing order according to the
    /// starting time)
    pub(crate) fn all_successors(&self, node: NodeId) -> impl Iterator<Item=NodeId> + '_ {
        // TODO: Could use binary_search for speed up
        self.nodes_sorted_by_start.iter().copied().filter(move |&n| self.can_reach(node,n))
    }

    /// provides all nodes that are can reach node (in decreasing order according to the
    /// end time)
    pub(crate) fn all_predecessors(&self, node: NodeId) -> impl Iterator<Item=NodeId> + '_ {
        // TODO: Could use binary_search for speed up
        self.nodes_sorted_by_end.iter().rev().copied().filter(move |&n| self.can_reach(n, node))
    }

    pub(crate) fn all_nodes(&self) -> impl Iterator<Item=NodeId> + '_ {
        self.nodes_sorted_by_start.iter().copied()
    }

    pub(crate) fn minimal_overhead(&self) -> Duration {
        let earliest_start_time = self.start_nodes.values().map(|n| self.node(*n).end_time()).min().unwrap();
        self.end_nodes.iter().map(|n| self.node(*n).start_time() - earliest_start_time).sum::<Duration>()
         - self.start_nodes.values().map(|n| self.node(*n).end_time() - earliest_start_time).sum()
         - self.total_useful_duration()
    }

    pub(crate) fn total_useful_duration(&self) -> Duration {
        self.service_nodes.iter().chain(self.maintenance_nodes.iter())
            .map(|n| (0..self.node(*n).demand().number_of_units())
            .map(|_| self.node(*n).duration()).sum()).sum()
        // node that service trips are counted as big as their demand is
    }
}

// static functions
impl Network {
    pub(crate) fn load_from_csv(path_service: &str, path_maintenance: &str, path_endpoints: &str, loc: Rc<Locations>, units: Rc<Units>) -> Network {
        let mut nodes: HashMap<NodeId, Node> = HashMap::new();

        let mut service_nodes: Vec<NodeId> = Vec::new();
        let mut reader = csv::ReaderBuilder::new().delimiter(b';').from_path(path_service).expect("csv-file for loading service_trips not found");
        for (i, result) in reader.records().enumerate() {
            let record = result.expect("Some recond cannot be read while reading service_trips");
            let _driving_day = record.get(0).unwrap();
            let _train_number = record.get(1).unwrap();
            let start_time = Time::new(record.get(2).unwrap());
            let start_location = loc.get_location(record.get(3).unwrap());
            let start_side = StationSide::from(record.get(4).unwrap());
            let end_time = Time::new(record.get(5).unwrap());
            let end_location = loc.get_location(record.get(6).unwrap());
            let end_side = StationSide::from(record.get(7).unwrap());
            let length =  Distance::from_km(record.get(8).unwrap().parse().unwrap());
            let demand_amount: u8 = record.get(9).unwrap().parse().unwrap();
            let id = NodeId::from(&format!("ST:{}", record.get(10).unwrap()));
            let name = format!("{}-{}:{}",start_location, end_location,i);

            let service_trip = Node::create_service_node(
                id,
                start_location,
                end_location,
                start_time,
                end_time,
                length,
                Demand::new(demand_amount),
                name
                );
            nodes.insert(id,service_trip);
            service_nodes.push(id);
        }

        let mut maintenance_nodes: Vec<NodeId> = Vec::new();
        let mut reader = csv::ReaderBuilder::new().delimiter(b';').from_path(path_maintenance).expect("csv-file for loading maintenance_slots not found");
        for result in reader.records() {
            let record = result.expect("Some recond cannot be read while reading maintenance_slots");
            let id = NodeId::from(&format!("MS:{}", record.get(0).unwrap()));
            let location = loc.get_location(record.get(1).unwrap());
            let start_time = Time::new(record.get(2).unwrap());
            let end_time = Time::new(record.get(3).unwrap());
            let name = format!("!{}:{}!",location,record.get(0).unwrap());

            let maintenance_slot = Node::create_maintenance_node(
                id,
                location,
                start_time,
                end_time,
                name
                );
            nodes.insert(id,maintenance_slot);
            maintenance_nodes.push(id);
        }

        let mut start_nodes: HashMap<UnitId, NodeId> = HashMap::new();
        for unit_id in units.iter() {
            let unit = units.get_unit(unit_id);
            let node_id = NodeId::from(&format!("SN:{}", unit_id));
            let name = format!("|{}@{}", unit_id, unit.start_location());
            let start_node = Node::create_start_node(node_id, unit_id, unit.start_location(), unit.start_time(),name);
            nodes.insert(node_id,start_node);
            start_nodes.insert(unit_id,node_id);
        }

        let mut end_nodes: Vec<NodeId> = Vec::new();
        let mut reader = csv::ReaderBuilder::new().delimiter(b';').from_path(path_endpoints).expect("csv-file for loading end_points not found");
        for result in reader.records() {
            let record = result.expect("Some recond cannot be read while reading end_points");
            let id = NodeId::from(&format!("EN:{}", record.get(0).unwrap()));
            let unit_type = UnitType::Standard;
            let time = Time::new(record.get(1).unwrap());
            let location = loc.get_location(record.get(2).unwrap());
            let duration_till_maintenance = Duration::from_iso(record.get(3).unwrap());
            let dist_till_maintenance =  Distance::from_km(record.get(4).unwrap().parse().unwrap());
            let name = format!("{}:{}|",location, record.get(0).unwrap());

            let end_point = Node::create_end_node(
                id,
                unit_type,
                location,
                time,
                duration_till_maintenance,
                dist_till_maintenance,
                name
                );
            nodes.insert(id,end_point);
            end_nodes.push(id);
        }

        let mut nodes_sorted_by_start: Vec<NodeId> = nodes.keys().copied().collect();
        nodes_sorted_by_start.sort_by(|n1, n2| nodes.get(n1).unwrap().cmp_start_time(nodes.get(n2).unwrap()));
        let mut nodes_sorted_by_end: Vec<NodeId> = nodes.keys().copied().collect();
        nodes_sorted_by_end.sort_by(|n1, n2| nodes.get(n1).unwrap().cmp_end_time(nodes.get(n2).unwrap()));

        // sort all indices by the start_time
        service_nodes.sort_by(|n1, n2| nodes.get(n1).unwrap().cmp_start_time(nodes.get(n2).unwrap()));
        maintenance_nodes.sort_by(|n1, n2| nodes.get(n1).unwrap().cmp_start_time(nodes.get(n2).unwrap()));
        end_nodes.sort_by(|n1, n2| nodes.get(n1).unwrap().cmp_start_time(nodes.get(n2).unwrap()));

        Network{nodes,service_nodes,maintenance_nodes,start_nodes,end_nodes,nodes_sorted_by_start, nodes_sorted_by_end,loc}
    }
}



impl fmt::Display for Network {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f,"** network with {} nodes:", self.size())?;
        for (i,n) in self.nodes_sorted_by_start.iter().enumerate() {
            writeln!(f,"\t{}: {}", i, self.nodes.get(n).unwrap())?;
        }
        Ok(())
    }
}
