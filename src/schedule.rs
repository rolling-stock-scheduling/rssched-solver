mod tour;
use tour::Tour;

use crate::train_formation::TrainFormation;

use crate::network::Network;
use crate::units::Units;
use crate::locations::Locations;
use crate::distance::Distance;
use crate::base_types::{NodeId,UnitId,Penalty};
use crate::objective::Objective;

use std::collections::{HashMap,HashSet,VecDeque};

use std::fmt;
use crate::base_types::PENALTY_ZERO;

pub(crate) struct Schedule<'a> {
    tours: HashMap<UnitId, Tour<'a>>,
    covered_by: HashMap<NodeId, TrainFormation<'a>>,

    // non-covered or only partially covered service nodes are stored seperately for quick objective_value conputation
    uncovered_nodes: HashSet<NodeId>,
    objective_value: Option<Objective>,

    loc: &'a Locations,
    units: &'a Units,
    nw: &'a Network<'a>,
}



// methods
impl<'a> Schedule <'a> {
    pub(crate) fn tour_of(&self, unit: UnitId) -> &Tour {
        self.tours.get(&unit).unwrap()
    }

    pub(crate) fn assign(&mut self, unit: UnitId, node_sequence: Vec<NodeId>) -> Result<(),String> {
        let tour = self.tours.get_mut(&unit).unwrap();
        let new_nodes = node_sequence.clone();

        // insert sequence into tour
        let removed_nodes = tour.insert(node_sequence)?;

        // update covered_by:
        for node in removed_nodes.iter() {
            self.covered_by.get_mut(node).unwrap().remove(unit);
        }
        for node in new_nodes.iter() {
            self.covered_by.get_mut(node).unwrap().add(unit);
        }

        // update uncovered_nodes:
        for node in removed_nodes.iter() {
            if self.cover_penalty_of(*node) != PENALTY_ZERO {
                self.uncovered_nodes.insert(*node);
            }
        }
        for node in new_nodes.iter() {
            if self.cover_penalty_of(*node) == PENALTY_ZERO {
                self.uncovered_nodes.remove(node);
            }
        }
        Ok(())
    }

    /// simulates inserting the node_sequence into the tour of unit. Return all nodes that would
    /// have been removed from the tour.
    pub(crate) fn assign_test(&self, unit: UnitId, node_sequence: Vec<NodeId>) -> Result<Vec<NodeId>,String> {
        let mut tour: Tour = self.tours.get(&unit).unwrap().clone();
        tour.insert(node_sequence)

    }

    fn cover_penalty_of(&self, node: NodeId) -> Penalty {
        self.nw.node(node).cover_penalty(self.covered_by.get(&node).unwrap())
    }

    pub(crate) fn total_cover_penalty(&self) -> Penalty {
        self.uncovered_nodes.iter().map(|&n| self.cover_penalty_of(n)).sum()
    }

    pub(crate) fn total_distance(&self) -> Distance {
        self.tours.values().map(|t| t.distance()).sum()
    }

    pub(crate) fn uncovered_nodes(&self) -> Vec<NodeId> {
        let mut list: Vec<NodeId> = self.uncovered_nodes.iter().cloned().collect();
        list.sort_by(|&n1,&n2| self.nw.node(n1).cmp_start_time(self.nw.node(n2)));
        list
    }

    pub(crate) fn has_uncovered_nodes(&self) -> bool {
        self.uncovered_nodes.len() > 0
    }

    pub(crate) fn uncovered_successors(&self, node: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        self.nw.all_successors(node).filter(|n| self.uncovered_nodes.contains(n))
    }

    pub(crate) fn print(&self) {
        println!("** schedule with {} tours:", self.tours.len());
        for unit in self.units.get_all() {
            print!("\ttour of {}:", unit);
            self.tours.get(&unit).unwrap().print();
        }
    }
}


impl<'a> fmt::Display for Schedule<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "** schedule with {} tours:", self.tours.len())?;
        for unit in self.units.get_all() {
            writeln!(f, "\t{}", self.tours.get(&unit).unwrap())?;
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
        let mut end_nodes: VecDeque<NodeId> = nw.end_nodes().collect();
        end_nodes.make_contiguous().sort_by(|&e1,&e2| nw.node(e1).start_time().cmp(&nw.node(e2).start_time()));

        for unit_id in units.get_all() {
            let unit = units.get_unit(unit_id);
            let start_node = nw.start_node_of(unit_id);
            let pos = end_nodes.iter().position(|&e| nw.node(e).unit_type() == unit.unit_type() && nw.can_reach(start_node, e)).expect(format!("No suitable end_node found for start_node: {}", start_node).as_str());
            let end_node = end_nodes.remove(pos).unwrap();

            tours.insert(unit_id, Tour::new(unit_id, vec!(start_node, end_node),loc,nw));

            covered_by.insert(start_node, TrainFormation::new(vec!(unit_id), units));
            covered_by.insert(end_node, TrainFormation::new(vec!(unit_id), units));
        }

        for node in nw.service_nodes() {
            covered_by.insert(node, TrainFormation::new(vec!(), units));
        }
        for node in nw.maintenance_nodes() {
            covered_by.insert(node, TrainFormation::new(vec!(), units));
        }

        assert!(end_nodes.is_empty(), "There are more end_nodes than units!");
        let mut uncovered_nodes: HashSet<NodeId> = HashSet::new();
        uncovered_nodes.extend(nw.service_nodes());
        uncovered_nodes.extend(nw.maintenance_nodes());


        Schedule{tours, covered_by, uncovered_nodes, objective_value:None, loc, units, nw}
    }
}
