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

/// This represents a tour of a single vehicle (or a dummy tour). The following holds at all times:
///
/// The tour is a path in the network (implying that there are no intermediate depots).
/// Non-dummy tours always start at a StartDepot and ends at a EndDepot.
/// Dummy tours do not contain any depots.
/// Each tour contains at least on non-depot node.
///
/// It is an immutable objects. So whenever some modification is applied a copy of the tour
/// is created.
#[derive(Clone)]
pub(crate) struct Tour {
    nodes: Vec<NodeId>, // nodes will always be sorted by start_time; for non-dummy exactly first
    // and last node is a depot
    is_dummy: bool, // true if this is a dummy tour

    useful_duration: Duration, // duration of service trips and maintenance, excluding dead head and idle time
    service_distance: Distance, // distance covered by service trips
    dead_head_distance: Distance, // distance covered by dead head trips

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
    pub(crate) fn non_depot_nodes(&self) -> impl Iterator<Item = NodeId> + '_ {
        if self.is_dummy {
            self.nodes.iter().copied()
        } else {
            self.nodes[1..self.nodes.len() - 1].iter().copied()
        }
    }

    /// total dead-head distance traveled by the tour
    pub(crate) fn dead_head_distance(&self) -> Distance {
        self.dead_head_distance
    }

    /// return the overhead_time (dead head travel time + idle time) of the tour.
    pub(crate) fn useful_duration(&self) -> Duration {
        self.useful_duration
    }

    /// return the service distance (distance of service trips) of the tour
    pub(crate) fn service_distance(&self) -> Distance {
        self.service_distance
    }

    /// the overhead time (dead_head + idle) between the predecessor and the node itself
    /// for the first non-depot node, as well as a depot, the overhead time is set to be infinity.
    /// (this is to allow for splitting before the first non-depot node in all cases)
    pub(crate) fn preceding_overhead(&self, node: NodeId) -> Result<Duration, String> {
        if node == self.first_node() {
            Ok(Duration::Infinity)
        } else {
            let pos = self.position_of(node)?;
            let predecessor = *self.nodes.get(pos - 1).ok_or("invalid position")?;
            Ok(self.nw.node(node).start_time() - self.nw.node(predecessor).end_time())
        }
    }
    /// the overhead time (dead_head + idle) between the node itself and its successor
    /// for the last non-depot node, as well as a depot, the overhead time is set to be infinity.
    /// (this is to allow for splitting the tour after the last non-depot node in all cases)
    pub(crate) fn subsequent_overhead(&self, node: NodeId) -> Result<Duration, String> {
        if node == self.last_node() {
            Ok(Duration::Infinity)
        } else {
            let pos = self.position_of(node)?;
            let successor = *self.nodes.get(pos + 1).ok_or("invalid position")?;
            Ok(self.nw.node(successor).start_time() - self.nw.node(node).end_time())
        }
    }

    pub(crate) fn first_node(&self) -> NodeId {
        *self.nodes.first().unwrap()
    }

    pub(crate) fn last_node(&self) -> NodeId {
        *self.nodes.last().unwrap()
    }

    pub(crate) fn nth_node(&self, pos: usize) -> Option<NodeId> {
        self.nodes.get(pos).map(|n| *n)
    }

    pub(crate) fn start_time(&self) -> DateTime {
        if self.is_dummy {
            self.nw.node(self.first_node()).start_time()
        } else {
            self.nw.node(self.nth_node(1).unwrap()).start_time()
        }
    }

    pub(crate) fn end_time(&self) -> DateTime {
        if self.is_dummy {
            self.nw.node(self.last_node()).end_time()
        } else {
            self.nw
                .node(self.nth_node(self.len() - 2).unwrap())
                .end_time()
        }
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
    /// when the segment would have been inserted. These nodes form a path that is returned.
    /// If this path is empty (or consists only of depots) None is returned.
    /// Fails if the segment insertion would not lead  to a valid Tour (for example start node
    /// cannot reach segment.start(), or segment.end() cannot reach end node).
    pub(super) fn conflict(&self, segment: Segment) -> Result<Option<Path>, String> {
        let (start_pos, end_pos) = self.get_insert_positions(segment);

        self.test_if_valid_replacement(segment, start_pos, end_pos)?;

        Ok(Path::new_trusted(
            self.nodes[start_pos..end_pos].to_vec(),
            self.nw.clone(),
        ))
    }

    /// Returns the tour where the provided path is inserted into the correct position (time-wise). The path will
    /// stay uninterrupted, clashing nodes are removed and returned as new path (None if empty).
    ///
    /// # Properties:
    /// - Assumes that provided node sequence is feasible.
    /// - Dummy: If path contains depots (at the start or end), the depots are
    /// removed at the beginning.
    /// - Non-dummy: If the provided sequence contains a start depot it will be inserted as a prefix.
    /// - Non-dummy: If the provided path contains an end depot it will be inserted as a suffix.
    /// - Non-dummy: tours it fails if the start depot clashes and is not replaced with a new
    /// start depot. The same holds for the end depot.
    pub(super) fn insert(&self, path: Path) -> Result<(Tour, Option<Path>), String> {
        // remove depots from path if self.is_dummy=true.
        let mut path = path;
        if self.is_dummy {
            if self.nw.node(path.first()).is_depot() {
                path = path.drop_first().unwrap();
            }
            if self.nw.node(path.last()).is_depot() {
                path = path.drop_last().unwrap();
            }
        }

        // get insertion position and check if insertion is valid
        let segment = Segment::new(path.first(), path.last());
        let (start_pos, end_pos) = self.get_insert_positions(segment);
        self.test_if_valid_replacement(segment, start_pos, end_pos)?;

        // compute useful_duration, service_distance and dead_head_distance for the path:
        let path_useful_duration = path.iter().map(|n| self.nw.node(n).duration()).sum();
        let path_service_distance = path.iter().map(|n| self.nw.node(n).travel_distance()).sum();
        let path_dead_head_distance: Distance = if start_pos == 0 {
            Distance::zero()
        } else {
            self.nw
                .dead_head_distance_between(self.nodes[start_pos - 1], path.first())
        } + path
            .iter()
            .tuple_windows()
            .map(|(a, b)| self.nw.dead_head_distance_between(a, b))
            .sum()
            + if end_pos >= self.nodes.len() {
                Distance::zero()
            } else {
                self.nw
                    .dead_head_distance_between(path.last(), self.nodes[end_pos])
            };

        // compute useful_duration, service_distance and dead_head_distance for the segment that is
        // removed:
        let removed_useful_duration = (start_pos..end_pos)
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
        let removed_nodes: Vec<NodeId> = new_tour_nodes
            .splice(start_pos..end_pos, path.consume())
            .collect();

        // compute the useful_duration, service_distance and dead_head_distance for the new tour:
        let useful_duration = self.useful_duration + path_useful_duration - removed_useful_duration;
        let service_distance =
            self.service_distance + path_service_distance - removed_service_distance;
        let dead_head_distance =
            self.dead_head_distance + path_dead_head_distance - removed_dead_head_distance;

        Ok((
            Tour::new_precomputed(
                new_tour_nodes,
                self.is_dummy,
                useful_duration,
                service_distance,
                dead_head_distance,
                self.config.clone(),
                self.nw.clone(),
            ),
            Path::new_trusted(removed_nodes, self.nw.clone()),
        ))
    }

    /// checks whether segment can be removed from tour or not.
    pub(crate) fn test_removable(&self, segment: Segment) -> Result<(), String> {
        let start_pos = self.position_of(segment.start())?;
        let end_pos = self.position_of(segment.end())?;
        self.test_if_sequence_is_removable(start_pos, end_pos)
    }

    /// remove the segment of the tour. The subpath between segment.start() and segment.end() is removed and the new
    /// shortened Tour as well as the removed nodes (as Path) are returned.
    /// Fails if either segment.start() or segment.end() are not part of the Tour or if the start or end node would
    /// get removed but not both.
    /// In case that there is no non-depot left after the removel None is returned.
    pub(crate) fn remove(&self, segment: Segment) -> Result<(Option<Tour>, Path), String> {
        let start_pos = self.position_of(segment.start())?;
        let end_pos = self.position_of(segment.end())?;
        self.test_if_sequence_is_removable(start_pos, end_pos)?;

        // compute useful_duration, service_distance and dead_head_distance for the removed segment:
        let removed_useful_duration = (start_pos..end_pos + 1)
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
        if tour_nodes.is_empty() || (!self.is_dummy() && tour_nodes.len() <= 2) {
            return Ok((
                None,
                Path::new_trusted(removed_nodes, self.nw.clone())
                    .expect("empty path should be impossible."),
            ));
        }

        let useful_duration = self.useful_duration - removed_useful_duration;
        let service_distance = self.service_distance - removed_service_distance;
        let dead_head_distance =
            self.dead_head_distance + added_dead_head_distance - removed_dead_head_distance;

        Ok((
            Some(Tour::new_precomputed(
                tour_nodes,
                self.is_dummy,
                useful_duration,
                service_distance,
                dead_head_distance,
                self.config.clone(),
                self.nw.clone(),
            )),
            Path::new_trusted(removed_nodes, self.nw.clone())
                .expect("empty path should be impossible."),
        ))
    }

    pub(crate) fn sub_path(&self, segment: Segment) -> Result<Path, String> {
        // TODO: probably problem if segment starts or ends at depot
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
        if start_pos > end_pos {
            return Err(String::from("segment.start() is after segment.end()."));
        }

        Ok(
            Path::new_trusted(self.nodes[start_pos..end_pos + 1].to_vec(), self.nw.clone())
                .expect("segment is empty path."),
        )
    }
}

// private methods
impl Tour {
    pub(crate) fn position_of(&self, node: NodeId) -> Result<usize, String> {
        let pos = self
            .nodes
            .binary_search_by(|other| self.nw.node(*other).cmp_start_time(self.nw.node(node)))
            .map_err(|_| "Node not part of tour.")?;
        Ok(pos)
    }

    /// Return the range of the conflicting nodes. start..end must be replaced if segment is
    /// inserted.
    /// That means start is the first conflicting node and end-1 is the last conflciting node.
    /// Node that if segment starts (or ends) with a depot it will be prefix (or suffix).
    fn get_insert_positions(&self, segment: Segment) -> (Position, Position) {
        let first = segment.start();
        let last = segment.end();

        let start_pos = if self.nw.node(first).is_depot() {
            0
        } else {
            self.earliest_not_reaching_node(first)
                .unwrap_or(self.nodes.len())
        };

        let end_pos = if self.nw.node(last).is_depot() {
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
        if start_pos == 0 && !self.is_dummy && !self.nw.node(segment.start()).is_depot() {
            return Err(String::from(
                "Cannot replace start depot by a segment that does not start with a depot.",
            ));
        }
        // test if end nodes make sense:
        if end_pos == self.nodes.len() && !self.is_dummy && !self.nw.node(segment.end()).is_depot()
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
        if !self.is_dummy && start_position == 0 && end_position <= self.nodes.len() - 3 {
            return Err(String::from(
                "Start depot cannot be removed without removing all non-depots.",
            ));
        }
        if !self.is_dummy && end_position == self.nodes.len() - 1 && start_position >= 2 {
            return Err(String::from(
                "End depot cannot be removed without removing all non-depots.",
            ));
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
        if !nw.node(nodes[0]).is_start_depot() {
            error_msg.push_str(&format!(
                "Tour needs to start with a StartDepot, not with: {}.\n",
                nw.node(nodes[0])
            ));
        }
        if !nw.node(nodes[nodes.len() - 1]).is_end_depot() {
            error_msg.push_str(&format!(
                "Tour needs to end with a EndDepot, not with: {},\n",
                nw.node(nodes[nodes.len() - 1])
            ));
        }
        if nodes.len() < 3 {
            error_msg
                .push_str("Tour needs to have at least three nodes (at least one non-depot).\n");
        }
        for i in 1..nodes.len() - 1 {
            if nw.node(nodes[i]).is_depot() {
                error_msg.push_str(&format!(
                    "Tour can only have service or maintenance nodes in the middle, not: {}.\n",
                    nw.node(nodes[i])
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
        if nw.node(nodes[0]).is_depot() {
            return Err(format!(
                "Dummy-tour cannot start with a depot: {}.\n",
                nw.node(nodes[0])
            ));
        }
        if nw.node(nodes[nodes.len() - 1]).is_depot() {
            return Err(format!(
                "Dummy-tour cannot end with a depot, not with: {},\n",
                nw.node(nodes[nodes.len() - 1])
            ));
        }
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
        let mut nodes = path.consume();
        // remove start and end depot
        if nw.node(*nodes.first().unwrap()).is_depot() {
            nodes.remove(0);
        };
        if nw.node(*nodes.last().unwrap()).is_depot() {
            nodes.pop();
        };
        assert!(nodes.len() > 0);
        Tour::new_computing(nodes, true, config, nw)
    }

    fn new_computing(
        nodes: Vec<NodeId>,
        is_dummy: bool,
        config: Arc<Config>,
        nw: Arc<Network>,
    ) -> Tour {
        let useful_duration = nodes
            .iter()
            .map(|&n| nw.node(n).duration())
            .sum::<Duration>();
        let service_distance = nodes.iter().map(|&n| nw.node(n).travel_distance()).sum();
        let dead_head_distance = nodes
            .iter()
            .tuple_windows()
            .map(|(&a, &b)| nw.dead_head_distance_between(a, b))
            .sum();

        Tour::new_precomputed(
            nodes,
            is_dummy,
            useful_duration,
            service_distance,
            dead_head_distance,
            config,
            nw,
        )
    }

    /// Creates a new tour from a vector of NodeIds. Trusts that the vector leads to a valid Tour.
    fn new_precomputed(
        nodes: Vec<NodeId>,
        is_dummy: bool,
        useful_duration: Duration,
        service_distance: Distance,
        dead_head_distance: Distance,
        config: Arc<Config>,
        nw: Arc<Network>,
    ) -> Tour {
        Tour {
            nodes,
            is_dummy,
            useful_duration,
            service_distance,
            dead_head_distance,
            config,
            nw,
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use sbb_model::{
        base_types::{DateTime, Distance, Duration, NodeId},
        config::Config,
        json_serialisation::load_rolling_stock_problem_instance_from_json,
        network::Network,
    };

    use crate::schedule::path::{Path, Segment};

    use super::Tour;

    struct TestData {
        network: Arc<Network>,
        config: Arc<Config>,
        trip12: NodeId,
        trip23: NodeId,
        trip34: NodeId,
        trip45: NodeId,
        trip51: NodeId,
        trip31: NodeId,
        trip14: NodeId,
        start_depot1: NodeId,
        end_depot1: NodeId,
    }

    fn init_test_data() -> TestData {
        // load file from json
        let (_, _, network, config) =
            load_rolling_stock_problem_instance_from_json("resources/test_instance.json");
        TestData {
            network,
            config,
            trip12: NodeId::from("trip1-2"),
            trip23: NodeId::from("trip2-3"),
            trip34: NodeId::from("trip3-4"),
            trip45: NodeId::from("trip4-5"),
            trip51: NodeId::from("trip5-1"),
            trip31: NodeId::from("trip3-1"),
            trip14: NodeId::from("trip1-4"),
            start_depot1: NodeId::from("start_depot1_vt1"),
            end_depot1: NodeId::from("end_depot1_vt1"),
        }
    }

    fn default_tour(d: &TestData) -> Tour {
        Tour::new(
            vec![
                d.start_depot1,
                d.trip12,
                d.trip23,
                d.trip34,
                d.trip45,
                d.trip51,
                d.end_depot1,
            ],
            d.config.clone(),
            d.network.clone(),
        )
        .unwrap()
    }

    fn default_path(d: &TestData) -> Path {
        Path::new(vec![d.trip31, d.trip14], d.network.clone())
            .unwrap()
            .unwrap()
    }

    #[test]
    fn basic_methods_test() {
        // ARRANGE
        let d = init_test_data();
        let thirty_minutes = Duration::new("0:30");

        // ACT
        let tour = default_tour(&d);
        let path = default_path(&d);
        let dummy_tour = Tour::new_dummy_by_path(path, d.config.clone(), d.network.clone());

        // ASSERT
        assert_eq!(tour.is_dummy(), false);
        assert_eq!(tour.nodes.len(), 7);
        assert_eq!(tour.non_depot_nodes().count(), 5);
        assert_eq!(tour.useful_duration(), Duration::new("2:30"));
        assert_eq!(tour.service_distance(), Distance::from_meter(15000));
        assert_eq!(tour.dead_head_distance(), Distance::zero());
        assert_eq!(
            tour.preceding_overhead(d.start_depot1),
            Ok(Duration::Infinity)
        );
        assert_eq!(
            tour.subsequent_overhead(d.start_depot1),
            Ok(Duration::Infinity)
        );
        assert_eq!(tour.preceding_overhead(d.trip12), Ok(Duration::Infinity));
        assert_eq!(
            tour.subsequent_overhead(d.trip12),
            Ok(Duration::new("0:30"))
        );
        assert_eq!(tour.preceding_overhead(d.trip23), Ok(thirty_minutes));
        assert_eq!(
            tour.subsequent_overhead(d.trip23),
            Ok(Duration::new("0:30"))
        );
        assert_eq!(tour.preceding_overhead(d.trip34), Ok(thirty_minutes));
        assert_eq!(
            tour.subsequent_overhead(d.trip34),
            Ok(Duration::new("0:30"))
        );
        assert_eq!(tour.preceding_overhead(d.trip45), Ok(thirty_minutes));
        assert_eq!(
            tour.subsequent_overhead(d.trip45),
            Ok(Duration::new("0:30"))
        );
        assert_eq!(tour.preceding_overhead(d.trip51), Ok(thirty_minutes));
        assert_eq!(tour.subsequent_overhead(d.trip51), Ok(Duration::Infinity));
        assert_eq!(
            tour.preceding_overhead(d.end_depot1),
            Ok(Duration::Infinity)
        );
        assert_eq!(
            tour.subsequent_overhead(d.end_depot1),
            Ok(Duration::Infinity)
        );
        assert!(tour.preceding_overhead(d.trip31).is_err());
        assert!(tour.subsequent_overhead(d.trip31).is_err());
        assert_eq!(tour.first_node(), d.start_depot1);
        assert_eq!(tour.last_node(), d.end_depot1);
        assert_eq!(tour.nth_node(0), Some(d.start_depot1));
        assert_eq!(tour.nth_node(1), Some(d.trip12));
        assert_eq!(tour.nth_node(2), Some(d.trip23));
        assert_eq!(tour.nth_node(3), Some(d.trip34));
        assert_eq!(tour.nth_node(4), Some(d.trip45));
        assert_eq!(tour.nth_node(5), Some(d.trip51));
        assert_eq!(tour.nth_node(6), Some(d.end_depot1));
        assert_eq!(tour.nth_node(7), None);
        assert_eq!(tour.start_time(), DateTime::new("2020-01-01T06:00"));
        assert_eq!(tour.end_time(), DateTime::new("2020-01-01T10:30"));

        assert_eq!(dummy_tour.is_dummy(), true);
        assert_eq!(dummy_tour.nodes.len(), 2);
        assert_eq!(dummy_tour.non_depot_nodes().count(), 2);
        assert_eq!(dummy_tour.useful_duration(), Duration::new("1:00"));
        assert_eq!(dummy_tour.service_distance(), Distance::from_meter(13000));
        assert_eq!(dummy_tour.dead_head_distance(), Distance::zero());
        assert_eq!(
            dummy_tour.preceding_overhead(d.trip31),
            Ok(Duration::Infinity)
        );
        assert_eq!(dummy_tour.subsequent_overhead(d.trip31), Ok(thirty_minutes));
        assert_eq!(dummy_tour.preceding_overhead(d.trip14), Ok(thirty_minutes));
        assert_eq!(
            dummy_tour.subsequent_overhead(d.trip14),
            Ok(Duration::Infinity)
        );
        assert_eq!(dummy_tour.first_node(), d.trip31);
        assert_eq!(dummy_tour.last_node(), d.trip14);
        assert_eq!(dummy_tour.nth_node(0), Some(d.trip31));
        assert_eq!(dummy_tour.nth_node(1), Some(d.trip14));
        assert_eq!(dummy_tour.nth_node(2), None);
        assert_eq!(dummy_tour.start_time(), DateTime::new("2020-01-01T08:00"));
        assert_eq!(dummy_tour.end_time(), DateTime::new("2020-01-01T09:30"));
    }

    #[test]
    fn conflict_test() {
        // ARRANGE
        let d = init_test_data();
        let tour = default_tour(&d);
        let segment = Segment::new(d.trip31, d.trip14);

        // ACT
        let conflicted_path = tour.conflict(segment);

        // ASSERT
        assert!(conflicted_path.is_ok());
        let unwrapped_path = conflicted_path.unwrap().unwrap();
        assert_iter(unwrapped_path.iter(), vec![d.trip34, d.trip45, d.trip51]);
    }
    #[test]
    fn insert_test() {
        // ARRANGE
        let d = init_test_data();
        let tour = default_tour(&d);
        let path = default_path(&d);

        // ACT
        let insert_result = tour.insert(path);

        // ASSERT
        assert!(insert_result.is_ok());
        let (new_tour, removed_path_option) = insert_result.unwrap();
        assert_iter(
            new_tour.all_nodes_iter(),
            vec![
                d.start_depot1,
                d.trip12,
                d.trip23,
                d.trip31,
                d.trip14,
                d.end_depot1,
            ],
        );

        let removed_path = removed_path_option.unwrap();
        assert_iter(removed_path.iter(), vec![d.trip34, d.trip45, d.trip51]);
    }

    #[test]
    fn removable_test() {
        // ARRANGE
        let d = init_test_data();
        let tour = default_tour(&d);
        let segment_for_removal = Segment::new(d.trip12, d.trip45);

        // ACT
        let removable_result = tour.test_removable(segment_for_removal);

        // ASSERT
        assert!(removable_result.is_ok());
    }

    #[test]
    fn remove_test() {
        // ARRANGE
        let d = init_test_data();
        let tour = default_tour(&d);
        let segment_for_removal = Segment::new(d.trip12, d.trip45);

        // ACT
        let remove_result = tour.remove(segment_for_removal);

        // ASSERT
        assert!(remove_result.is_ok());
        let (new_tour_option, removed_path) = remove_result.unwrap();
        let new_tour = new_tour_option.unwrap();
        assert_iter(
            new_tour.all_nodes_iter(),
            vec![d.start_depot1, d.trip51, d.end_depot1],
        );

        assert_iter(
            removed_path.iter(),
            vec![d.trip12, d.trip23, d.trip34, d.trip45],
        );
    }

    #[test]
    fn sub_path_test() {
        // ARRANGE

        let d = init_test_data();
        let tour = default_tour(&d);
        let segment = Segment::new(d.trip12, d.trip45);

        // ACT
        let sub_path_result = tour.sub_path(segment);

        // ASSERT
        assert!(sub_path_result.is_ok());
        let sub_path = sub_path_result.unwrap();
        assert_iter(
            sub_path.iter(),
            vec![d.trip12, d.trip23, d.trip34, d.trip45],
        );
    }

    #[test]
    fn remove_all_nodes_test() {
        // ARRANGE
        let d = init_test_data();
        let tour = default_tour(&d);
        let all_inner_nodes_segment = Segment::new(d.trip12, d.trip51);
        let start_depot_segment = Segment::new(d.start_depot1, d.trip51);
        let end_depot_segment = Segment::new(d.trip12, d.end_depot1);
        let all_nodes_segment = Segment::new(d.start_depot1, d.end_depot1);

        // ACT
        let all_inner_nodes_result = tour.remove(all_inner_nodes_segment);
        let start_depot_result = tour.remove(start_depot_segment);
        let end_depot_result = tour.remove(end_depot_segment);
        let all_nodes_result = tour.remove(all_nodes_segment);

        // ASSERT
        assert!(all_inner_nodes_result.is_ok());
        let (new_tour_option, removed_path) = all_inner_nodes_result.unwrap();
        assert!(new_tour_option.is_none());
        assert_iter(
            removed_path.iter(),
            vec![d.trip12, d.trip23, d.trip34, d.trip45, d.trip51],
        );

        assert!(start_depot_result.is_ok());
        let (new_tour_option, removed_path) = start_depot_result.unwrap();
        assert!(new_tour_option.is_none());
        assert_iter(
            removed_path.iter(),
            vec![
                d.start_depot1,
                d.trip12,
                d.trip23,
                d.trip34,
                d.trip45,
                d.trip51,
            ],
        );

        assert!(end_depot_result.is_ok());
        let (new_tour_option, removed_path) = end_depot_result.unwrap();
        assert!(new_tour_option.is_none());
        assert_iter(
            removed_path.iter(),
            vec![
                d.trip12,
                d.trip23,
                d.trip34,
                d.trip45,
                d.trip51,
                d.end_depot1,
            ],
        );

        assert!(all_nodes_result.is_ok());
        let (new_tour_option, removed_path) = all_nodes_result.unwrap();
        assert!(new_tour_option.is_none());
        assert_iter(
            removed_path.iter(),
            vec![
                d.start_depot1,
                d.trip12,
                d.trip23,
                d.trip34,
                d.trip45,
                d.trip51,
                d.end_depot1,
            ],
        );
    }

    #[test]
    fn remove_single_node_test() {
        // ARRANGE
        let d = init_test_data();
        let tour = default_tour(&d);
        let segment_for_removal = Segment::new(d.trip12, d.trip12);

        // ACT
        let remove_result = tour.remove(segment_for_removal);

        // ASSERT
        assert!(remove_result.is_ok());
        let (new_tour_option, removed_path) = remove_result.unwrap();
        assert_iter(
            new_tour_option.unwrap().all_nodes_iter(),
            vec![
                d.start_depot1,
                d.trip23,
                d.trip34,
                d.trip45,
                d.trip51,
                d.end_depot1,
            ],
        );
        assert_iter(removed_path.iter(), vec![d.trip12]);
    }

    #[test]
    fn remove_invalid_segment() {
        // ARRANGE
        let d = init_test_data();
        let tour = default_tour(&d);
        let reversed_segment = Segment::new(d.trip23, d.trip12);
        let start_invalid_segment = Segment::new(d.trip31, d.trip45);
        let end_invalid_segment = Segment::new(d.trip12, d.trip14);
        let start_depot_segment = Segment::new(d.start_depot1, d.start_depot1);
        let end_depot_segment = Segment::new(d.end_depot1, d.end_depot1);

        // ACT
        let reversed_result = tour.remove(reversed_segment);
        let start_invalid_result = tour.remove(start_invalid_segment);
        let end_invalid_result = tour.remove(end_invalid_segment);
        let start_depot_result = tour.remove(start_depot_segment);
        let end_depot_result = tour.remove(end_depot_segment);

        // ASSERT
        assert!(reversed_result.is_err());
        assert!(start_invalid_result.is_err());
        assert!(end_invalid_result.is_err());
        assert!(start_depot_result.is_err());
        assert!(end_depot_result.is_err());
    }

    // TODO Test
    // - if start depot is replaced
    // - if end depot is replaced
    // - error if end depot cannot be reached

    fn assert_iter(mut iter: impl Iterator<Item = NodeId>, expected: Vec<NodeId>) {
        for expected_node in expected {
            assert_eq!(iter.next(), Some(expected_node));
        }
        assert_eq!(iter.next(), None);
    }
}
