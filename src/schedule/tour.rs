use std::fmt;
use crate::distance::Distance;
use crate::time::{Time, Duration};
use crate::locations::Locations;
use crate::units::UnitType;
use crate::network::Network;
use crate::network::nodes::Node;
use crate::base_types::NodeId;
use crate::schedule::path::{Path,Segment};

use std::cmp::Ordering;

use itertools::Itertools;

use std::sync::Arc;

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

    overhead_time: Duration,
    service_distance: Distance,
    dead_head_distance: Distance,

    loc: Arc<Locations>,
    nw: Arc<Network>,
}

impl Tour {
    pub(crate) fn nodes_iter(&self) -> impl Iterator<Item=&NodeId> +'_{
        self.nodes.iter()
    }

    /// return an iterator over all nodes (by start time) skipping the StartNode
    pub(crate) fn movable_nodes(&self) -> impl Iterator<Item=&NodeId> {
        let mut iter = self.nodes.iter();
        if !self.is_dummy {
            iter.next(); // skip StartNode
        }
        iter
    }

    /// the overhead time (dead_head + idle) between the predecessor and  the node itself
    pub(crate) fn preceding_overhead(&self, node: NodeId) -> Duration {
        if node == self.first_node() {
            Duration::Infinity
        } else {
            let pos = self.position_of(node).unwrap();
            let predecessor = *self.nodes.get(pos - 1).unwrap();
            self.nw.node(node).start_time() - self.nw.node(predecessor).end_time()
        }
    }
    /// the overhead time (dead_head + ilde) between the node itself and its successor
    pub(crate) fn subsequent_overhead(&self, node: NodeId) -> Duration {
        if node == self.last_node() {
            Duration::Infinity
        } else {
            let pos = self.position_of(node).unwrap();
            let successor = *self.nodes.get(pos + 1).unwrap();
            self.nw.node(successor).start_time() - self.nw.node(node).end_time()
        }
    }

    pub(crate) fn last_node(&self) -> NodeId {
        *self.nodes.last().unwrap()
    }

    pub(crate) fn first_node(&self) -> NodeId {
        *self.nodes.first().unwrap()
    }

    pub(crate) fn nth_node(&self,pos: usize) -> NodeId {
        *self.nodes.get(pos).unwrap()
    }

    pub(crate) fn len(&self) -> usize {
        self.nodes.len()
    }

    pub(crate) fn distance(&self) -> Distance {
        self.service_distance + self.dead_head_distance
    }

    pub(crate) fn dead_head_distance(&self) -> Distance {
        self.dead_head_distance
    }

    // pub(crate) fn travel_time(&self) -> Duration { // Care: does not count Maintenance
        // let service_tt: Duration = self.nodes.iter().map(|&n| self.nw.node(n).travel_time()).sum();
        // let dead_head_tt = self.nodes.iter().tuple_windows().map(
            // |(&a,&b)| self.loc.travel_time(self.nw.node(a).end_location(), self.nw.node(b).start_location())).sum();
        // service_tt + dead_head_tt
    // }

    /// return the overhead_time (dead head travel time + idle time) of the tour.
    pub(crate) fn overhead_time(&self) -> Duration {
        self.overhead_time
    }

    /// return the usefule_time (end - start for each node, including maintenance_slots) of the tour.
    // pub(crate) fn useful_time(&self) -> Duration {
        // self.nodes.iter().map(|&n| self.nw.node(n).useful_duration()).sum()
    // }

    pub(crate) fn sub_path(&self, segment: Segment) -> Result<Path,String> {
        let start_pos = self.earliest_not_reaching_node(segment.start()).ok_or_else(||String::from("segment.start() not part of Tour."))?;
        if segment.start() != self.nodes[start_pos] {
            return Err(String::from("segment.start() not part of Tour."));
        }
        let end_pos = self.earliest_not_reaching_node(segment.end()).ok_or_else(|| String::from("segment.end() not part of Tour."))?;
        if segment.end() != self.nodes[end_pos] {
            return Err(String::from("segment.end() not part of Tour."));
        }

        Ok(Path::new_trusted(self.nodes[start_pos..end_pos+1].to_vec(), self.nw.clone()))

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

        // compute useful_time, service_distance and dead_head_distance for the path:
        let path_useful_time = path.iter().map(|n| self.nw.node(*n).useful_duration()).sum();
        let path_service_distance = path.iter().map(|n| self.nw.node(*n).travel_distance()).sum();
        let path_dead_head_distance =
            if start_pos == 0 {
                Distance::zero()
            } else {
                self.loc.distance(self.nw.node(*self.nodes.get(start_pos-1).unwrap()).end_location(), self.nw.node(path.first()).start_location())
            } +

            path.iter().tuple_windows().map(|(&a,&b)| self.loc.distance(self.nw.node(a).end_location(),self.nw.node(b).start_location())).sum() +

            if end_pos >= self.nodes.len() || path.is_empty() {
                Distance::zero()
            } else {
                self.loc.distance(self.nw.node(path.last()).end_location(), self.nw.node(*self.nodes.get(end_pos).unwrap()).start_location())
            };


        // remove all elements in start_pos..end_pos and replace them by
        // node_sequence. Removed nodes are returned.
        let mut new_tour_nodes = self.nodes.clone();
        new_tour_nodes.splice(start_pos..end_pos,path.consume());



        // compute useful_time, service_distance and dead_head_distance for the segment that is
        // removed:
        let removed_useful_time = (start_pos..end_pos).map(|i| self.nw.node(self.nodes[i]).useful_duration()).sum();
        let removed_service_distance = (start_pos..end_pos).map(|i| self.nw.node(self.nodes[i]).travel_distance()).sum();
        let removed_dead_head_distance: Distance =
            if start_pos == 0 || start_pos >= self.nodes.len() {
                Distance::zero()
            } else {
                self.loc.distance(self.nw.node(self.nodes[start_pos-1]).end_location(), self.nw.node(self.nodes[start_pos]).start_location())
            } +
            (start_pos..end_pos).tuple_windows().map(
            |(i,j)| self.loc.distance(self.nw.node(self.nodes[i]).end_location(),self.nw.node(self.nodes[j]).start_location())).sum() +
            if end_pos == self.nodes.len() || (start_pos == end_pos && start_pos > 0) || end_pos == 0 {
                Distance::zero()
            } else {
                self.loc.distance(self.nw.node(*self.nodes.get(end_pos-1).unwrap()).end_location(), self.nw.node(*self.nodes.get(end_pos).unwrap()).start_location())
            };


        // compute the overhead_time, service_distance and dead_head_distance for the new tour:
        let overhead_time = if self.is_dummy {
            // for a dummy the total time (start of the first node - end of the last node)
            // might have changed.
            let total_time_original = self.nw.node(*self.nodes.last().unwrap()).end_time() - self.nw.node(*self.nodes.first().unwrap()).start_time();
            let total_time_new = self.nw.node(*new_tour_nodes.last().unwrap()).end_time() - self.nw.node(*new_tour_nodes.first().unwrap()).start_time();
            self.overhead_time + total_time_new + removed_useful_time - path_useful_time - total_time_original
        } else {
            self.overhead_time + removed_useful_time - path_useful_time
        };
        let service_distance = self.service_distance + path_service_distance - removed_service_distance;
        let dead_head_distance = self.dead_head_distance + path_dead_head_distance - removed_dead_head_distance;

        Ok(Tour::new_trusted(self.unit_type, new_tour_nodes, self.is_dummy, overhead_time, service_distance, dead_head_distance, self.loc.clone(),self.nw.clone()))
    }

    pub(super) fn insert_single_node(&self, node: NodeId) -> Result<Tour,String> {
        self.insert(Path::new(vec!(node), self.nw.clone()))
    }


    /// remove the segment of the tour. The subpath between segment.start() and segment.end() is removed and the new
    /// shortened Tour as well as the removed nodes (as Path) are returned.
    /// Fails if either segment.start() or segment.end() are not part of the Tour or if the Start or EndNode would
    /// get removed.
    pub(crate) fn remove(&self, segment: Segment) -> Result<(Tour, Path),String> {

        let start_pos= self.position_of(segment.start())?;
        let end_pos= self.position_of(segment.end())?;

        self.removable_by_pos(start_pos, end_pos)?;

        // compute usefile_time, service_distance and dead_head_distance for the removed segment:
        let removed_useful_time = (start_pos..end_pos+1).map(|i| self.nw.node(self.nodes[i]).useful_duration()).sum();
        let removed_service_distance = (start_pos..end_pos+1).map(|i| self.nw.node(self.nodes[i]).travel_distance()).sum();
        let removed_dead_head_distance: Distance =
            if start_pos == 0 {
                Distance::zero()
            } else {
                self.loc.distance(self.nw.node(*self.nodes.get(start_pos-1).unwrap()).end_location(), self.nw.node(*self.nodes.get(start_pos).unwrap()).start_location())
            } +
            (start_pos..end_pos+1).tuple_windows().map(
            |(i,j)| self.loc.distance(self.nw.node(self.nodes[i]).end_location(),self.nw.node(self.nodes[j]).start_location())).sum() +
            if end_pos == self.nodes.len() - 1 {
                Distance::zero()
            } else {
                self.loc.distance(self.nw.node(self.nodes[end_pos]).end_location(), self.nw.node(self.nodes[end_pos+1]).start_location())
            };

        // compute the dead_head_distance for the new gap that is created:
        let added_dead_head_distance =
            if start_pos == 0 || end_pos == self.nodes.len() - 1 {
                Distance::zero()
            } else {
                self.loc.distance(self.nw.node(self.nodes[start_pos-1]).end_location(), self.nw.node(self.nodes[end_pos+1]).start_location())
            };


        // remove the segment from the tour:
        let mut tour_nodes: Vec<NodeId> = self.nodes[..start_pos].to_vec();
        tour_nodes.extend(self.nodes[end_pos+1..].iter().copied());
        let removed_nodes: Vec<NodeId> = self.nodes[start_pos..end_pos+1].to_vec();


        // compute the overhead_time, service_distance and dead_head_distance for the new tour:
        let overhead_time =
            if self.is_dummy {
                if tour_nodes.len() == 0 {
                    Duration::zero()
                } else {
                    let total_time_original = self.nw.node(*self.nodes.last().unwrap()).end_time() - self.nw.node(*self.nodes.first().unwrap()).start_time();
                    let total_time_new = self.nw.node(*tour_nodes.last().unwrap()).end_time() - self.nw.node(*tour_nodes.first().unwrap()).start_time();
                    self.overhead_time + removed_useful_time + total_time_new - total_time_original
                }
            } else {
                self.overhead_time + removed_useful_time
            };
        let service_distance = self.service_distance - removed_service_distance;
        let dead_head_distance = self.dead_head_distance + added_dead_head_distance - removed_dead_head_distance;


        Ok((Tour::new_trusted(self.unit_type, tour_nodes, self.is_dummy, overhead_time, service_distance, dead_head_distance, self.loc.clone(),self.nw.clone()), Path::new_trusted(removed_nodes,self.nw.clone())))
    }

    pub(crate) fn remove_single_node(&self, node: NodeId) -> Result<Tour,String> {
        self.remove(Segment::new(node, node)).map(|tuple| tuple.0)
    }

    /// for a given segment (in general of another tour) returns all nodes that are conflicting
    /// when the segment would have been inserted. These nodes form a path.
    /// Fails if the segment insertion would not lead  to a valid Tour (for example start node
    /// cannot reach segment.start(), or segment.end() cannot reach end node).
    pub(super) fn conflict(&self, segment: Segment) -> Result<Path,String> {

        let (start_pos,end_pos) = self.get_insert_positions(segment);

        self.test_if_valid_replacement(segment, start_pos, end_pos)?;

        let conflicted_nodes = self.nodes[start_pos..end_pos].to_vec();
        Ok(Path::new_trusted(conflicted_nodes,self.nw.clone()))
    }

    pub(super) fn conflict_single_node(&self, node: NodeId) -> Result<Path,String> {
        self.conflict(Segment::new(node, node))
    }

    pub(crate) fn position_of(&self, node: NodeId) -> Result<usize, String> {
        let pos = self.nodes.binary_search_by(|other| self.nw.node(*other)
                                              .cmp_start_time(self.nw.node(node))).map_err(|_| "Node not part of tour.")?;
        assert!(node == self.nodes[pos],"fehler");
        Ok(pos)
    }

    pub(crate) fn removable(&self, segment: Segment) -> bool {
        let start_pos_res = self.position_of(segment.start());
        let end_pos_res = self.position_of(segment.end());
        if start_pos_res.is_err() || end_pos_res.is_err() {
            false
        } else {
            self.removable_by_pos(start_pos_res.unwrap(), end_pos_res.unwrap()).is_ok()}
    }

    pub(crate) fn removable_single_node(&self, node: NodeId) -> bool {
        self.removable(Segment::new(node, node))
    }


    /// start_position is here the position of the first node that should be removed
    /// end_position is here the position in the tour of the last node that should be removed
    /// in other words [start_position .. end_position+1) is about to be removed
    fn removable_by_pos(&self, start_position: usize, end_position: usize) -> Result<(), String> {
        if !self.is_dummy && start_position == 0 {
            return Err(String::from("StartNode cannot be removed."));
        }
        if !self.is_dummy && end_position == self.nodes.len()-1 {
            return Err(String::from("EndNode cannot be removed."));
        }
        if start_position > end_position {
            return Err(String::from("segment.start() comes after segment.end()."));
        }

        if start_position > 0 && end_position < self.nodes.len() - 1 && !self.nw.can_reach(self.nodes[start_position-1],self.nodes[end_position+1]) {
            return Err(format!("Removing nodes ({} to {}) makes the tour invalid. Dead-head-trip is slower than service-trips.", self.nodes[start_position], self.nodes[end_position]));
        }
        Ok(())

    }

    pub(crate) fn print(&self) {
        println!("{}tour with {} nodes:", if self.is_dummy {"dummy-"} else {""}, self.nodes.len(),);
        for node in self.nodes.iter() {
            print!("\t* ");
            self.nw.node(*node).print();
        }
    }

}


// one tour is bigger than the other if the number of nodes are bigger.
// Ties are broken by comparing nodes from start to finish.
// Nodes ar compared by start time (ties are by end_time then id)
impl Ord for Tour {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.nodes.len().cmp(&other.nodes.len()) {
            Ordering::Equal => {
                match self.nodes.iter().zip(other.nodes.iter()).map(|(node, other_node)| self.nw.node(*node).cmp_start_time(self.nw.node(*other_node))).find(|ord| *ord != Ordering::Equal) {
                    None => Ordering::Equal,
                    Some(other) => other
                    }
                }
            other => other
        }
    }
}

impl PartialOrd for Tour {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// two tours are equal if they consists of the exact same nodes
impl PartialEq for Tour {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Eq for Tour {}



// private methods
impl Tour {
    fn test_if_valid_replacement(&self, segment: Segment, start_pos: Position, end_pos: Position) -> Result<(), String> {

        // first test if end nodes make sense:
        let mut has_endnode = false;
        let last_node = self.nw.node(segment.end());
        if matches!(last_node, Node::End(_)) {
            has_endnode = true;
            if last_node.unit_type() != self.unit_type {
                return Err(String::from("Unit types do not match!"));
            }
            if self.is_dummy {
                return Err(String::from("Dummy-Tours cannot take EndNodes!"));
            }
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

    pub(crate) fn earliest_not_reaching_node(&self, node: NodeId) -> Option<Position> {
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

    /// Creates a new tour from a vector of NodeIds. Checks that the tour is valid:
    /// * starts with a StartNode
    /// * end with an EndNode
    /// * only Service or MaintenanceNodes in the middle
    /// * each node can reach is successor
    /// If one of the checks fails an error message is returned.
    pub(super) fn new(unit_type: UnitType, nodes: Vec<NodeId>, loc: Arc<Locations>, nw: Arc<Network>) -> Result<Tour, String> {
        Tour::new_allow_invalid(unit_type, nodes, loc, nw).map_err(|(_, error_msg)| error_msg)
    }

    /// Creates a new tour from a vector of NodeIds. Checks that the tour is valid:
    /// * starts with a StartNode
    /// * end with an EndNode
    /// * only Service or MaintenanceNodes in the middle
    /// * each node can reach is successor
    /// If one of the checks fails an error is returned containing the error message but also the
    /// invalid tour.
    pub(super) fn new_allow_invalid(unit_type: UnitType, nodes: Vec<NodeId>, loc: Arc<Locations>, nw: Arc<Network>) -> Result<Tour, (Tour, String)> {
        let mut error_msg = String::new();
        if !matches!(nw.node(nodes[0]),Node::Start(_)) {
            error_msg.push_str(&format!("Tour needs to start with a StartNode, not with: {}.\n", nw.node(nodes[0])));
        }
        if !matches!(nw.node(nodes[nodes.len()-1]), Node::End(_)) {
            error_msg.push_str(&format!("Tour needs to end with a EndNode, not with: {},\n", nw.node(nodes[nodes.len()-1])));
        }
        for i in 1..nodes.len() - 1 {
            let n = nw.node(nodes[i]);
            if !matches!(n, Node::Service(_)) && !matches!(n, Node::Maintenance(_)) {
                error_msg.push_str(&format!("Tour can only have Service or Maintenance Nodes in the middle, not: {}.\n", n));
            }
        }
        for (&a,&b) in nodes.iter().tuple_windows() {
            if !nw.can_reach(a,b) {
                error_msg.push_str(&format!("Not a valid Tour: {} cannot reach {}.\n", nw.node(a), nw.node(b)));
            }
        }
        if error_msg.len() > 0 {
            Err((Tour::new_computing(unit_type, nodes, false, loc, nw), error_msg))
        } else {
            Ok(Tour::new_computing(unit_type, nodes, false, loc, nw))
        }
    }

    pub(super) fn new_dummy(unit_type: UnitType, nodes: Vec<NodeId>, loc: Arc<Locations>, nw: Arc<Network>) -> Result<Tour, String> {
        for (&a,&b) in nodes.iter().tuple_windows() {
            if !nw.can_reach(a,b) {
                return Err(format!("Not a valid Dummy-Tour: {} cannot reach {}.\n", nw.node(a), nw.node(b)));
            }
        }
        Ok(Tour::new_computing(unit_type, nodes, true, loc, nw))
    }

    /// Creates a new tour from a vector of NodeIds. Trusts that the vector leads to a valid Tour.
    pub(super) fn new_trusted(unit_type: UnitType, nodes: Vec<NodeId>, is_dummy: bool, overhead_time: Duration, service_distance: Distance, dead_head_distance: Distance, loc: Arc<Locations>, nw: Arc<Network>) -> Tour {
        Tour{unit_type, nodes, is_dummy, overhead_time, service_distance, dead_head_distance, loc, nw}
    }

    pub(super) fn new_dummy_by_path(unit_type: UnitType, path: Path, loc: Arc<Locations>, nw: Arc<Network>) -> Tour {
        Tour::new_computing(unit_type, path.consume(), true, loc, nw)
    }

    fn new_computing(unit_type: UnitType, nodes: Vec<NodeId>, is_dummy: bool, loc: Arc<Locations>, nw: Arc<Network>) -> Tour {
        let overhead_time = nodes.iter().tuple_windows().map(|(&a,&b)| nw.node(b).start_time() - nw.node(a).end_time()).sum();
        let service_distance = nodes.iter().map(|&n| nw.node(n).travel_distance()).sum();
        let dead_head_distance = nodes.iter().tuple_windows().map(
            |(&a,&b)| loc.distance(nw.node(a).end_location(),nw.node(b).start_location())).sum();
        Tour{unit_type, nodes, is_dummy, overhead_time, service_distance, dead_head_distance, loc, nw}
    }
}


impl fmt::Display for Tour {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut nodes_iter = self.nodes.iter();
        write!(f, "{}", self.nw.node(*nodes_iter.next().unwrap()))?;
        for node in nodes_iter {
            write!(f, " - {}", self.nw.node(*node))?;
        }
        Ok(())
    }
}

