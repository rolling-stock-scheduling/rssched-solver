mod tour;
use tour::Tour;

mod train_formation;
use train_formation::TrainFormation;

use crate::network::Network;
use crate::network::nodes::Node;
use crate::units::Units;
use crate::locations::Locations;
use crate::base_types::{NodeId,UnitId};
use crate::objective::Objective;

use std::collections::{HashMap,HashSet,VecDeque};

use std::fmt;

pub(crate) struct Schedule<'a> {
    tours: HashMap<UnitId, Tour<'a>>,
    covered_by: HashMap<NodeId, TrainFormation>,

    // non-covered or only partially covered service nodes are stored seperately for quick objective_value conputation
    uncovered_service: HashSet<NodeId>,
    uncovered_maintenance: HashSet<NodeId>,
    // maybe add uncovered_maintenance nodes as well, later on
    objective_value: Option<Objective>,

    loc: &'a Locations,
    units: &'a Units,
    nw: &'a Network<'a>,
}



// methods
impl<'a> Schedule <'a> {
    pub(crate) fn get_tour_of(&self, unit: UnitId) -> &Tour {
        self.tours.get(&unit).unwrap()
    }

    pub(crate) fn assign(&mut self, unit: UnitId, node_sequence: Vec<NodeId>) {
        let tour = self.tours.get_mut(&unit).unwrap();

        for node in node_sequence.iter() {
            self.covered_by.get_mut(node).unwrap().add(unit);

            // TODO: remove from uncovered-list if needed
        }

        let removed_nodes = tour.insert(node_sequence);
        for node in removed_nodes.iter() {
            self.covered_by.get_mut(node).unwrap().remove(unit);

            // TODO: Add to uncovered-lists if needed
            // if !self.is_satisfied(*node) {
                // match self.nw.node(*node) {
                    // Node::Service(_) => {self.uncovered_service.insert(*node);}
                    // Node::Maintenance(_) => {self.uncovered_maintenance.insert(*node);}
                    // _ => {panic!("Uncovered point is neither service nor maintenance");}
                // }
            // }

        }
    }

    fn is_satisfied(&self, node: NodeId) -> bool {
        // TODO: check whether the demand of the node fits to the TrainFormation assigned to this
        // node
        true
    }

    pub(crate) fn print(&self) {
        println!("** schedule with {} tours:", self.tours.len());
        for (unit, tour) in self.tours.iter() {
            print!("\ttour of {}:", unit);
            tour.print();
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

// static functions
impl<'a> Schedule<'a> {
    pub(crate) fn initialize(loc: &'a Locations, units: &'a Units, nw: &'a Network) -> Schedule<'a> {

        let mut tours: HashMap<UnitId, Tour> = HashMap::with_capacity(units.len());
        let mut covered_by: HashMap<NodeId, TrainFormation> = HashMap::new();

        // create trivial tours from start_point directly to end point
        // end_ponints are picked greedily from earliest to latest (unit_type must fit)
        let mut end_nodes: VecDeque<NodeId> = nw.end_nodes_ids().collect();
        end_nodes.make_contiguous().sort_by(|&e1,&e2| nw.node(e1).start_time().cmp(&nw.node(e2).start_time()));

        for unit in units.iter() {
            let unit_id = unit.get_id();
            let start_node = nw.start_node_id_of(unit_id);
            let pos = end_nodes.iter().position(|&e| nw.node(e).unit_type() == unit.get_type() && nw.can_reach(start_node, e)).expect(format!("No suitable end_node found for start_node: {}", start_node).as_str());
            let end_node = end_nodes.remove(pos).unwrap();

            tours.insert(unit_id, Tour::new(unit_id, vec!(start_node, end_node),loc,nw));

            covered_by.insert(start_node, TrainFormation::new(vec!(unit_id)));
            covered_by.insert(end_node, TrainFormation::new(vec!(unit_id)));
        }

        for node in nw.service_nodes_ids() {
            covered_by.insert(node, TrainFormation::new(vec!()));
        }
        for node in nw.maintenance_nodes_ids() {
            covered_by.insert(node, TrainFormation::new(vec!()));
        }

        assert!(end_nodes.is_empty(), "There are more end_nodes than units!");
        let mut uncovered_service: HashSet<NodeId> = HashSet::new();
        uncovered_service.extend(nw.service_nodes_ids());
        let mut uncovered_maintenance: HashSet<NodeId> = HashSet::new();
        uncovered_maintenance.extend(nw.maintenance_nodes_ids());


        Schedule{tours, covered_by, uncovered_service, uncovered_maintenance, objective_value:None, loc, units, nw}
    }
}
