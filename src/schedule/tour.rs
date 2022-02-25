use std::fmt;
use crate::distance::Distance;
use crate::time::{Time, Duration};
use crate::locations::Locations;
use crate::network::Network;
use crate::network::nodes::Node;
use crate::base_types::{NodeId,UnitId};
use crate::schedule::path::Path;

use itertools::Itertools;

use std::rc::Rc;

type Position = usize; // the position within the tour from 0 to nodes.len()-1

/// this represents a tour of a single unit. It always start at the StartNode of the unit and ends
/// with an EndNode that has the type of the unit.
/// It should be an immutable objects. So whenever some modification is applied a copy of the tour
/// is created.
#[derive(Clone)]
pub(crate) struct Tour {
    unit: UnitId,
    nodes: Vec<NodeId>, // nodes will always be sorted by start_time
    loc: Rc<Locations>,
    nw: Rc<Network>,
}

impl Tour {
    pub(crate) fn nodes_iter(&self) -> impl Iterator<Item=&NodeId> +'_{
        self.nodes.iter()
    }

    pub(crate) fn last_node(&self) -> NodeId {
        *self.nodes.last().unwrap()
    }

    pub(crate) fn distance(&self) -> Distance {
        let service_length: Distance = self.nodes.iter().map(|&n| self.nw.node(n).length()).sum();

        let dead_head_length = self.nodes.iter().tuple_windows().map(
            |(&a,&b)| self.loc.distance(self.nw.node(a).end_location(),self.nw.node(b).start_location())).sum();
        service_length + dead_head_length
    }

    pub(crate) fn travel_time(&self) -> Duration {
        let service_tt: Duration = self.nodes.iter().map(|&n| self.nw.node(n).travel_time()).sum();
        let dead_head_tt = self.nodes.iter().tuple_windows().map(
            |(&a,&b)| self.loc.travel_time(self.nw.node(a).end_location(), self.nw.node(b).start_location())).sum();
        service_tt + dead_head_tt
    }

    /// inserts the provided node sequence on the correct position (time-wise). The sequence will
    /// stay uninterrupted. All removed nodes (due to time-clashes) are returned.
    /// Assumes that provided node sequence is feasible.
    /// Panics if sequence is not reachable from the start node, and if end_node cannot be reached,
    /// sequence must itself end with a end_node
    pub(super) fn insert(&self, path: Path) -> Result<(Tour,Path),String> {
        let first = path.first();
        let last = path.last();

        let start_pos = self.latest_node_reaching(first).ok_or_else(|| format!("Unit, cannot reach node"))?;
        let end_pos = self.earliest_node_reached_by(last).ok_or_else(|| format!("Cannot insert sequence to path of unit {}, as the end_point cannot be reached!", self.unit))?;

        let mut new_tour_nodes = self.nodes.clone();
        // remove all elements strictly between start_pos and end_pos and replace them by
        // node_sequence. Removed nodes are returned.
        let removed_nodes = new_tour_nodes.splice(start_pos+1..end_pos,path.consume()).collect();
        Ok((Tour::new_trusted(self.unit, new_tour_nodes,self.loc.clone(),self.nw.clone()),Path::new_trusted(removed_nodes,self.loc.clone(),self.nw.clone())))
    }

    fn latest_node_reaching(&self, node: NodeId) -> Option<Position>{
        if !self.nw.can_reach(self.nodes[0], node) {
            None
        } else {

            let candidate = self.latest_arrival_before(self.nw.node(node).start_time(), 0, self.nodes.len());
            match candidate {
                None => None,
                Some(p) => {
                    let mut pos = p;
                    while !self.nw.can_reach(self.nodes[pos],node) {
                        pos -= 1;
                    }
                    Some(pos)
                }
            }
        }
    }

    fn latest_arrival_before(&self, time: Time, left: Position, right: Position) -> Option<Position> {
        if left+1 == right {
            if self.nw.node(self.nodes[left]).end_time() <= time { Some(left) } else { None }
        } else {
            let mid = left + (right - left) / 2;
            if self.nw.node(self.nodes[mid]).end_time() <= time {
                self.latest_arrival_before(time, mid, right)
            } else {
                self.latest_arrival_before(time, left, mid)
            }
        }
    }

    fn earliest_node_reached_by(&self, node: NodeId) -> Option<Position>{
        if !self.nw.can_reach(node, *self.nodes.last().unwrap()) {
            None
        } else {

            let candidate = self.earliest_departure_after(self.nw.node(node).end_time(), 0, self.nodes.len());
            match candidate {
                None => None,
                Some(p) => {
                    let mut pos = p;
                    while !self.nw.can_reach(node, self.nodes[pos]) {
                        pos += 1;
                    }
                    Some(pos)
                }
            }
        }
    }

    fn earliest_departure_after(&self, time: Time, left: Position, right: Position) -> Option<Position> {
        if left+1 == right {
            if self.nw.node(self.nodes[left]).start_time() >= time { Some(left) } else { None }
        } else {
            let mid = left + (right - left - 1) / 2;
            if self.nw.node(self.nodes[mid]).start_time() >= time {
                self.earliest_departure_after(time, left, mid+1)
            } else {
                self.earliest_departure_after(time, mid+1, right)
            }
        }
    }

    pub(crate) fn print(&self) {
        println!("tour with {} nodes of length {} and travel time {}:", self.nodes.len(), self.distance(), self.travel_time());
        for node in self.nodes.iter() {
            println!("\t\t* {}", self.nw.node(*node));
        }
    }
}

impl Tour {

    /// Creates a new tour from a vector of NodeIds. Asserts that the tour is valid:
    /// * starts with a StartNode
    /// * end with an EndNode
    /// * only Service or MaintenanceNodes in the middle
    /// * each node can reach is successor
    pub(super) fn new(unit: UnitId, nodes: Vec<NodeId>, loc: Rc<Locations>,nw: Rc<Network>) -> Tour {
        assert!(matches!(nw.node(nodes[0]),Node::Start(_)), "Tour needs to start with a StartNode");
        assert!(matches!(nw.node(nodes[nodes.len()-1]), Node::End(_)), "Tour needs to end with a EndNode");
        for i in 1..nodes.len() - 1 {
            let n = nw.node(nodes[i]);
            assert!(matches!(n, Node::Service(_)) || matches!(n, Node::Maintenance(_)), "Tour can only have Service or Maintenance Nodes in the middle");
        }
        for (&a,&b) in nodes.iter().tuple_windows() {
            assert!(nw.can_reach(a,b),"Not a valid Tour");
        }
        Tour{unit, nodes, loc, nw}
    }

    /// Creates a new tour from a vector of NodeIds. Trusts that the vector leads to a valid Tour.
    pub(super) fn new_trusted(unit: UnitId, nodes: Vec<NodeId>, loc: Rc<Locations>,nw: Rc<Network>) -> Tour {
        Tour{unit, nodes, loc, nw}
    }
}


impl fmt::Display for Tour {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "tour of {} with {} nodes", self.unit, self.nodes.len())
    }
}
