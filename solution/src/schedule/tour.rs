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
    nodes: Vec<NodeId>,               // nodes will always be sorted by start_time
    depots: Option<(NodeId, NodeId)>, // the depots where the tour starts and ends (if it is not a dummy tour)

    overhead_time: Duration, // the overhead time (dead_head + idle) of the tour
    service_distance: Distance,
    dead_head_distance: Distance,
    continuous_idle_time_cost: Cost,

    config: Arc<Config>,
    nw: Arc<Network>,
}

impl Tour {
    pub(crate) fn is_dummy(&self) -> bool {
        self.depots.is_none()
    }

    pub(crate) fn all_nodes_iter(&self) -> impl Iterator<Item = NodeId> + '_ {
        match self.depots {
            None => {
                // in order to obtain the correct Iterator type we have to add an empty iterator at
                // the beginning and the end of type Once.
                let mut empty_start = iter::once(NodeId::from("."));
                empty_start.next(); // skip StartNode
                let mut empty_end = iter::once(NodeId::from("."));
                empty_end.next(); // skip StartNode
                empty_start
                    .chain(self.nodes.iter().copied())
                    .chain(empty_end)
            }
            Some((start, end)) => iter::once(start)
                .chain(self.nodes.iter().copied())
                .chain(iter::once(end)),
        }
    }

    /// return an iterator over all nodes (by start time) skipping the depot at the start and end
    pub(crate) fn movable_nodes(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes.iter().copied()
    }
}
