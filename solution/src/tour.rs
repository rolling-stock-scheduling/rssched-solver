#[cfg(test)]
mod tests;
use crate::path::Path;
use crate::segment::Segment;
use model::base_types::{Cost, Distance, NodeId};
use model::config::Config;
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
    nodes: Vec<NodeId>, // nodes will always be sorted by start_time; for non-dummy exactly first
    // and last node is a depot
    is_dummy: bool, // true if this is a dummy tour

    useful_duration: Duration, // duration of service trips and maintenance, excluding dead head and idle time
    service_distance: Distance, // distance covered by service trips
    dead_head_distance: Distance, // distance covered by dead head trips
    costs: Cost, // given by service_trip_duration * costs.service_trip + dead_head_trip_duration *
    // costs.dead_head_trip + idle_time * costs.idle + maintenance_time * costs.maintenance
    network: Arc<Network>,
    config: Arc<Config>,
}

// basic public methods
impl Tour {
    pub fn is_dummy(&self) -> bool {
        self.is_dummy
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn all_nodes_iter(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes.iter().copied()
    }

    /// return an iterator over all nodes (by start time) skipping the depot at the start and end
    pub fn all_non_depot_nodes_iter(&self) -> impl Iterator<Item = NodeId> + '_ {
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

    pub fn costs(&self) -> Cost {
        self.costs
    }

    /// the overhead time (dead_head + idle) between the predecessor and the node itself
    /// for the first non-depot node, as well as a depot, the overhead time is set to be infinity.
    /// (this is to allow for splitting before the first non-depot node in all cases)
    pub fn preceding_overhead(&self, node: NodeId) -> Result<Duration, String> {
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
    pub fn subsequent_overhead(&self, node: NodeId) -> Result<Duration, String> {
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

    pub fn first_node(&self) -> NodeId {
        *self.nodes.first().unwrap()
    }

    pub fn last_node(&self) -> NodeId {
        *self.nodes.last().unwrap()
    }

    pub fn nth_node(&self, pos: usize) -> Option<NodeId> {
        self.nodes.get(pos).copied()
    }

    pub fn first_non_depot(&self) -> Option<NodeId> {
        self.all_non_depot_nodes_iter().next()
    }

    /// returns the last non-depot (service node or maintenance node) of the tour, ignoring depot.
    /// If the tour does only contain depots None is returned.
    pub fn last_non_depot(&self) -> Option<NodeId> {
        self.nodes
            .iter()
            .rev()
            .find(|&&n| !self.network.node(n).is_depot())
            .copied()
    }

    pub fn start_depot(&self) -> Result<NodeId, String> {
        if self.network.node(self.first_node()).is_start_depot() {
            Ok(self.first_node())
        } else {
            Err("tour does not have a start depot.".to_string())
        }
    }

    pub fn end_depot(&self) -> Result<NodeId, String> {
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
                .node(self.nth_node(self.len() - 2).unwrap())
                .end_time()
                + self.network.dead_head_time_between(
                    self.nth_node(self.len() - 2).unwrap(),
                    self.last_node(),
                )
        }
    }

    pub fn removable(&self, segment: Segment) -> bool {
        let start_pos_res = self.position_of(segment.start());
        let end_pos_res = self.position_of(segment.end());

        match (start_pos_res, end_pos_res) {
            (Ok(start_pos), Ok(end_pos)) => self.removable_by_pos(start_pos, end_pos).is_ok(),
            _ => false,
        }
    }

    /// start_position is here the position of the first node that should be removed
    /// end_position is here the position in the tour of the last node that should be removed
    /// in other words [start_position .. end_position+1) is about to be removed
    fn removable_by_pos(&self, start_position: usize, end_position: usize) -> Result<(), String> {
        if start_position > end_position {
            return Err(String::from("segment.start() comes after segment.end()."));
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

    /// return the position of the node in the tour that is the latest one that cannot reach the
    /// provided node.
    /// If all nodes can reach the provided node, None is returned.
    pub(super) fn latest_not_reaching_node(&self, node: NodeId) -> Option<Position> {
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

    pub(crate) fn print(&self) {
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
}

// modification methods
impl Tour {
    /// for a given segment (in general of another tour) returns all nodes that are conflicting
    /// when the segment would have been inserted. These nodes form a path that is returned.
    /// If this path is empty (or consists only of depots) None is returned.
    /// Fails if the segment insertion would not lead  to a valid Tour (for example start node
    /// cannot reach segment.start(), or segment.end() cannot reach end node).
    pub(super) fn conflict(&self, segment: Segment) -> Option<Path> {
        let (start_pos, end_pos) = self.get_insert_positions(segment);
        Path::new_trusted(
            self.nodes[start_pos..end_pos].to_vec(),
            self.network.clone(),
        )
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
    /// - Note that depot can never clash. So their is no failure possible.
    pub(super) fn insert_path(&self, path: Path) -> (Tour, Option<Path>) {
        // remove depots from path if self.is_dummy=true.
        let mut path = path;
        if self.is_dummy {
            if self.network.node(path.first()).is_depot() {
                path = path.drop_first().unwrap();
            }
            if self.network.node(path.last()).is_depot() {
                path = path.drop_last().unwrap();
            }
        }
        self.insert_sequence(path.consume())
    }

    pub(super) fn replace_start_depot(&self, new_start_depot: NodeId) -> Result<Tour, String> {
        if self.is_dummy {
            return Err("cannot replace start depot of dummy tour".to_string());
        }
        if !self.network.node(new_start_depot).is_start_depot() {
            return Err("node has to be start depot".to_string());
        }
        let mut nodes = self.nodes.clone();
        nodes[0] = new_start_depot;
        let first_non_depot = nodes[1];
        let new_dead_head_distance = self.dead_head_distance
            - self
                .network
                .dead_head_distance_between(self.first_node(), first_non_depot)
            + self
                .network
                .dead_head_distance_between(new_start_depot, first_non_depot);
        let new_costs = self.costs
            - self
                .network
                .dead_head_time_between(self.first_node(), first_non_depot)
                .in_sec()
                * self.config.costs.dead_head_trip
            + self
                .network
                .dead_head_time_between(new_start_depot, first_non_depot)
                .in_sec()
                * self.config.costs.dead_head_trip;
        Ok(Tour::new_precomputed(
            nodes,
            self.is_dummy,
            self.useful_duration,
            self.service_distance,
            new_dead_head_distance,
            new_costs,
            self.network.clone(),
            self.config.clone(),
        ))
    }

    pub(super) fn replace_end_depot(&self, new_end_depot: NodeId) -> Result<Tour, String> {
        if self.is_dummy {
            return Err("cannot replace end depot of dummy tour".to_string());
        }
        if !self.network.node(new_end_depot).is_end_depot() {
            return Err("node has to be end depot".to_string());
        }
        let mut nodes = self.nodes.clone();
        let end_index = nodes.len() - 1;
        nodes[end_index] = new_end_depot;
        let last_non_depot = nodes[end_index - 1];
        let new_dead_head_distance = self.dead_head_distance
            - self
                .network
                .dead_head_distance_between(last_non_depot, self.last_node())
            + self
                .network
                .dead_head_distance_between(last_non_depot, new_end_depot);
        let new_costs = self.costs
            - self
                .network
                .dead_head_time_between(last_non_depot, self.last_node())
                .in_sec()
                * self.config.costs.dead_head_trip
            + self
                .network
                .dead_head_time_between(last_non_depot, new_end_depot)
                .in_sec()
                * self.config.costs.dead_head_trip;
        Ok(Tour::new_precomputed(
            nodes,
            self.is_dummy,
            self.useful_duration,
            self.service_distance,
            new_dead_head_distance,
            new_costs,
            self.network.clone(),
            self.config.clone(),
        ))
    }

    /// checks whether segment can be removed from tour or not.
    pub(crate) fn check_removable(&self, segment: Segment) -> Result<(), String> {
        let start_pos = self.position_of(segment.start())?;
        let end_pos = self.position_of(segment.end())?;
        self.check_if_sequence_is_removable(start_pos, end_pos)
    }

    /// Removes the segment of the tour. The subpath between segment.start() and segment.end() is removed and the new
    /// shortened Tour as well as the removed nodes (as Path) are returned.
    /// Fails if either segment.start() or segment.end() is not part of the Tour.
    /// Fails if the start or end node would get removed but not both.
    /// In case that there is no non-depot left after the removel None is returned.
    pub(crate) fn remove(&self, segment: Segment) -> Result<(Option<Tour>, Path), String> {
        let start_pos = self.position_of(segment.start())?;
        let end_pos = self.position_of(segment.end())?;
        self.check_if_sequence_is_removable(start_pos, end_pos)?;

        // compute useful_duration, service_distance and dead_head_distance for the removed segment:
        let removed_useful_duration = (start_pos..end_pos + 1)
            .map(|i| self.network.node(self.nodes[i]).duration())
            .sum();
        let removed_service_distance = (start_pos..end_pos + 1)
            .map(|i| self.network.node(self.nodes[i]).travel_distance())
            .sum();
        let removed_dead_head_distance: Distance = if start_pos == 0 {
            Distance::zero()
        } else {
            self.network
                .dead_head_distance_between(self.nodes[start_pos - 1], self.nodes[start_pos])
        } + (start_pos..end_pos + 1)
            .tuple_windows()
            .map(|(i, j)| {
                self.network
                    .dead_head_distance_between(self.nodes[i], self.nodes[j])
            })
            .sum()
            + if end_pos == self.nodes.len() - 1 {
                Distance::zero()
            } else {
                self.network
                    .dead_head_distance_between(self.nodes[end_pos], self.nodes[end_pos + 1])
            };

        // compute the dead_head_distance for the new gap that is created:
        let added_dead_head_distance = if start_pos == 0 || end_pos == self.nodes.len() - 1 {
            Distance::zero()
        } else {
            self.network
                .dead_head_distance_between(self.nodes[start_pos - 1], self.nodes[end_pos + 1])
        };

        // compute new costs
        let new_costs = self.costs
            // costs by dead head trip before removed segment
            - if start_pos == 0 {
                0
                } else {
                    self
                    .network
                    .dead_head_time_between(self.nodes[start_pos - 1], self.nodes[start_pos])
                    .in_sec()
                    * self.config.costs.dead_head_trip
            }
            // costs by dead head trips in between
            - (start_pos..end_pos + 1).tuple_windows()
                .map(|(i, j)| {
                    self.network
                        .dead_head_time_between(self.nodes[i], self.nodes[j])
                        .in_sec()
                        * self.config.costs.dead_head_trip
                })
                .sum::<Cost>()
            // costs by dead head trip after removed segment
            - if end_pos == self.nodes.len() - 1 {
                0
            } else {
            self
                .network
                .dead_head_time_between(self.nodes[end_pos], self.nodes[end_pos + 1])
                .in_sec()
                * self.config.costs.dead_head_trip
            }
            // costs by nodes in segment
            - (start_pos..end_pos + 1)
                .map(|i| {
                    self.network.node(self.nodes[i]).duration().in_sec()
                        * match self.network.node(self.nodes[i]) {
                            Node::Service(_) => self.config.costs.service_trip,
                            Node::Maintenance(_) => self.config.costs.maintenance,
                            _ => 0,
                        }
                })
                .sum::<Cost>()
            // new costs by dead head trip replacing the segment
            + self
                .network
                .dead_head_time_between(self.nodes[start_pos - 1], self.nodes[end_pos + 1])
                .in_sec()
                * self.config.costs.dead_head_trip;

        // remove the segment from the tour:
        let mut tour_nodes: Vec<NodeId> = self.nodes[..start_pos].to_vec();
        tour_nodes.extend(self.nodes[end_pos + 1..].iter().copied());
        let removed_nodes: Vec<NodeId> = self.nodes[start_pos..end_pos + 1].to_vec();
        if tour_nodes.is_empty() || (!self.is_dummy() && tour_nodes.len() <= 2) {
            return Ok((
                None,
                Path::new_trusted(removed_nodes, self.network.clone())
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
                new_costs,
                self.network.clone(),
                self.config.clone(),
            )),
            Path::new_trusted(removed_nodes, self.network.clone())
                .expect("empty path should be impossible."),
        ))
    }

    pub(crate) fn sub_path(&self, segment: Segment) -> Result<Path, String> {
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
}

// private methods
impl Tour {
    pub fn position_of(&self, node: NodeId) -> Result<usize, String> {
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

    /// computes the position of the latest tour-node that is not reached by node.
    /// if node can reach all tour-nodes, None is returned.
    fn latest_not_reached_by_node(&self, node: NodeId) -> Option<Position> {
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

    // it is assumed that new_nodes are a valid path in the network
    fn insert_sequence(&self, new_nodes: Vec<NodeId>) -> (Tour, Option<Path>) {
        // get insertion position and check if insertion is valid
        let segment = Segment::new(*new_nodes.first().unwrap(), *new_nodes.last().unwrap());
        let (start_pos, end_pos) = self.get_insert_positions(segment);

        // compute useful_duration, service_distance and dead_head_distance for the path:
        let path_useful_duration = new_nodes
            .iter()
            .copied()
            .map(|n| self.network.node(n).duration())
            .sum();
        let path_service_distance = new_nodes
            .iter()
            .map(|n| self.network.node(*n).travel_distance())
            .sum();
        let path_dead_head_distance: Distance = if start_pos == 0 {
            Distance::zero()
        } else {
            self.network
                .dead_head_distance_between(self.nodes[start_pos - 1], segment.start())
        } + new_nodes
            .iter()
            .copied()
            .tuple_windows()
            .map(|(a, b)| self.network.dead_head_distance_between(a, b))
            .sum()
            + if end_pos >= self.nodes.len() {
                Distance::zero()
            } else {
                self.network
                    .dead_head_distance_between(segment.end(), self.nodes[end_pos])
            };

        // compute useful_duration, service_distance and dead_head_distance for the segment that is
        // removed:
        let removed_useful_duration = (start_pos..end_pos)
            .map(|i| self.network.node(self.nodes[i]).duration())
            .sum();
        let removed_service_distance = (start_pos..end_pos)
            .map(|i| self.network.node(self.nodes[i]).travel_distance())
            .sum();
        let removed_dead_head_distance: Distance =
            if start_pos == 0 || start_pos >= self.nodes.len() {
                Distance::zero()
            } else {
                self.network
                    .dead_head_distance_between(self.nodes[start_pos - 1], self.nodes[start_pos])
            } + (start_pos..end_pos)
                .tuple_windows()
                .map(|(i, j)| {
                    self.network
                        .dead_head_distance_between(self.nodes[i], self.nodes[j])
                })
                .sum()
                + if end_pos == self.nodes.len()
                    || (start_pos == end_pos && start_pos > 0)
                    || end_pos == 0
                {
                    Distance::zero()
                } else {
                    self.network
                        .dead_head_distance_between(self.nodes[end_pos - 1], self.nodes[end_pos])
                };

        // compute the useful_duration, service_distance and dead_head_distance for the new tour:
        let useful_duration = self.useful_duration + path_useful_duration - removed_useful_duration;
        let service_distance =
            self.service_distance + path_service_distance - removed_service_distance;
        let dead_head_distance =
            self.dead_head_distance + path_dead_head_distance - removed_dead_head_distance;

        let new_costs = self.costs
            // costs by dead head trip before removed segment
            - if start_pos == 0 {
                0
            } else {
                self
                    .network
                    .dead_head_time_between(self.nodes[start_pos - 1], self.nodes[start_pos])
                    .in_sec()
                    * self.config.costs.dead_head_trip
            }
            // costs by dead head trips in between
            - (start_pos..end_pos).tuple_windows()
                .map(|(i, j)| {
                    self.network
                        .dead_head_time_between(self.nodes[i], self.nodes[j])
                        .in_sec()
                        * self.config.costs.dead_head_trip
                })
                .sum::<Cost>()
            // costs by dead head trip after removed segment
            - if end_pos == self.nodes.len() {
                0
            } else {
                self
                    .network
                    .dead_head_time_between(self.nodes[end_pos- 1], self.nodes[end_pos])
                    .in_sec()
                    * self.config.costs.dead_head_trip
            }
            // costs of removed nodes
            - (start_pos..end_pos)
                .map(|i| {
                    self.network.node(self.nodes[i]).duration().in_sec()
                        * match self.network.node(self.nodes[i]) {
                            Node::Service(_) => self.config.costs.service_trip,
                            Node::Maintenance(_) => self.config.costs.maintenance,
                            _ => 0,
                        }
                })
            .sum::<Cost>()
            // new costs for dead head trip before new nodes
            + if start_pos == 0 {
                0
            } else {
                self
                    .network
                    .dead_head_time_between(self.nodes[start_pos - 1], new_nodes[0])
                    .in_sec()
                    * self.config.costs.dead_head_trip
            }
            // new costs for dead head trips in between
            + new_nodes
                .iter()
                .tuple_windows()
                .map(|(a, b)| {
                    self.network
                        .dead_head_time_between(*a, *b)
                        .in_sec()
                        * self.config.costs.dead_head_trip
                })
                .sum::<Cost>()
            // new costs for dead head trip after new nodes
            + if end_pos == self.nodes.len() {
                0
            } else {
                self
                    .network
                    .dead_head_time_between(new_nodes[new_nodes.len() - 1], self.nodes[end_pos])
                    .in_sec()
                    * self.config.costs.dead_head_trip
            }
            // new costs for the new nodes
            + new_nodes
                .iter()
                .map(|n| {
                    self.network.node(*n).duration().in_sec()
                        * match self.network.node(*n) {
                            Node::Service(_) => self.config.costs.service_trip,
                            Node::Maintenance(_) => self.config.costs.maintenance,
                            _ => 0,
                        }
                })
                .sum::<Cost>();

        // remove all elements in start_pos..end_pos and replace them by
        // node_sequence.
        let mut new_tour_nodes = self.nodes.clone();
        let removed_nodes: Vec<NodeId> = new_tour_nodes
            .splice(start_pos..end_pos, new_nodes)
            .collect();

        (
            Tour::new_precomputed(
                new_tour_nodes,
                self.is_dummy,
                useful_duration,
                service_distance,
                dead_head_distance,
                new_costs,
                self.network.clone(),
                self.config.clone(),
            ),
            Path::new_trusted(removed_nodes, self.network.clone()),
        )
    }

    /// start_position is here the position of the first node that should be removed
    /// end_position is here the position in the tour of the last node that should be removed
    /// in other words [start_position .. end_position+1) is about to be removed
    fn check_if_sequence_is_removable(
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
                .network
                .can_reach(self.nodes[start_position - 1], self.nodes[end_position + 1])
        {
            return Err(format!("Removing nodes ({} to {}) makes the tour invalid. Dead-head-trip is slower than service-trips.", self.nodes[start_position], self.nodes[end_position]));
        }
        Ok(())
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
        let mut nodes_iter = self.nodes.iter();
        write!(f, "{}", self.network.node(*nodes_iter.next().unwrap()))?;
        for node in nodes_iter {
            write!(f, " - {}", self.network.node(*node))?;
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
        network: Arc<Network>,
        config: Arc<Config>,
    ) -> Result<Tour, String> {
        Tour::new_allow_invalid(nodes, network, config).map_err(|(_, error_msg)| error_msg)
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
        network: Arc<Network>,
        config: Arc<Config>,
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
            Err((
                Tour::new_computing(nodes, false, network, config),
                error_msg,
            ))
        } else {
            Ok(Tour::new_computing(nodes, false, network, config))
        }
    }

    pub(super) fn new_dummy(path: Path, network: Arc<Network>, config: Arc<Config>) -> Tour {
        let mut nodes = path.consume();
        // remove start and end depot
        if network.node(*nodes.first().unwrap()).is_depot() {
            nodes.remove(0);
        };
        if network.node(*nodes.last().unwrap()).is_depot() {
            nodes.pop();
        };
        assert!(!nodes.is_empty());
        Tour::new_computing(nodes, true, network, config)
    }

    fn new_computing(
        nodes: Vec<NodeId>,
        is_dummy: bool,
        network: Arc<Network>,
        config: Arc<Config>,
    ) -> Tour {
        let useful_duration = nodes
            .iter()
            .map(|&n| network.node(n).duration())
            .sum::<Duration>();
        let service_distance = nodes
            .iter()
            .map(|&n| network.node(n).travel_distance())
            .sum();
        let dead_head_distance = nodes
            .iter()
            .tuple_windows()
            .map(|(&a, &b)| network.dead_head_distance_between(a, b))
            .sum();
        let costs = nodes
            .iter()
            .map(|&n| {
                network.node(n).duration().in_sec()
                    * match network.node(n) {
                        Node::Service(_) => config.costs.service_trip,
                        Node::Maintenance(_) => config.costs.maintenance,
                        _ => 0,
                    }
            })
            .sum::<Cost>()
            + nodes
                .iter()
                .tuple_windows()
                .map(|(&a, &b)| {
                    network.dead_head_time_between(a, b).in_sec() * config.costs.dead_head_trip
                })
                .sum::<Cost>();

        Tour::new_precomputed(
            nodes,
            is_dummy,
            useful_duration,
            service_distance,
            dead_head_distance,
            costs,
            network,
            config,
        )
    }

    /// Creates a new tour from a vector of NodeIds. Trusts that the vector leads to a valid Tour.
    #[allow(clippy::too_many_arguments)]
    fn new_precomputed(
        nodes: Vec<NodeId>,
        is_dummy: bool,
        useful_duration: Duration,
        service_distance: Distance,
        dead_head_distance: Distance,
        costs: Cost,
        network: Arc<Network>,
        config: Arc<Config>,
    ) -> Tour {
        Tour {
            nodes,
            is_dummy,
            useful_duration,
            service_distance,
            dead_head_distance,
            costs,
            network,
            config,
        }
    }
}
