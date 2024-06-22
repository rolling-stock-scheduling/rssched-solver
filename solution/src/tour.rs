mod modifications;
#[cfg(test)]
mod tests;
use crate::path::Path;
use crate::segment::Segment;
use model::base_types::{Cost, Distance, MaintenanceCounter, NodeIdx, INF_DISTANCE};
use model::network::nodes::Node;
use model::network::Network;
use std::cmp::Ordering;
use std::fmt;
use time::{DateTime, Duration};

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
/// Note that tours do not care for vehicle types. So the vehicle types of the depots might not
/// match.
///
/// It is an immutable objects. So whenever some modification is applied a copy of the tour
/// is created.
#[derive(Clone)]
pub struct Tour {
    nodes: Vec<NodeIdx>, // nodes will always be sorted by start_time; for non-dummy exactly first
    // and last node is a depot
    is_dummy: bool, // true if this is a dummy tour

    visits_maintenance: bool,  // true if this tour visits a maintenance node
    useful_duration: Duration, // duration of service trips and maintenance, excluding dead head and idle time
    service_distance: Distance, // distance covered by service trips
    dead_head_distance: Distance, // distance covered by dead head trips
    // cost = service_trip_duration * costs.service_trip
    // + maintenance_time * costs.maintenance
    // + dead_head_trip_duration * costs.dead_head_trip
    // + idle_time * costs.idle
    costs: Cost,
    network: Arc<Network>,
}

// basic public methods
impl Tour {
    pub fn is_dummy(&self) -> bool {
        self.is_dummy
    }

    pub fn visits_maintenance(&self) -> bool {
        self.visits_maintenance
    }

    pub fn length(&self) -> usize {
        self.nodes.len()
    }

    pub fn all_nodes_iter(&self) -> impl Iterator<Item = NodeIdx> + '_ {
        self.nodes.iter().copied()
    }

    /// return an iterator over all nodes (by start time) skipping the depot at the start and end
    pub fn all_non_depot_nodes_iter(&self) -> impl Iterator<Item = NodeIdx> + '_ {
        if self.is_dummy {
            self.nodes.iter().copied()
        } else {
            self.nodes[1..self.nodes.len() - 1].iter().copied()
        }
    }

    /// total dead-head distance traveled by the tour
    pub fn dead_head_distance(&self) -> Distance {
        self.dead_head_distance
    }

    /// total useful duration of the tour (service time + maintenance time)
    pub fn useful_duration(&self) -> Duration {
        self.useful_duration
    }

    /// return the service distance (distance of service trips) of the tour
    pub fn service_distance(&self) -> Distance {
        self.service_distance
    }

    /// return the total distance (service distance + dead head distance) of the tour
    pub fn total_distance(&self) -> Distance {
        self.service_distance + self.dead_head_distance
    }

    /// return the maintenance counter of the tour which is the total distance traveled minus the
    /// maximal distance allowed if the tour visits a maintenance node.
    pub fn maintenance_counter(&self) -> MaintenanceCounter {
        if self.visits_maintenance {
            self.total_distance().in_meter().unwrap_or(INF_DISTANCE) as MaintenanceCounter
                - self
                    .network
                    .config()
                    .maintenance
                    .maximal_distance
                    .in_meter()
                    .unwrap_or(INF_DISTANCE) as MaintenanceCounter
        } else {
            self.total_distance().in_meter().unwrap_or(INF_DISTANCE) as MaintenanceCounter
        }
    }

    pub fn costs(&self) -> Cost {
        self.costs
    }

    /// the overhead time (dead_head + idle) between the predecessor and the node itself
    /// for the first non-depot node, as well as a depot, the overhead time is set to be infinity.
    /// (this is to allow for splitting before the first non-depot node in all cases)
    pub fn preceding_overhead(&self, node: NodeIdx) -> Result<Duration, String> {
        if node == self.first_node() {
            Ok(Duration::Infinity)
        } else {
            let pos = self.position_of(node)?;
            let predecessor = *self.nodes.get(pos - 1).ok_or("invalid position")?;
            Ok(self.network.node(node).start_time() - self.network.node(predecessor).end_time())
        }
    }
    /// the overhead time (dead_head + idle) between the node itself and its successor
    /// for the last non-depot node, as well as a depot, the overhead time is set to be infinity.
    /// (this is to allow for splitting the tour after the last non-depot node in all cases)
    pub fn subsequent_overhead(&self, node: NodeIdx) -> Result<Duration, String> {
        if node == self.last_node() {
            Ok(Duration::Infinity)
        } else {
            let pos = self.position_of(node)?;
            let successor = *self.nodes.get(pos + 1).ok_or("invalid position")?;
            Ok(self.network.node(successor).start_time() - self.network.node(node).end_time())
        }
    }

    /// total overhead duration (dead_head + idle) of the tour
    pub fn total_overhead_duration(&self) -> Duration {
        self.end_time() - self.start_time() - self.useful_duration()
    }

    pub fn first_node(&self) -> NodeIdx {
        *self.nodes.first().unwrap()
    }

    pub fn last_node(&self) -> NodeIdx {
        *self.nodes.last().unwrap()
    }

    pub fn nth_node(&self, pos: Position) -> Option<NodeIdx> {
        self.nodes.get(pos).copied()
    }

    pub fn first_non_depot(&self) -> Option<NodeIdx> {
        self.all_non_depot_nodes_iter().next()
    }

    /// returns the last non-depot (service node or maintenance node) of the tour, ignoring depot.
    /// If the tour does only contain depots None is returned.
    pub fn last_non_depot(&self) -> Option<NodeIdx> {
        self.nodes
            .iter()
            .rev()
            .find(|&&n| !self.network.node(n).is_depot())
            .copied()
    }

    pub fn start_depot(&self) -> Result<NodeIdx, String> {
        if self.network.node(self.first_node()).is_start_depot() {
            Ok(self.first_node())
        } else {
            Err("tour does not have a start depot.".to_string())
        }
    }

    pub fn end_depot(&self) -> Result<NodeIdx, String> {
        if self.network.node(self.last_node()).is_end_depot() {
            Ok(self.last_node())
        } else {
            Err("tour does not have an end depot.".to_string())
        }
    }

    pub fn start_time(&self) -> DateTime {
        if self.is_dummy {
            self.network.node(self.first_node()).start_time()
        } else {
            // take start time of first non-depot node and subtract time needed to reach it from
            // the start depot
            self.network.node(self.nth_node(1).unwrap()).start_time()
                - self
                    .network
                    .dead_head_time_between(self.first_node(), self.nth_node(1).unwrap())
        }
    }

    pub fn end_time(&self) -> DateTime {
        if self.is_dummy {
            self.network.node(self.last_node()).end_time()
        } else {
            self.network
                .node(self.nth_node(self.length() - 2).unwrap())
                .end_time()
                + self.network.dead_head_time_between(
                    self.nth_node(self.length() - 2).unwrap(),
                    self.last_node(),
                )
        }
    }

    /// checks whether segment can be removed from tour or not.
    pub fn check_removable(&self, segment: Segment) -> Result<(), String> {
        let start_pos = self.position_of(segment.start())?;
        let end_pos = self.position_of(segment.end())?;
        self.check_if_sequence_is_removable(start_pos, end_pos)
    }

    /// return the position of the node in the tour that is the latest one that cannot reach the
    /// provided node.
    /// If all nodes can reach the provided node, None is returned.
    pub fn latest_not_reaching_node(&self, node: NodeIdx) -> Option<Position> {
        if self.network.can_reach(*self.nodes.last().unwrap(), node) {
            return None; // all tour-nodes can reach node, even the last
        }
        let candidate =
            self.earliest_arrival_after(self.network.node(node).start_time(), 0, self.nodes.len());
        let mut pos = candidate.unwrap_or(self.nodes.len() - 1);
        while pos > 0 && !self.network.can_reach(self.nodes[pos - 1], node) {
            pos -= 1;
        }
        Some(pos)
    }

    /// For a given segment (in general of another tour) returns all nodes that are conflicting
    /// when the segment would have been inserted. These nodes form a path that is returned.
    /// If this path is empty (or consists only of depots) None is returned.
    /// Fails if the segment insertion would not lead  to a valid Tour (for example start node
    /// cannot reach segment.start(), or segment.end() cannot reach end node).
    pub fn conflict(&self, segment: Segment) -> Option<Path> {
        let (start_pos, end_pos) = self.get_insert_positions(segment);
        Path::new_trusted(
            self.nodes[start_pos..end_pos].to_vec(),
            self.network.clone(),
        )
    }

    /// Return the path given by the segment.
    pub fn sub_path(&self, segment: Segment) -> Result<Path, String> {
        let start_pos = self
            .latest_not_reaching_node(segment.start())
            .ok_or_else(|| String::from("segment.start() not part of Tour."))?;
        if segment.start() != self.nodes[start_pos] {
            return Err(String::from("segment.start() not part of Tour."));
        }
        let end_pos = self
            .latest_not_reaching_node(segment.end())
            .ok_or_else(|| String::from("segment.end() not part of Tour."))?;
        if segment.end() != self.nodes[end_pos] {
            return Err(String::from("segment.end() not part of Tour."));
        }
        if start_pos > end_pos {
            return Err(String::from("segment.start() is after segment.end()."));
        }

        Ok(Path::new_trusted(
            self.nodes[start_pos..end_pos + 1].to_vec(),
            self.network.clone(),
        )
        .expect("segment is empty path."))
    }

    pub fn print(&self) {
        println!(
            "{}tour with {} nodes:",
            if self.is_dummy { "dummy-" } else { "" },
            self.nodes.len(),
        );
        for node in self.nodes.iter() {
            print!("\t* ");
            self.network.node(*node).print();
        }
    }

    pub fn verify_consistency(&self) {
        // check reachability
        for (node1, node2) in self.nodes.iter().tuple_windows() {
            assert!(self.network.can_reach(*node1, *node2));
        }

        // check if non-dummy tour starts and ends with depots
        if !self.is_dummy {
            assert!(self.network.node(self.first_node()).is_start_depot());
            assert!(self.network.node(self.last_node()).is_end_depot());
        }

        // check if visits maintenance
        assert_eq!(
            self.nodes
                .iter()
                .any(|&n| self.network.node(n).is_maintenance()),
            self.visits_maintenance
        );

        // check useful_duration
        assert_eq!(self.compute_useful_duration(), self.useful_duration);

        // check service_distance
        assert_eq!(self.compute_service_distance(), self.service_distance);

        // check dead_head_distance
        assert_eq!(self.compute_dead_head_distance(), self.dead_head_distance,);

        // check costs
        assert_eq!(self.compute_costs(), self.costs);
    }
}

// private methods
impl Tour {
    fn position_of(&self, node: NodeIdx) -> Result<Position, String> {
        let pos = self
            .nodes
            .binary_search_by(|other| {
                self.network
                    .node(*other)
                    .cmp_start_time(self.network.node(node))
            })
            .map_err(|_| "Node not part of tour.")?;
        Ok(pos)
    }

    /// start_position is here the position of the first node that should be removed
    /// end_position is here the position in the tour of the last node that should be removed
    /// in other words [start_position .. end_position+1) is about to be removed
    fn check_if_sequence_is_removable(
        &self,
        start_position: Position,
        end_position: Position,
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
                .network
                .can_reach(self.nodes[start_position - 1], self.nodes[end_position + 1])
        {
            return Err(format!("Removing nodes ({} to {}) makes the tour invalid. Dead-head-trip is slower than service-trips.", self.nodes[start_position], self.nodes[end_position]));
        }
        Ok(())
    }

    /// Return the range of the conflicting nodes. start..end must be replaced if segment is
    /// inserted.
    /// That means start is the first conflicting node and end-1 is the last conflciting node.
    /// Node that if segment starts (or ends) with a depot it will be prefix (or suffix).
    fn get_insert_positions(&self, segment: Segment) -> (Position, Position) {
        let first = segment.start();
        let last = segment.end();

        let start_pos = if self.network.node(first).is_depot() {
            0
        } else {
            self.latest_not_reaching_node(first)
                .unwrap_or(self.nodes.len())
        };

        let end_pos = if self.network.node(last).is_depot() {
            self.nodes.len()
        } else {
            match self.latest_not_reached_by_node(last) {
                None => 0,
                Some(p) => p + 1,
            }
        };
        (start_pos, end_pos)
    }

    /// computes the position of the latest tour-node that is not reached by node.
    /// if node can reach all tour-nodes, None is returned.
    fn latest_not_reached_by_node(&self, node: NodeIdx) -> Option<Position> {
        if self.network.can_reach(node, *self.nodes.first().unwrap()) {
            return None; // node can reach all nodes, even the first
        }
        // the candidate cannot be reached by node for sure.
        let candidate =
            self.latest_departure_before(self.network.node(node).end_time(), 0, self.nodes.len());
        // but later nodes might also not be reached by node.

        let mut pos = candidate.unwrap_or(0);
        while pos < self.nodes.len() - 1 && !self.network.can_reach(node, self.nodes[pos + 1]) {
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
            if self.network.node(self.nodes[left]).start_time() <= time {
                Some(left)
            } else {
                None
            }
        } else {
            let mid = left + (right - left) / 2;
            if self.network.node(self.nodes[mid]).start_time() <= time {
                self.latest_departure_before(time, mid, right)
            } else {
                self.latest_departure_before(time, left, mid)
            }
        }
    }

    fn earliest_arrival_after(
        &self,
        time: DateTime,
        left: Position,
        right: Position,
    ) -> Option<Position> {
        if left + 1 == right {
            if self.network.node(self.nodes[left]).end_time() >= time {
                Some(left)
            } else {
                None
            }
        } else {
            let mid = left + (right - left) / 2;
            if self.network.node(self.nodes[mid - 1]).end_time() >= time {
                self.earliest_arrival_after(time, left, mid)
            } else {
                self.earliest_arrival_after(time, mid, right)
            }
        }
    }

    fn compute_useful_duration(&self) -> Duration {
        Tour::compute_useful_duration_of_nodes(&self.nodes, &self.network)
    }

    fn compute_service_distance(&self) -> Distance {
        Tour::compute_service_distance_of_nodes(&self.nodes, &self.network)
    }

    fn compute_dead_head_distance(&self) -> Distance {
        Tour::compute_dead_head_distance_of_nodes(&self.nodes, &self.network)
    }

    fn compute_costs(&self) -> Cost {
        Self::compute_costs_of_nodes(&self.nodes, &self.network)
    }
}

// one tour is bigger than the other if the number of nodes are bigger.
// Ties are broken by comparing nodes from start to finish.
// Nodes ar compared by start time (ties are by end_time then id)
impl Ord for Tour {
    fn cmp(&self, other: &Self) -> Ordering {
        self.nodes.len().cmp(&other.nodes.len()).then(
            match self
                .nodes
                .iter()
                .zip(other.nodes.iter())
                .map(|(node, other_node)| {
                    self.network
                        .node(*node)
                        .cmp_start_time(self.network.node(*other_node))
                })
                .find(|ord| *ord != Ordering::Equal)
            {
                None => Ordering::Equal,
                Some(other) => other,
            },
        )
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

impl fmt::Display for Tour {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // write!(f, "{}", self.nodes.iter().map(|n| self.network.node(*n).to_string()).join(" - "))?;
        write!(
            f,
            "{}",
            self.nodes.iter().map(|n| self.network.node(*n)).join(" - ")
        )?;
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
    pub(super) fn new(nodes: Vec<NodeIdx>, network: Arc<Network>) -> Result<Tour, String> {
        Tour::new_allow_invalid(nodes, network).map_err(|(_, error_msg)| error_msg)
    }

    /// Creates a new tour from a vector of NodeIds. Checks that the tour is valid:
    /// * starts with a StartNode
    /// * end with an EndNode
    /// * only Service or MaintenanceNodes in the middle
    /// * each node can reach is successor
    /// If one of the checks fails an error is returned containing the error message but also the
    /// invalid tour.
    pub(super) fn new_allow_invalid(
        nodes: Vec<NodeIdx>,
        network: Arc<Network>,
    ) -> Result<Tour, (Tour, String)> {
        let mut error_msg = String::new();
        if !network.node(nodes[0]).is_start_depot() {
            error_msg.push_str(&format!(
                "Tour needs to start with a StartDepot, not with: {}.\n",
                network.node(nodes[0])
            ));
        }
        if !network.node(nodes[nodes.len() - 1]).is_end_depot() {
            error_msg.push_str(&format!(
                "Tour needs to end with a EndDepot, not with: {},\n",
                network.node(nodes[nodes.len() - 1])
            ));
        }
        if nodes.len() < 3 {
            error_msg
                .push_str("Tour needs to have at least three nodes (at least one non-depot).\n");
        }
        for node in nodes.iter().take(nodes.len() - 1).skip(1) {
            if network.node(*node).is_depot() {
                error_msg.push_str(&format!(
                    "Tour can only have service or maintenance nodes in the middle, not: {}.\n",
                    network.node(*node)
                ));
            }
        }
        for (&a, &b) in nodes.iter().tuple_windows() {
            if !network.can_reach(a, b) {
                error_msg.push_str(&format!(
                    "Not a valid Tour: {} cannot reach {}.\n",
                    network.node(a),
                    network.node(b)
                ));
            }
        }
        if !error_msg.is_empty() {
            Err((Tour::new_computing(nodes, false, network), error_msg))
        } else {
            Ok(Tour::new_computing(nodes, false, network))
        }
    }

    pub(super) fn new_dummy(path: Path, network: Arc<Network>) -> Result<Tour, String> {
        let mut nodes = path.consume();
        // remove non-service nodes
        nodes.retain(|&n| network.node(n).is_service());
        if nodes.is_empty() {
            return Err("Dummy tour needs to have at least one service nodes.".to_string());
        }
        Ok(Tour::new_computing(nodes, true, network))
    }

    fn new_computing(nodes: Vec<NodeIdx>, is_dummy: bool, network: Arc<Network>) -> Tour {
        let useful_duration = Tour::compute_useful_duration_of_nodes(&nodes, &network);
        let service_distance = Tour::compute_service_distance_of_nodes(&nodes, &network);
        let dead_head_distance = Tour::compute_dead_head_distance_of_nodes(&nodes, &network);
        let costs = Tour::compute_costs_of_nodes(&nodes, &network);
        let visits_maintenance = Tour::compute_visits_maintenance(&nodes, &network);

        Tour::new_precomputed(
            nodes,
            is_dummy,
            visits_maintenance,
            useful_duration,
            service_distance,
            dead_head_distance,
            costs,
            network,
        )
    }

    fn compute_service_distance_of_nodes(nodes: &[NodeIdx], network: &Network) -> Distance {
        nodes
            .iter()
            .map(|n| network.node(*n).travel_distance())
            .sum()
    }

    fn compute_dead_head_distance_of_nodes(nodes: &[NodeIdx], network: &Network) -> Distance {
        if nodes.len() == 1 {
            Distance::ZERO
        } else {
            nodes
                .iter()
                .tuple_windows()
                .map(|(a, b)| network.dead_head_distance_between(*a, *b))
                .sum()
        }
    }

    fn compute_useful_duration_of_nodes(nodes: &[NodeIdx], network: &Network) -> Duration {
        nodes.iter().map(|n| network.node(*n).duration()).sum()
    }

    fn compute_costs_of_nodes(nodes: &[NodeIdx], network: &Network) -> Cost {
        nodes
            .iter()
            .map(|n| {
                network
                    .node(*n)
                    .duration()
                    .in_sec()
                    .unwrap_or(network.planning_days().in_sec().unwrap())
                    * match network.node(*n) {
                        Node::Service(_) => network.config().costs.service_trip,
                        Node::Maintenance(_) => network.config().costs.maintenance,
                        _ => 0,
                    }
            })
            .sum::<Cost>()
            + nodes
                .iter()
                .tuple_windows()
                .map(|(a, b)| {
                    network
                        .dead_head_time_between(*a, *b)
                        .in_sec()
                        .unwrap_or(network.planning_days().in_sec().unwrap())
                        * network.config().costs.dead_head_trip
                        + network
                            .idle_time_between(*a, *b)
                            .in_sec()
                            .unwrap_or(network.planning_days().in_sec().unwrap())
                            * network.config().costs.idle
                })
                .sum::<Cost>()
    }

    fn compute_visits_maintenance(nodes: &[NodeIdx], network: &Network) -> bool {
        nodes.iter().any(|&n| network.node(n).is_maintenance())
    }

    /// Creates a new tour from a vector of NodeIds. Trusts that the vector leads to a valid Tour.
    #[allow(clippy::too_many_arguments)]
    fn new_precomputed(
        nodes: Vec<NodeIdx>,
        is_dummy: bool,
        visits_maintenance: bool,
        useful_duration: Duration,
        service_distance: Distance,
        dead_head_distance: Distance,
        costs: Cost,
        network: Arc<Network>,
    ) -> Tour {
        Tour {
            nodes,
            is_dummy,
            visits_maintenance,
            useful_duration,
            service_distance,
            dead_head_distance,
            costs,
            network,
        }
    }
}
