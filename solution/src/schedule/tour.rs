use crate::schedule::objective;
use crate::schedule::path::{Path, Segment};
use sbb_model::base_types::{Cost, DateTime, Distance, Duration, NodeId, COST_ZERO};
use sbb_model::config::Config;
use sbb_model::network::{nodes::Node, Network};
use std::{fmt, iter};

use std::cmp::Ordering;

use itertools::Itertools;

use std::sync::Arc;

type Position = usize; // the position within the tour from 0 to nodes.len()-1

/// this represents a tour of a single vehicle (or a dummy tour). Real tours always start and ends at a depot.
/// For dummy tour this does not hold.
/// It is an immutable objects. So whenever some modification is applied a copy of the tour
/// is created.
#[derive(Clone)]
pub(crate) struct Tour {
    nodes: Vec<NodeId>, // nodes will always be sorted by start_time; for non-dummy exactly first
    // and last node is a depot
    is_dummy: bool, // true if this is a dummy tour

    overhead_time: Duration, // the overhead time (dead_head + idle) of the tour
    service_distance: Distance,
    dead_head_distance: Distance,

    config: Arc<Config>,
    nw: Arc<Network>,
}

// basic public methods
impl Tour {
    pub(crate) fn is_dummy(&self) -> bool {
        self.is_dummy
    }

    pub(crate) fn len(&self) -> usize {
        self.nodes.len()
    }

    pub(crate) fn all_nodes_iter(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes.iter().copied()
    }

    /// return an iterator over all nodes (by start time) skipping the depot at the start and end
    pub(crate) fn movable_nodes(&self) -> Box<dyn Iterator<Item = NodeId> + '_> {
        if self.is_dummy {
            Box::new(self.nodes.iter().copied())
        } else {
            Box::new(
                self.nodes
                    .iter()
                    .skip(1)
                    .take(self.nodes.len() - 2)
                    .copied(),
            )
        }
    }

    pub(crate) fn dead_head_distance(&self) -> Distance {
        self.dead_head_distance
    }

    /// return the overhead_time (dead head travel time + idle time) of the tour.
    pub(crate) fn overhead_time(&self) -> Duration {
        self.overhead_time
    }

    /// return the service distance of the tour
    pub(crate) fn service_distance(&self) -> Distance {
        self.service_distance
    }

    /// the overhead time (dead_head + idle) between the predecessor and the node itself
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

    pub(crate) fn first_node(&self) -> NodeId {
        *self.nodes.first().unwrap()
    }

    pub(crate) fn last_node(&self) -> NodeId {
        *self.nodes.last().unwrap()
    }

    pub(crate) fn nth_node(&self, pos: usize) -> NodeId {
        *self.nodes.get(pos).expect("invalid position")
    }

    pub(crate) fn print(&self) {
        println!(
            "{}tour with {} nodes:",
            if self.is_dummy { "dummy-" } else { "" },
            self.nodes.len(),
        );
        for node in self.nodes.iter() {
            print!("\t* ");
            self.nw.node(*node).print();
        }
    }
}

// modification methods
impl Tour {
    /// for a given segment (in general of another tour) returns all nodes that are conflicting
    /// when the segment would have been inserted. These nodes form a path.
    /// Fails if the segment insertion would not lead  to a valid Tour (for example start node
    /// cannot reach segment.start(), or segment.end() cannot reach end node).
    pub(super) fn conflict(&self, segment: Segment) -> Result<Path, String> {
        let (start_pos, end_pos) = self.get_insert_positions(segment);

        self.test_if_valid_replacement(segment, start_pos, end_pos)?;

        let conflicted_nodes = self.nodes[start_pos..end_pos].to_vec();
        Ok(Path::new_trusted(conflicted_nodes, self.nw.clone()))
    }

    /// Returns the tour where the provided path is inserted into the correct position (time-wise). The path will
    /// stay uninterrupted, clashing nodes are removed.
    ///
    /// # Properties:
    /// - Assumes that provided node sequence is feasible.
    /// - If path contains depots (at the start or end) be self is a dummy-tour the depots are
    /// removed at the beginning.
    /// - If the provided sequence contains a start depot it will be inserted as a prefix.
    /// - If the provided path contains an end depot it will be inserted as a suffix (for non-dummy
    /// tours).
    /// - For non-dummy tours it fails if the start depot clashes and is not replaced with a new
    /// start depot. The same holds for the end depot.
    pub(super) fn insert(&self, path: Path) -> Result<Tour, String> {
        if path.is_empty() {
            return Err("Cannot insert empty path.".to_string());
        }

        // remove depots from path if self.is_dummy=true.
        let mut path = path;
        if self.is_dummy && matches!(*self.nw.node(path.first()), Node::Depot(_)) {
            path = path.drop_first();
        }
        if self.is_dummy && matches!(*self.nw.node(path.last()), Node::Depot(_)) {
            path = path.drop_last();
        }

        // get insertion position and check if insertion is valid
        let segment = Segment::new(path.first(), path.last());
        let (start_pos, end_pos) = self.get_insert_positions(segment);
        self.test_if_valid_replacement(segment, start_pos, end_pos)?;

        // compute useful_time, service_distance and dead_head_distance for the path:
        let path_useful_time = path.iter().map(|n| self.nw.node(*n).duration()).sum();
        let path_service_distance = path
            .iter()
            .map(|n| self.nw.node(*n).travel_distance())
            .sum();
        let path_dead_head_distance: Distance = if start_pos == 0 {
            Distance::zero()
        } else {
            self.nw
                .dead_head_distance_between(self.nodes[start_pos - 1], path.first())
        } + path
            .iter()
            .tuple_windows()
            .map(|(&a, &b)| self.nw.dead_head_distance_between(a, b))
            .sum()
            + if end_pos >= self.nodes.len() || path.is_empty() {
                Distance::zero()
            } else {
                self.nw
                    .dead_head_distance_between(path.last(), self.nodes[end_pos])
            };

        // compute useful_time, service_distance and dead_head_distance for the segment that is
        // removed:
        let removed_useful_time = (start_pos..end_pos)
            .map(|i| self.nw.node(self.nodes[i]).duration())
            .sum();
        let removed_service_distance = (start_pos..end_pos)
            .map(|i| self.nw.node(self.nodes[i]).travel_distance())
            .sum();
        let removed_dead_head_distance: Distance =
            if start_pos == 0 || start_pos >= self.nodes.len() {
                Distance::zero()
            } else {
                self.nw
                    .dead_head_distance_between(self.nodes[start_pos - 1], self.nodes[start_pos])
            } + (start_pos..end_pos)
                .tuple_windows()
                .map(|(i, j)| {
                    self.nw
                        .dead_head_distance_between(self.nodes[i], self.nodes[j])
                })
                .sum()
                + if end_pos == self.nodes.len()
                    || (start_pos == end_pos && start_pos > 0)
                    || end_pos == 0
                {
                    Distance::zero()
                } else {
                    self.nw
                        .dead_head_distance_between(self.nodes[end_pos - 1], self.nodes[end_pos])
                };

        // remove all elements in start_pos..end_pos and replace them by
        // node_sequence.
        let mut new_tour_nodes = self.nodes.clone();
        new_tour_nodes.splice(start_pos..end_pos, path.consume());

        // compute the overhead_time, service_distance and dead_head_distance for the new tour:
        let overhead_time = if start_pos == 0 || end_pos == self.nodes.len() {
            // the total time (start of the first node - end of the last node)
            // might have changed.
            let total_time_original = self.nw.node(*self.nodes.last().unwrap()).end_time()
                - self.nw.node(*self.nodes.first().unwrap()).start_time();
            let total_time_new = self.nw.node(*new_tour_nodes.last().unwrap()).end_time()
                - self.nw.node(*new_tour_nodes.first().unwrap()).start_time();
            self.overhead_time + total_time_new + removed_useful_time
                - path_useful_time
                - total_time_original
        } else {
            self.overhead_time + removed_useful_time - path_useful_time
        };
        let service_distance =
            self.service_distance + path_service_distance - removed_service_distance;
        let dead_head_distance =
            self.dead_head_distance + path_dead_head_distance - removed_dead_head_distance;

        Ok(Tour::new_precomputed(
            new_tour_nodes,
            self.is_dummy,
            overhead_time,
            service_distance,
            dead_head_distance,
            self.config.clone(),
            self.nw.clone(),
        ))
    }

    /// checks whether segment can be removed from tour or not.
    pub(crate) fn removable(&self, segment: Segment) -> bool {
        let start_pos_res = self.position_of(segment.start());
        let end_pos_res = self.position_of(segment.end());
        if start_pos_res.is_err() || end_pos_res.is_err() {
            false
        } else {
            self.test_if_sequence_is_removable(start_pos_res.unwrap(), end_pos_res.unwrap())
                .is_ok()
        }
    }

    /// remove the segment of the tour. The subpath between segment.start() and segment.end() is removed and the new
    /// shortened Tour as well as the removed nodes (as Path) are returned.
    /// Fails if either segment.start() or segment.end() are not part of the Tour or if the Start or EndNode would
    /// get removed.
    pub(crate) fn remove(&self, segment: Segment) -> Result<(Tour, Path), String> {
        let start_pos = self.position_of(segment.start())?;
        let end_pos = self.position_of(segment.end())?;

        self.test_if_sequence_is_removable(start_pos, end_pos)?;

        // compute usefile_time, service_distance and dead_head_distance for the removed segment:
        let removed_useful_time = (start_pos..end_pos + 1)
            .map(|i| self.nw.node(self.nodes[i]).duration())
            .sum();
        let removed_service_distance = (start_pos..end_pos + 1)
            .map(|i| self.nw.node(self.nodes[i]).travel_distance())
            .sum();
        let removed_dead_head_distance: Distance = if start_pos == 0 {
            Distance::zero()
        } else {
            self.nw
                .dead_head_distance_between(self.nodes[start_pos - 1], self.nodes[start_pos])
        } + (start_pos..end_pos + 1)
            .tuple_windows()
            .map(|(i, j)| {
                self.nw
                    .dead_head_distance_between(self.nodes[i], self.nodes[j])
            })
            .sum()
            + if end_pos == self.nodes.len() - 1 {
                Distance::zero()
            } else {
                self.nw
                    .dead_head_distance_between(self.nodes[end_pos], self.nodes[end_pos + 1])
            };

        // compute the dead_head_distance for the new gap that is created:
        let added_dead_head_distance = if start_pos == 0 || end_pos == self.nodes.len() - 1 {
            Distance::zero()
        } else {
            self.nw
                .dead_head_distance_between(self.nodes[start_pos - 1], self.nodes[end_pos + 1])
        };

        // remove the segment from the tour:
        let mut tour_nodes: Vec<NodeId> = self.nodes[..start_pos].to_vec();
        tour_nodes.extend(self.nodes[end_pos + 1..].iter().copied());
        let removed_nodes: Vec<NodeId> = self.nodes[start_pos..end_pos + 1].to_vec();

        // compute the overhead_time, service_distance and dead_head_distance for the new tour:
        let overhead_time = if start_pos == 0 || end_pos == self.nodes.len() - 1 {
            if tour_nodes.len() == 0 {
                Duration::zero()
            } else {
                let total_time_original = self.nw.node(*self.nodes.last().unwrap()).end_time()
                    - self.nw.node(*self.nodes.first().unwrap()).start_time();
                let total_time_new = self.nw.node(*tour_nodes.last().unwrap()).end_time()
                    - self.nw.node(*tour_nodes.first().unwrap()).start_time();
                self.overhead_time + removed_useful_time + total_time_new - total_time_original
            }
        } else {
            self.overhead_time + removed_useful_time
        };
        let service_distance = self.service_distance - removed_service_distance;
        let dead_head_distance =
            self.dead_head_distance + added_dead_head_distance - removed_dead_head_distance;

        Ok((
            Tour::new_precomputed(
                tour_nodes,
                self.is_dummy,
                overhead_time,
                service_distance,
                dead_head_distance,
                self.config.clone(),
                self.nw.clone(),
            ),
            Path::new_trusted(removed_nodes, self.nw.clone()),
        ))
    }

    pub(crate) fn sub_path(&self, segment: Segment) -> Result<Path, String> {
        let start_pos = self
            .earliest_not_reaching_node(segment.start())
            .ok_or_else(|| String::from("segment.start() not part of Tour."))?;
        if segment.start() != self.nodes[start_pos] {
            return Err(String::from("segment.start() not part of Tour."));
        }
        let end_pos = self
            .earliest_not_reaching_node(segment.end())
            .ok_or_else(|| String::from("segment.end() not part of Tour."))?;
        if segment.end() != self.nodes[end_pos] {
            return Err(String::from("segment.end() not part of Tour."));
        }

        Ok(Path::new_trusted(
            self.nodes[start_pos..end_pos + 1].to_vec(),
            self.nw.clone(),
        ))
    }
}

// private methods
impl Tour {
    pub(crate) fn position_of(&self, node: NodeId) -> Result<usize, String> {
        let pos = self
            .nodes
            .binary_search_by(|other| self.nw.node(*other).cmp_start_time(self.nw.node(node)))
            .map_err(|_| "Node not part of tour.")?;
        assert!(node == self.nodes[pos], "fehler"); // TODO remove
        Ok(pos)
    }
    /// Return the range of the conflicting nodes. start..end must be replaced if segment is
    /// inserted.
    /// That means start is the first conflicting node and end-1 is the last conflciting node.
    /// Node that if segment starts (or ends) with a depot it will be prefix (or suffix).
    fn get_insert_positions(&self, segment: Segment) -> (Position, Position) {
        let first = segment.start();
        let last = segment.end();

        let start_pos = if matches!(self.nw.node(first), Node::Depot(_)) {
            0
        } else {
            self.earliest_not_reaching_node(first)
                .unwrap_or(self.nodes.len())
        };

        let end_pos = if matches!(self.nw.node(last), Node::Depot(_)) {
            self.nodes.len()
        } else {
            match self.latest_not_reached_by_node(last) {
                None => 0,
                Some(p) => p + 1,
            }
        };
        (start_pos, end_pos)
    }

    fn earliest_not_reaching_node(&self, node: NodeId) -> Option<Position> {
        if self.nw.can_reach(*self.nodes.last().unwrap(), node) {
            return None; // all tour-nodes can reach node, even the last
        }
        let candidate =
            self.earliest_arrival_after(self.nw.node(node).start_time(), 0, self.nodes.len());
        let mut pos = candidate.unwrap_or(self.nodes.len() - 1);
        while pos > 0 && !self.nw.can_reach(self.nodes[pos - 1], node) {
            pos -= 1;
        }
        Some(pos)
    }

    fn earliest_arrival_after(
        &self,
        time: DateTime,
        left: Position,
        right: Position,
    ) -> Option<Position> {
        if left + 1 == right {
            if self.nw.node(self.nodes[left]).end_time() >= time {
                Some(left)
            } else {
                None
            }
        } else {
            let mid = left + (right - left) / 2;
            if self.nw.node(self.nodes[mid - 1]).end_time() >= time {
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
        let candidate =
            self.latest_departure_before(self.nw.node(node).end_time(), 0, self.nodes.len());
        // but later nodes might also not be reached by node.

        let mut pos = candidate.unwrap_or(0);
        while pos < self.nodes.len() - 1 && !self.nw.can_reach(node, self.nodes[pos + 1]) {
            pos += 1;
        }
        Some(pos)
    }

    fn latest_departure_before(
        &self,
        time: DateTime,
        left: Position,
        right: Position,
    ) -> Option<Position> {
        if left + 1 == right {
            if self.nw.node(self.nodes[left]).start_time() <= time {
                Some(left)
            } else {
                None
            }
        } else {
            let mid = left + (right - left) / 2;
            if self.nw.node(self.nodes[mid]).start_time() <= time {
                self.latest_departure_before(time, mid, right)
            } else {
                self.latest_departure_before(time, left, mid)
            }
        }
    }

    // the insertion could be invalid because of the following reasons (only if tour is non-dummy):
    // - the segment replaces the start depot and does not have a depot at the start itself
    // - the segment replaces the end depot and does not have a depot at the end itself
    fn test_if_valid_replacement(
        &self,
        segment: Segment,
        start_pos: Position,
        end_pos: Position,
    ) -> Result<(), String> {
        // test if start node makes sense:
        if start_pos == 0
            && !self.is_dummy
            && !matches!(self.nw.node(segment.start()), Node::Depot(_))
        {
            return Err(String::from(
                "Cannot replace start depot by a segment that does not start with a depot.",
            ));
        }
        // first test if end nodes make sense:
        if end_pos == self.nodes.len()
            && !self.is_dummy
            && !matches!(self.nw.node(segment.end()), Node::Depot(_))
        {
            return Err(String::from(
                "Cannot replace end depot by a segment that does not end with a depot.",
            ));
        }

        Ok(())
    }

    /// start_position is here the position of the first node that should be removed
    /// end_position is here the position in the tour of the last node that should be removed
    /// in other words [start_position .. end_position+1) is about to be removed
    fn test_if_sequence_is_removable(
        &self,
        start_position: usize,
        end_position: usize,
    ) -> Result<(), String> {
        if !self.is_dummy && start_position == 0 {
            return Err(String::from("Start depot cannot be removed."));
        }
        if !self.is_dummy && end_position == self.nodes.len() - 1 {
            return Err(String::from("End depot cannot be removed."));
        }
        if start_position > end_position {
            return Err(String::from("start_position comes after end_position."));
        }

        if start_position > 0
            && end_position < self.nodes.len() - 1
            && !self
                .nw
                .can_reach(self.nodes[start_position - 1], self.nodes[end_position + 1])
        {
            return Err(format!("Removing nodes ({} to {}) makes the tour invalid. Dead-head-trip is slower than service-trips.", self.nodes[start_position], self.nodes[end_position]));
        }
        Ok(())
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

// constructors
impl Tour {
    /// Creates a new tour from a vector of NodeIds. Checks that the tour is valid:
    /// * starts with a StartNode
    /// * end with an EndNode
    /// * only Service or MaintenanceNodes in the middle
    /// * each node can reach its successor
    /// If one of the checks fails an error message is returned.
    pub(super) fn new(
        nodes: Vec<NodeId>,
        config: Arc<Config>,
        nw: Arc<Network>,
    ) -> Result<Tour, String> {
        Tour::new_allow_invalid(nodes, config, nw).map_err(|(_, error_msg)| error_msg)
    }

    /// Creates a new tour from a vector of NodeIds. Checks that the tour is valid:
    /// * starts with a StartNode
    /// * end with an EndNode
    /// * only Service or MaintenanceNodes in the middle
    /// * each node can reach is successor
    /// If one of the checks fails an error is returned containing the error message but also the
    /// invalid tour.
    pub(super) fn new_allow_invalid(
        nodes: Vec<NodeId>,
        config: Arc<Config>,
        nw: Arc<Network>,
    ) -> Result<Tour, (Tour, String)> {
        let mut error_msg = String::new();
        if !matches!(nw.node(nodes[0]), Node::Depot(_)) {
            error_msg.push_str(&format!(
                "Tour needs to start with a depot, not with: {}.\n",
                nw.node(nodes[0])
            ));
        }
        if !matches!(nw.node(nodes[nodes.len() - 1]), Node::Depot(_)) {
            error_msg.push_str(&format!(
                "Tour needs to end with a depot, not with: {},\n",
                nw.node(nodes[nodes.len() - 1])
            ));
        }
        for i in 1..nodes.len() - 1 {
            let n = nw.node(nodes[i]);
            if !matches!(n, Node::Service(_)) && !matches!(n, Node::Maintenance(_)) {
                error_msg.push_str(&format!(
                    "Tour can only have Service or Maintenance Nodes in the middle, not: {}.\n",
                    n
                ));
            }
        }
        for (&a, &b) in nodes.iter().tuple_windows() {
            if !nw.can_reach(a, b) {
                error_msg.push_str(&format!(
                    "Not a valid Tour: {} cannot reach {}.\n",
                    nw.node(a),
                    nw.node(b)
                ));
            }
        }
        if error_msg.len() > 0 {
            Err((Tour::new_computing(nodes, false, config, nw), error_msg))
        } else {
            Ok(Tour::new_computing(nodes, false, config, nw))
        }
    }

    pub(super) fn new_dummy(
        nodes: Vec<NodeId>,
        config: Arc<Config>,
        nw: Arc<Network>,
    ) -> Result<Tour, String> {
        for (&a, &b) in nodes.iter().tuple_windows() {
            if !nw.can_reach(a, b) {
                return Err(format!(
                    "Not a valid Dummy-Tour: {} cannot reach {}.\n",
                    nw.node(a),
                    nw.node(b)
                ));
            }
        }
        Ok(Tour::new_computing(nodes, true, config, nw))
    }

    pub(super) fn new_dummy_by_path(path: Path, config: Arc<Config>, nw: Arc<Network>) -> Tour {
        Tour::new_computing(path.consume(), true, config, nw)
    }

    fn new_computing(
        nodes: Vec<NodeId>,
        is_dummy: bool,
        config: Arc<Config>,
        nw: Arc<Network>,
    ) -> Tour {
        let overhead_time = nodes
            .iter()
            .tuple_windows()
            .map(|(&a, &b)| nw.node(b).start_time() - nw.node(a).end_time())
            .sum();
        let service_distance = nodes.iter().map(|&n| nw.node(n).travel_distance()).sum();
        let dead_head_distance = nodes
            .iter()
            .tuple_windows()
            .map(|(&a, &b)| nw.dead_head_distance_between(a, b))
            .sum();

        Tour::new_precomputed(
            nodes,
            is_dummy,
            overhead_time,
            service_distance,
            dead_head_distance,
            config,
            nw,
        )
    }

    /// Creates a new tour from a vector of NodeIds. Trusts that the vector leads to a valid Tour.
    pub(super) fn new_precomputed(
        nodes: Vec<NodeId>,
        is_dummy: bool,
        overhead_time: Duration,
        service_distance: Distance,
        dead_head_distance: Distance,
        config: Arc<Config>,
        nw: Arc<Network>,
    ) -> Tour {
        Tour {
            nodes,
            is_dummy,
            overhead_time,
            service_distance,
            dead_head_distance,
            config,
            nw,
        }
    }
}

#[cfg(test)]
mod test {
    use sbb_model::{
        base_types::{Distance, Duration, NodeId},
        json_serialisation::load_rolling_stock_problem_instance_from_json,
    };

    use crate::schedule::path::Path;

    use super::Tour;
    #[test]
    fn tour_test() {
        // load file from json
        let (_, _, network, config) =
            load_rolling_stock_problem_instance_from_json("resources/test_instance.json");

        // create a tour
        let trip12 = NodeId::from("trip1-2");
        let trip23 = NodeId::from("trip2-3");
        let trip34 = NodeId::from("trip3-4");
        let trip45 = NodeId::from("trip4-5");
        let trip51 = NodeId::from("trip5-1");
        let trip31 = NodeId::from("trip3-1");
        let trip14 = NodeId::from("trip1-4");
        let depot1 = NodeId::from("depot1_vt1");
        let tour = Tour::new(
            vec![depot1, trip12, trip23, trip34, trip45, trip51, depot1],
            config.clone(),
            network.clone(),
        )
        .unwrap();

        assert_eq!(tour.is_dummy(), false);
        assert_eq!(tour.nodes.len(), 7);
        assert_eq!(tour.movable_nodes().count(), 5);
        assert_eq!(tour.overhead_time(), Duration::zero()); // TODO fix bug
        assert_eq!(tour.service_distance(), Distance::from_meter(15000));
        assert_eq!(tour.dead_head_distance(), Distance::zero());
        assert_eq!(tour.preceding_overhead(trip23), Duration::new("0:30"));
        assert_eq!(tour.subsequent_overhead(trip23), Duration::new("0:30"));
        assert_eq!(tour.first_node(), depot1);
        assert_eq!(tour.last_node(), depot1);
        assert_eq!(tour.nth_node(2), trip23);

        let path = Path::new(vec![trip31, trip14], network.clone());
        let dummy_tour = Tour::new_dummy_by_path(path, config.clone(), network.clone());
        assert_eq!(dummy_tour.is_dummy(), true);
        assert_eq!(dummy_tour.nodes.len(), 2);
        assert_eq!(dummy_tour.movable_nodes().count(), 2);
        assert_eq!(dummy_tour.overhead_time(), Duration::new("0:30"));
        assert_eq!(dummy_tour.service_distance(), Distance::from_meter(13000));
        assert_eq!(dummy_tour.dead_head_distance(), Distance::zero());
        assert_eq!(dummy_tour.preceding_overhead(trip31), Duration::Infinity);
        assert_eq!(
            dummy_tour.subsequent_overhead(trip31),
            Duration::new("0:30")
        );
        assert_eq!(dummy_tour.first_node(), trip31);
        assert_eq!(dummy_tour.last_node(), trip14);
        assert_eq!(dummy_tour.nth_node(1), trip14);
    }
}
