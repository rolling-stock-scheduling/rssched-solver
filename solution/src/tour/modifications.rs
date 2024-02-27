use itertools::Itertools;
use model::{
    base_types::{Cost, Distance, NodeId},
    network::nodes::Node,
};

use crate::{path::Path, segment::Segment};

use super::{Position, Tour};

// modification methods
impl Tour {
    /// Return the tour where the start depot is replaced by the new_start_depot.
    pub fn replace_start_depot(&self, new_start_depot: NodeId) -> Result<Tour, String> {
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

    /// Return the tour where the end depot is replaced by the new_end_depot.
    pub fn replace_end_depot(&self, new_end_depot: NodeId) -> Result<Tour, String> {
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

    /// Removes the segment of the tour. The subpath between segment.start() and segment.end() (inclusive) is removed and the new
    /// shortened Tour as well as the removed nodes (as Path) are returned.
    /// Fails if either segment.start() or segment.end() is not part of the Tour.
    /// Fails if the start or end node would get removed but not both.
    /// In case that there is no non-depot left after the removel None is returned.
    pub fn remove(&self, segment: Segment) -> Result<(Option<Tour>, Path), String> {
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
            - self.costs_of_segment(start_pos, end_pos)
            // new costs by dead head trip replacing the segment
            + self.dead_head_and_idle_costs_between_two_nodes(self.nodes[start_pos - 1], self.nodes[end_pos + 1]);

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
    pub fn insert_path(&self, path: Path) -> (Tour, Option<Path>) {
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
        let new_nodes = path.consume();

        // get insertion position and check if insertion is valid
        let segment = Segment::new(*new_nodes.first().unwrap(), *new_nodes.last().unwrap());
        let (start_pos, end_pos) = self.get_insert_positions(segment); // start_pos up to end_pos-1 will be removed

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
            // costs of removed segment
            - self.costs_of_segment(start_pos, end_pos-1)
            // new costs for dead head trip before new nodes
            + self.costs_of_new_nodes(&new_nodes, start_pos, end_pos-1);
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
}

// private methods
impl Tour {
    /// Returns the costs reduction if the all nodes between start_pos and end_pos (inclusive start_pos and end_pos) are removed.
    /// Does not count for the new dead head trip of the created gap.
    fn costs_of_segment(&self, start_pos: Position, end_pos: Position) -> Cost {
        // costs by dead head trip before removed segment
        self.dead_head_and_idle_costs_before_node(start_pos)
            // costs by dead head trips in between
            + (start_pos..end_pos)
                .map(|pos| {
                    self.dead_head_and_idle_costs_after_node_unchecked(pos)
                })
                .sum::<Cost>()
            // costs by dead head trip after removed segment
            + self.dead_head_and_idle_costs_after_node(end_pos)
            // costs by nodes in segment
            + (start_pos..end_pos + 1)
                .map(|i| {
                    self.service_and_maintenance_costs_by_pos(i)
                })
                .sum::<Cost>()
    }

    /// Returns the costs of the new nodes if they are inserted between the old nodes at start_pos-1 and end_pos+1, i.e.,
    /// it is assumed that the nodes between start_pos and end_pos (inclusive) are removed.
    fn costs_of_new_nodes(
        &self,
        new_nodes: &[NodeId],
        start_pos: Position,
        end_pos: Position,
    ) -> Cost {
        let costs: Cost = if start_pos == 0 {
            0
        } else {
            self.dead_head_and_idle_costs_between_two_nodes(self.nodes[start_pos - 1], new_nodes[0])
        }
        // new costs for dead head trips in between
        + new_nodes
            .iter()
            .tuple_windows()
            .map(|(a, b)| {
                self.dead_head_and_idle_costs_between_two_nodes(*a, *b)
            })
            .sum::<Cost>()
        // new costs for dead head trip after new nodes
        + if end_pos+1 == self.nodes.len() {
            0
        } else {
            self.dead_head_and_idle_costs_between_two_nodes(new_nodes[new_nodes.len() - 1], self.nodes[end_pos+1])
        }
        // new costs for the new nodes
        + new_nodes
            .iter()
            .map(|n| {
                self.service_and_maintenance_costs_by_id(*n)
            })
            .sum::<Cost>();
        costs
    }

    /// Returns the costs for the dead head trip and the idle time between the nodes at pos-1 and pos.
    /// If pos is 0, the costs are 0.
    fn dead_head_and_idle_costs_before_node(&self, pos: Position) -> Cost {
        if pos == 0 {
            0
        } else {
            self.dead_head_and_idle_costs_after_node(pos - 1)
        }
    }

    /// Returns the costs for the dead head trip and the idle time between the nodes at pos and pos+1.
    /// If pos is the last node, the costs are 0.
    fn dead_head_and_idle_costs_after_node(&self, pos: Position) -> Cost {
        if pos == self.nodes.len() - 1 {
            0
        } else {
            self.dead_head_and_idle_costs_after_node(pos)
        }
    }

    /// Returns the costs for the dead head trip and the idle time between the nodes at pos and pos+1.
    /// If pos is the last node, panics.
    fn dead_head_and_idle_costs_after_node_unchecked(&self, pos: Position) -> Cost {
        self.network
            .dead_head_time_between(self.nodes[pos], self.nodes[pos + 1])
            .in_sec()
            * self.config.costs.dead_head_trip
            + self
                .network
                .idle_time_between(self.nodes[pos], self.nodes[pos + 1])
                .in_sec()
                * self.config.costs.idle
    }

    /// Returns the costs for the dead head trip and the idle time between the two nodes assuming
    /// no intermediate stops.
    fn dead_head_and_idle_costs_between_two_nodes(&self, node1: NodeId, node2: NodeId) -> Cost {
        self.network.dead_head_time_between(node1, node2).in_sec()
            * self.config.costs.dead_head_trip
            + self.network.idle_time_between(node1, node2).in_sec() * self.config.costs.idle
    }

    fn service_and_maintenance_costs_by_pos(&self, pos: Position) -> Cost {
        self.service_and_maintenance_costs_by_id(self.nodes[pos])
    }

    fn service_and_maintenance_costs_by_id(&self, node: NodeId) -> Cost {
        self.network.node(node).duration().in_sec()
            * match self.network.node(node) {
                Node::Service(_) => self.config.costs.service_trip,
                Node::Maintenance(_) => self.config.costs.maintenance,
                _ => 0,
            }
    }
}
