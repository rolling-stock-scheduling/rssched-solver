use std::fmt;
use crate::distance::Distance;
use crate::time::{Time, Duration};
use crate::locations::Locations;
use crate::units::UnitType;
use crate::network::Network;
use crate::network::nodes::Node;
use crate::base_types::NodeId;
use crate::schedule::path::{Path,Segment};

use itertools::Itertools;

use std::rc::Rc;

type Position = usize; // the position within the tour from 0 to nodes.len()-1

/// this represents a tour of a single unit (or dummy unit). For real units it always start at the StartNode of the unit and ends
/// with an EndNode that has the type of the unit. For dummy units only the type must fit.
/// It is an immutable objects. So whenever some modification is applied a copy of the tour
/// is created.
#[derive(Clone)]
pub(crate) struct Tour {
    unit_type: UnitType,
    nodes: Vec<NodeId>, // nodes will always be sorted by start_time
    is_dummy: bool,

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

    pub(crate) fn first_node(&self) -> NodeId {
        *self.nodes.first().unwrap()
    }

    pub(crate) fn len(&self) -> usize {
        self.nodes.len()
    }

    pub(crate) fn distance(&self) -> Distance {
        let service_distance: Distance = self.nodes.iter().map(|&n| self.nw.node(n).travel_distance()).sum();

        let dead_head_distance = self.dead_head_distance();
        service_distance + dead_head_distance
    }

    pub(crate) fn dead_head_distance(&self) -> Distance {
        self.nodes.iter().tuple_windows().map(
            |(&a,&b)| self.loc.distance(self.nw.node(a).end_location(),self.nw.node(b).start_location())).sum()
    }

    pub(crate) fn travel_time(&self) -> Duration {
        let service_tt: Duration = self.nodes.iter().map(|&n| self.nw.node(n).travel_time()).sum();
        let dead_head_tt = self.nodes.iter().tuple_windows().map(
            |(&a,&b)| self.loc.travel_time(self.nw.node(a).end_location(), self.nw.node(b).start_location())).sum();
        service_tt + dead_head_tt
    }

    /// return the overhead_time (dead head travel time + idle time) of the tour.
    pub(crate) fn overhead_time(&self) -> Duration {
        self.nodes.iter().tuple_windows().map(|(&a,&b)| self.nw.node(b).start_time() - self.nw.node(a).end_time()).sum()
    }

    pub(crate) fn sub_path(&self, segment: Segment) -> Result<Path,String> {
        let start_pos = self.earliest_not_reaching_node(segment.start()).ok_or(String::from("segment.start() not part of Tour."))?;
        if segment.start() != self.nodes[start_pos] {
            return Err(String::from("segment.start() not part of Tour."));
        }
        let end_pos = self.earliest_not_reaching_node(segment.end()).ok_or(String::from("segment.end() not part of Tour."))?;
        if segment.end() != self.nodes[end_pos] {
            return Err(String::from("segment.end() not part of Tour."));
        }

        Ok(Path::new_trusted(self.nodes[start_pos..end_pos+1].iter().copied().collect(), self.loc.clone(), self.nw.clone()))

    }

    /// inserts the provided node sequence on the correct position (time-wise). The sequence will
    /// stay uninterrupted. All removed nodes (due to time-clashes) are returned.
    /// Assumes that provided node sequence is feasible.
    /// For unit tours it fails if sequence is not reachable from the start node. If end_node cannot be reached,
    /// sequence must itself end with a end_node (or it fails).
    pub(super) fn insert(&self, path: Path) -> Result<Tour,String> {
        let segment = Segment::new(path.first(), path.last());

        let (start_pos,end_pos) = self.get_insert_positions(segment);

        self.test_if_valid_replacement(segment, start_pos, end_pos)?;

        // remove all elements in start_pos..end_pos and replace them by
        // node_sequence. Removed nodes are returned.
        let mut new_tour_nodes = self.nodes.clone();
        new_tour_nodes.splice(start_pos..end_pos,path.consume());
        Ok(Tour::new_trusted(self.unit_type, new_tour_nodes, self.is_dummy, self.loc.clone(),self.nw.clone()))
    }

    /// remove the segment of the tour. The subpath between segment.start() and segment.end() is removed and the new
    /// shortened Tour as well as the removed nodes (as Path) are returned.
    /// Fails if either segment.start() or segment.end() are not part of the Tour or if the Start or EndNode would
    /// get removed.
    pub(crate) fn remove(&self, segment: Segment) -> Result<(Tour, Path),String> {

        let start_position = self.nodes.iter().position(|&n| n == segment.start()).ok_or("segment.start() is not part of the Tour")?;
        let end_position = self.nodes.iter().position(|&n| n == segment.end()).ok_or("segment.end() is not part of the Tour")?;

        if !self.is_dummy && start_position == 0 {
            return Err(String::from("StartNode cannot be removed."));
        }
        if !self.is_dummy && start_position == self.nodes.len()-1 {
            return Err(String::from("EndNode cannot be removed."));
        }
        if start_position > end_position {
            return Err(String::from("segment.start() comes after segment.end()."));
        }

        if start_position > 0 && end_position < self.nodes.len() - 1 && !self.nw.can_reach(self.nodes[start_position-1],self.nodes[end_position+1]) {
            return Err(String::from("Strange! Removing nodes make the tour invalid. Dead_head travel time not consistent."));
        }
        let mut tour_nodes: Vec<NodeId> = self.nodes[..start_position].iter().cloned().collect();
        tour_nodes.extend(self.nodes[end_position+1..].iter().cloned());
        let removed_nodes: Vec<NodeId> = self.nodes[start_position..end_position+1].iter().cloned().collect();

        Ok((Tour::new_trusted(self.unit_type, tour_nodes, self.is_dummy, self.loc.clone(),self.nw.clone()), Path::new_trusted(removed_nodes,self.loc.clone(),self.nw.clone())))
    }

    /// for a given segment (in general of another tour) returns all nodes that are conflicting
    /// when the segment would have been inserted. These nodes form a path.
    /// Fails if the segment insertion would not lead  to a valid Tour (for example start node
    /// cannot reach segment.start(), or segment.end() cannot reach end node).
    pub(super) fn conflict(&self, segment: Segment) -> Result<Path,String> {

        let (start_pos,end_pos) = self.get_insert_positions(segment);

        self.test_if_valid_replacement(segment, start_pos, end_pos)?;

        let conflicted_nodes = self.nodes[start_pos..end_pos].iter().cloned().collect();
        Ok(Path::new_trusted(conflicted_nodes,self.loc.clone(),self.nw.clone()))
    }

    pub(crate) fn print(&self) {
        println!("{}tour with {} nodes of length {} and travel time {}:", if self.is_dummy {"dummy-"} else {""}, self.nodes.len(), self.distance(), self.travel_time());
        for node in self.nodes.iter() {
            println!("\t\t* {}", self.nw.node(*node));
        }
    }
}





// private methods
impl Tour {
    fn test_if_valid_replacement(&self, segment: Segment, start_pos: Position, end_pos: Position) -> Result<(), String> {

        // first test if end nodes make sense:
        let mut has_endnode = false;
        let last = segment.end();
        let last_node = self.nw.node(last);
        if matches!(last_node, Node::End(_)) {
            if last_node.unit_type() != self.unit_type {
                return Err(String::from("Unit types do not match!"));
            }
            if self.is_dummy {
                return Err(String::from("Dummy-Tours cannot take EndNodes!"));
            }
            has_endnode = true;
        }
        if end_pos == self.nodes.len() && !self.is_dummy && !has_endnode {
            return Err(String::from("Cannot insert path to tour, as it does not end with an EndPoint and the old EndPoint cannot be reached!"));
        }

        // test if start node makes sense:
        if start_pos == 0 && !self.is_dummy {
            return Err(String::from("Cannot replace the start node."));
        }

        Ok(())

    }

    /// return the range of the conflicting nodes. start..end must be replaced by path.
    /// that means start is the first conflicting node and end-1 is the last conflciting node
    fn get_insert_positions(&self, segment: Segment) -> (Position,Position) {
        let first = segment.start();
        let last = segment.end();


        let start_pos = self.earliest_not_reaching_node(first).unwrap_or(self.nodes.len());
        let end_pos = match self.latest_not_reached_by_node(last) {
            None => 0,
            Some(p) => p+1
        };
        (start_pos,end_pos)
    }

    fn earliest_not_reaching_node(&self, node: NodeId) -> Option<Position> {
        if self.nw.can_reach(*self.nodes.last().unwrap(), node) {
            return None; // all tour-nodes can reach node, even the last
        }
        let candidate = self.earliest_arrival_after(self.nw.node(node).start_time(), 0, self.nodes.len());
        let mut pos = candidate.unwrap_or(self.nodes.len()-1);
        while pos > 0 && !self.nw.can_reach(self.nodes[pos-1],node) {
            pos -= 1;
        }
        Some(pos)
    }

    fn earliest_arrival_after(&self, time: Time, left: Position, right: Position) -> Option<Position> {
        if left+1 == right {
            if self.nw.node(self.nodes[left]).end_time() >= time { Some(left) } else { None }
        } else {
            let mid = left + (right - left) / 2;
            if self.nw.node(self.nodes[mid-1]).end_time() >= time {
                self.earliest_arrival_after(time, left, mid)
            } else {
                self.earliest_arrival_after(time, mid, right)
            }
        }
    }

    /// computes the position of the latest tour-node that is not reached by node.
    /// if node can reach all tour-nodes, None is returned.
    fn latest_not_reached_by_node(&self, node: NodeId) -> Option<Position> {
        if self.nw.can_reach(node, *self.nodes.first().unwrap()) {
            return None; // node can reach all nodes, even the first
        }
        // the candidate cannot be reached by node for sure.
        let candidate = self.latest_departure_before(self.nw.node(node).end_time(), 0, self.nodes.len());
        // but later nodes might also not be reached by node.

        let mut pos = candidate.unwrap_or(0);
        while pos < self.nodes.len()-1 && !self.nw.can_reach(node, self.nodes[pos+1]) {
            pos += 1;
        }
        Some(pos)
    }

    fn latest_departure_before(&self, time: Time, left: Position, right: Position) -> Option<Position> {
        if left+1 == right {
            if self.nw.node(self.nodes[left]).start_time() <= time { Some(left) } else { None }
        } else {
            let mid = left + (right - left) / 2;
            if self.nw.node(self.nodes[mid]).start_time() <= time {
                self.latest_departure_before(time, mid, right)
            } else {
                self.latest_departure_before(time, left, mid)
            }
        }
    }

}

impl Tour {

    /// Creates a new tour from a vector of NodeIds. Asserts that the tour is valid:
    /// * starts with a StartNode
    /// * end with an EndNode
    /// * only Service or MaintenanceNodes in the middle
    /// * each node can reach is successor
    pub(super) fn new(unit_type: UnitType, nodes: Vec<NodeId>, loc: Rc<Locations>, nw: Rc<Network>) -> Tour {
        assert!(matches!(nw.node(nodes[0]),Node::Start(_)), "Tour needs to start with a StartNode");
        assert!(matches!(nw.node(nodes[nodes.len()-1]), Node::End(_)), "Tour needs to end with a EndNode");
        for i in 1..nodes.len() - 1 {
            let n = nw.node(nodes[i]);
            assert!(matches!(n, Node::Service(_)) || matches!(n, Node::Maintenance(_)), "Tour can only have Service or Maintenance Nodes in the middle");
        }
        for (&a,&b) in nodes.iter().tuple_windows() {
            assert!(nw.can_reach(a,b),"Not a valid Tour");
        }
        Tour{unit_type, nodes, is_dummy: false, loc, nw}
    }

    pub(super) fn new_dummy(unit_type: UnitType, nodes: Vec<NodeId>, loc: Rc<Locations>, nw: Rc<Network>) -> Tour {
        for (&a,&b) in nodes.iter().tuple_windows() {
            assert!(nw.can_reach(a,b),"Not a valid Dummy-Tour");
        }
        Tour{unit_type, nodes, is_dummy: true, loc, nw}
    }

    /// Creates a new tour from a vector of NodeIds. Trusts that the vector leads to a valid Tour.
    pub(super) fn new_trusted(unit_type: UnitType, nodes: Vec<NodeId>, is_dummy: bool, loc: Rc<Locations>, nw: Rc<Network>) -> Tour {
        Tour{unit_type, nodes, is_dummy, loc, nw}
    }

    pub(super) fn new_dummy_by_path(unit_type: UnitType, path: Path, loc: Rc<Locations>, nw: Rc<Network>) -> Tour {
        Tour{unit_type, nodes:path.consume(), is_dummy: true, loc, nw}
    }
}


impl fmt::Display for Tour {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}tour for {:?} with {} nodes", if self.is_dummy {"dummy-"} else {""}, self.unit_type, self.nodes.len())
    }
}

