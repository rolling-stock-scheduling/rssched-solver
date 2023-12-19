use itertools::Itertools;
use sbb_model::base_types::NodeId;
use sbb_model::network::Network;
use std::fmt;

use std::sync::Arc;

use std::iter::Iterator;
/// Path is similar as Tour a sequence of nodes that form a path in the network.
///
/// Instead of a Tour a Path does not need to start nor end at a depot.
/// It can start at a StartDepot and end at an EndDepot.
/// It must have at least one non-depot node
/// It cannot have any intermediate depots (which is indirectly given as it is a path)
///
/// It is mainly used for sequence of uncovered nodes.
#[derive(Clone)]
pub struct Path {
    node_sequence: Vec<NodeId>,

    network: Arc<Network>,
}

// static functions
impl Path {
    /// crates a new Path and asserts that:
    /// it is a path in the network,
    /// it has no intermediate depots,
    /// it has at least one non-depot nodes.
    pub fn new(node_sequence: Vec<NodeId>, nw: Arc<Network>) -> Result<Option<Path>, String> {
        for (&a, &b) in node_sequence.iter().tuple_windows() {
            if !nw.can_reach(a, b) {
                return Err(format!("Not a valid Path: {} cannot reach {}.", a, b));
            };
        }
        Ok(Path::new_trusted(node_sequence, nw))
    }

    /// crates a new Path but does NOT assert if it is a feasible path in the network.
    /// If node_sequence does not contain any non-depot nodes, None is returned.
    pub(crate) fn new_trusted(node_sequence: Vec<NodeId>, nw: Arc<Network>) -> Option<Path> {
        if node_sequence.iter().all(|&node| nw.node(node).is_depot()) {
            None
        } else {
            Some(Path {
                node_sequence,
                network: nw,
            })
        }
    }

    pub fn new_from_single_node(node: NodeId, network: Arc<Network>) -> Path {
        assert!(!network.node(node).is_depot());
        Path {
            node_sequence: vec![node],
            network,
        }
    }
}

// methods
impl Path {
    pub(crate) fn iter(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.node_sequence.iter().copied()
    }

    pub(crate) fn len(&self) -> usize {
        self.node_sequence.len()
    }

    pub(crate) fn first(&self) -> NodeId {
        self.node_sequence[0]
    }

    pub(crate) fn last(&self) -> NodeId {
        self.node_sequence[self.node_sequence.len() - 1]
    }

    pub(crate) fn consume(self) -> Vec<NodeId> {
        self.node_sequence
    }

    /// if the path does not contain any non-depots afterwards, None is returned.
    pub(crate) fn drop_first(&self) -> Option<Path> {
        Path::new_trusted(self.node_sequence[1..].to_vec(), self.network.clone())
    }

    /// if the path does not contain any non-depots afterwards, None is returned.
    pub(crate) fn drop_last(&self) -> Option<Path> {
        Path::new_trusted(
            self.node_sequence[..self.node_sequence.len() - 1].to_vec(),
            self.network.clone(),
        )
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut nodes_iter = self.node_sequence.iter();
        write!(f, "{}", self.network.node(*nodes_iter.next().unwrap()))?;
        for node in nodes_iter {
            write!(f, " - {}", self.network.node(*node))?;
        }
        Ok(())
    }
}
