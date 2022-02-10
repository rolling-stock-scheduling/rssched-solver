mod tour;
use tour::Tour;

use crate::network::Network;
use crate::network::nodes::Node;
use crate::units::Units;
use crate::locations::Locations;
use crate::base_types::{NodeId,UnitId};
use crate::objective::Objective;

use std::collections::{HashMap,HashSet,VecDeque};

use std::fmt;

pub(crate) struct Schedule<'a> {
    tours: HashMap<UnitId, Tour>,
    locations: &'a Locations,
    units: &'a Units,
    network: &'a Network<'a>,
    // covered_by: HashMap<NodeId, TrainFormation>

    // non-covered or only partially covered service nodes are stored seperately for quick objective_value conputation
    uncovered_service: HashSet<NodeId>,
    // maybe add uncovered_maintenance nodes as well, later on
    objective_value: Option<Objective>,
}


impl<'a> Schedule<'a> {
    pub(crate) fn initialize(locations: &'a Locations, units: &'a Units, network: &'a Network) -> Schedule<'a> {

        let mut tours : HashMap<UnitId, Tour> = HashMap::with_capacity(units.len());

        // create trivial tours from start_point directly to end point
        // end_ponints are picked greedily from earliest to latest (unit_type must fit)
        let mut end_nodes: VecDeque<&Node> = network.end_nodes_iter().collect();
        end_nodes.make_contiguous().sort_by(|e1,e2| e1.start_time().cmp(&e2.start_time()));

        for unit in units.iter() {
            let unit_id = unit.get_id();
            let start_node = network.start_node_of(unit_id);
            let pos = end_nodes.iter().position(|e| e.unit_type() == unit.get_type() && network.can_reach(start_node, e)).expect(format!("No suitable end_node found for start_node: {}", start_node).as_str());
            let end_node = end_nodes.remove(pos).unwrap();

            tours.insert(unit_id, Tour::new(unit_id, vec!(start_node.id(), end_node.id())));
        }

        assert!(end_nodes.is_empty(), "There are more end_nodes than units!");
        let mut uncovered_service: HashSet<NodeId> = HashSet::new();
        uncovered_service.extend(network.service_nodes_iter().map(|s| s.id()));

        Schedule{tours, locations, units, network, uncovered_service, objective_value:None}
    }

    pub(crate) fn print(&self) {
        println!("** schedule with {} tours:", self.tours.len());
        for (unit, tour) in self.tours.iter() {
            print!("\ttour of {}:", unit);
            tour.print(self.locations, self.network);
        }
    }
}


impl<'a> fmt::Display for Schedule<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "** schedule with {} tours:", self.tours.len())?;
        for tour in self.tours.values() {
            writeln!(f, "\t{}", tour)?;
        }
        Ok(())
    }
}
