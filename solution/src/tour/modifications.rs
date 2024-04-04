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
                * self.network.config().costs.dead_head_trip
            + self
                .network
                .dead_head_time_between(new_start_depot, first_non_depot)
                .in_sec()
                * self.network.config().costs.dead_head_trip;
        // there is no idle time.

        Ok(Tour::new_precomputed(
            nodes,
            self.is_dummy,
            self.visits_maintenance,
            self.useful_duration,
            self.service_distance,
            new_dead_head_distance,
            new_costs,
            self.network.clone(),
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
                * self.network.config().costs.dead_head_trip
            + self
                .network
                .dead_head_time_between(last_non_depot, new_end_depot)
                .in_sec()
                * self.network.config().costs.dead_head_trip;
        // there is no idle time.

        Ok(Tour::new_precomputed(
            nodes,
            self.is_dummy,
            self.visits_maintenance,
            self.useful_duration,
            self.service_distance,
            new_dead_head_distance,
            new_costs,
            self.network.clone(),
        ))
    }

    /// Removes the segment of the tour. The subpath between segment.start() and segment.end() (inclusive) is removed and the new
    /// shortened Tour as well as the removed nodes (as Path) are returned.
    /// Fails if either segment.start() or segment.end() is not part of the Tour.
    /// Fails if the start or end node would get removed but not both.
    /// In case that there is no non-depot left after the removel None is returned.
    pub fn remove(&self, segment: Segment) -> Result<(Option<Tour>, Path), String> {
        let pos_seg_start = self.position_of(segment.start())?;
        let pos_seg_end = self.position_of(segment.end())?;
        self.check_if_sequence_is_removable(pos_seg_start, pos_seg_end)?;

        // compute useful_duration, service_distance, dead_head_distance, and costs for the new tour:
        let new_useful_duration = self.useful_duration
            - (pos_seg_start..pos_seg_end + 1)
                .map(|i| self.network.node(self.nodes[i]).duration())
                .sum();
        let new_service_distance = self.service_distance
            - (pos_seg_start..pos_seg_end + 1)
                .map(|i| self.network.node(self.nodes[i]).travel_distance())
                .sum();
        let new_dead_head_distance = self.dead_head_distance
            - self.dead_head_distance_of_segment(pos_seg_start, pos_seg_end+1)
            // dead_head_distance for the new gap that is created:
            + if pos_seg_start == 0 || pos_seg_end == self.nodes.len() - 1 {
                Distance::ZERO
            } else {
                self.network
                    .dead_head_distance_between(self.nodes[pos_seg_start - 1], self.nodes[pos_seg_end + 1])
            };
        let new_costs = self.costs
            - self.costs_of_segment(pos_seg_start, pos_seg_end+1)
            // new costs by dead head trip replacing the segment
            + if pos_seg_start == 0 || pos_seg_end == self.nodes.len() - 1 {
                0
            } else {
                self.dead_head_and_idle_costs_between_two_nodes(self.nodes[pos_seg_start - 1], self.nodes[pos_seg_end + 1])
            };

        // remove the segment from the tour:
        let mut tour_nodes: Vec<NodeId> = self.nodes[..pos_seg_start].to_vec();
        tour_nodes.extend(self.nodes[pos_seg_end + 1..].iter().copied());
        let removed_nodes: Vec<NodeId> = self.nodes[pos_seg_start..pos_seg_end + 1].to_vec();
        if tour_nodes.is_empty() || (!self.is_dummy() && tour_nodes.len() <= 2) {
            return Ok((
                None,
                Path::new_trusted(removed_nodes, self.network.clone())
                    .expect("empty path should be impossible."),
            ));
        }

        // 1) if the old tour had no maintenance node than the new tour has no maintenance node either.
        // 2) if the old tour had a maintenance node and the removed segment had no maintenance than the
        //    new tour has a maintenance node.
        // 3) if the old tour had a maintenance node and the removed segment had also a maintenance node
        //    than the new tour might have a second mateinance node, so we need to check all remaining nodes.
        let visits_maintenance = self.visits_maintenance
            && (!removed_nodes
                .iter()
                .any(|n| self.network.node(*n).is_maintenance())
                || tour_nodes
                    .iter()
                    .any(|n| self.network.node(*n).is_maintenance()));

        Ok((
            Some(Tour::new_precomputed(
                tour_nodes,
                self.is_dummy,
                visits_maintenance,
                new_useful_duration,
                new_service_distance,
                new_dead_head_distance,
                new_costs,
                self.network.clone(),
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

        let visits_maintenance =
            self.visits_maintenance || path.iter().any(|n| self.network.node(n).is_maintenance());

        let new_nodes = path.consume();

        // get insertion position and check if insertion is valid
        let segment = Segment::new(*new_nodes.first().unwrap(), *new_nodes.last().unwrap());
        let (start_pos, end_pos) = self.get_insert_positions(segment); // start_pos up to end_pos-1 (inclusive) will be removed

        // compute the useful_duration, service_distance, dead_head_distance, and costs for the new tour:
        let new_useful_duration = self.useful_duration
            - (start_pos..end_pos)
                .map(|i| self.network.node(self.nodes[i]).duration())
                .sum()
            + new_nodes
                .iter()
                .map(|n| self.network.node(*n).duration())
                .sum();
        let new_service_distance = self.service_distance
            - (start_pos..end_pos)
                .map(|i| self.network.node(self.nodes[i]).travel_distance())
                .sum()
            + new_nodes
                .iter()
                .map(|n| self.network.node(*n).travel_distance())
                .sum();
        let new_dead_head_distance = self.dead_head_distance
            - self.dead_head_distance_of_segment(start_pos, end_pos)
            + self.dead_head_distance_of_new_nodes(&new_nodes, start_pos, end_pos);

        let new_costs = self.costs - self.costs_of_segment(start_pos, end_pos)
            + self.costs_of_new_nodes(&new_nodes, start_pos, end_pos);

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
                visits_maintenance,
                new_useful_duration,
                new_service_distance,
                new_dead_head_distance,
                new_costs,
                self.network.clone(),
            ),
            Path::new_trusted(removed_nodes, self.network.clone()),
        )
    }
}

// private methods
impl Tour {
    /// Returns the dead_head_distance reduction if the all nodes between start_pos and end_pos - 1 (inclusive start_pos and end_pos - 1) are removed.
    /// Does not count for the new dead head trip of the created gap.
    /// If start_pos == end_pos, the distance is given by the dead head before the node at start_pos.
    fn dead_head_distance_of_segment(&self, start_pos: Position, end_pos: Position) -> Distance {
        if start_pos >= end_pos {
            // no nodes to be remove
            return if start_pos == 0 || start_pos == self.nodes.len() {
                // new nodes are inserted before the first or after the last node
                Distance::ZERO
            } else {
                // new nodes are inserted within the tour
                // return the distance of the dead head trip before the node at start_pos
                self.network
                    .dead_head_distance_between(self.nodes[start_pos - 1], self.nodes[start_pos])
            };
        }
        let result: Distance = if start_pos == 0 {
            Distance::ZERO
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
            + if end_pos == self.nodes.len() {
                Distance::ZERO
            } else {
                self.network
                    .dead_head_distance_between(self.nodes[end_pos - 1], self.nodes[end_pos])
            };
        result
    }

    /// Returns the dead_head_distance of the new nodes if they are inserted between the old nodes at start_pos-1 and end_pos, i.e.,
    /// it is assumed that the nodes between start_pos and end_pos-1 (inclusive) are removed.
    fn dead_head_distance_of_new_nodes(
        &self,
        new_nodes: &[NodeId],
        start_pos: Position,
        end_pos: Position,
    ) -> Distance {
        let dead_head_distance: Distance = if start_pos == 0 {
            Distance::ZERO
        } else {
            self.network
                .dead_head_distance_between(self.nodes[start_pos - 1], new_nodes[0])
        } + new_nodes
            .iter()
            .tuple_windows()
            .map(|(a, b)| self.network.dead_head_distance_between(*a, *b))
            .sum::<Distance>()
            + if end_pos >= self.nodes.len() {
                Distance::ZERO
            } else {
                self.network
                    .dead_head_distance_between(new_nodes[new_nodes.len() - 1], self.nodes[end_pos])
            };
        dead_head_distance
    }

    /// Returns the costs reduction if the all nodes between start_pos and end_pos-1 (inclusive start_pos and end_pos-1) are removed.
    /// Does not count for the new dead head trip of the created gap.
    /// If start_pos == end_pos, the costs are are given by the dead head trip and idle time before the node at start_pos.
    fn costs_of_segment(&self, start_pos: Position, end_pos: Position) -> Cost {
        if start_pos >= end_pos {
            // no nodes to be remove
            return self.dead_head_and_idle_costs_before_node(start_pos);
        }

        // costs by dead head trip before removed segment
        self.dead_head_and_idle_costs_before_node(start_pos)
            // costs by dead head trips in between
            + (start_pos..end_pos-1)
                .map(|pos| {
                    self.dead_head_and_idle_costs_after_node_unchecked(pos)
                })
                .sum::<Cost>()
            // costs by dead head trip after removed segment
            + self.dead_head_and_idle_costs_after_node(end_pos-1)
            // costs by nodes in segment
            + (start_pos..end_pos)
                .map(|i| {
                    self.service_and_maintenance_costs_by_pos(i)
                })
                .sum::<Cost>()
    }

    /// Returns the costs of the new nodes if they are inserted between the old nodes at start_pos-1 and end_pos, i.e.,
    /// it is assumed that the nodes between start_pos and end_pos-1 (inclusive) are removed.
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
        + if end_pos >= self.nodes.len() {
            0
        } else {
            self.dead_head_and_idle_costs_between_two_nodes(new_nodes[new_nodes.len() - 1], self.nodes[end_pos])
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
        if pos >= self.nodes.len() - 1 {
            0
        } else {
            self.dead_head_and_idle_costs_after_node_unchecked(pos)
        }
    }

    /// Returns the costs for the dead head trip and the idle time between the nodes at pos and pos+1.
    /// If pos is the last node, panics.
    fn dead_head_and_idle_costs_after_node_unchecked(&self, pos: Position) -> Cost {
        self.network
            .dead_head_time_between(self.nodes[pos], self.nodes[pos + 1])
            .in_sec()
            * self.network.config().costs.dead_head_trip
            + self
                .network
                .idle_time_between(self.nodes[pos], self.nodes[pos + 1])
                .in_sec()
                * self.network.config().costs.idle
    }

    /// Returns the costs for the dead head trip and the idle time between the two nodes assuming
    /// no intermediate stops.
    fn dead_head_and_idle_costs_between_two_nodes(&self, node1: NodeId, node2: NodeId) -> Cost {
        self.network.dead_head_time_between(node1, node2).in_sec()
            * self.network.config().costs.dead_head_trip
            + self.network.idle_time_between(node1, node2).in_sec()
                * self.network.config().costs.idle
    }

    fn service_and_maintenance_costs_by_pos(&self, pos: Position) -> Cost {
        self.service_and_maintenance_costs_by_id(self.nodes[pos])
    }

    fn service_and_maintenance_costs_by_id(&self, node: NodeId) -> Cost {
        self.network.node(node).duration().in_sec()
            * match self.network.node(node) {
                Node::Service(_) => self.network.config().costs.service_trip,
                Node::Maintenance(_) => self.network.config().costs.maintenance,
                _ => 0,
            }
    }
}
