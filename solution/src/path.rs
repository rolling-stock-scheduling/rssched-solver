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

#[derive(Debug, Clone, Copy)]
pub struct Segment {
    start: NodeId,
    end: NodeId,
}

////////////////////////////////////////////
///////////////// Path /////////////////////
////////////////////////////////////////////

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

    pub fn new_from_single_node(node: NodeId, nw: Arc<Network>) -> Path {
        assert!(!nw.node(node).is_depot());
        Path {
            node_sequence: vec![node],
            network: nw,
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

////////////////////////////////////////////
////////////// Segment /////////////////////
////////////////////////////////////////////

// static functions
impl Segment {
    pub(crate) fn new(start: NodeId, end: NodeId) -> Segment {
        Segment { start, end }
    }
}

// methods
impl Segment {
    pub(crate) fn start(&self) -> NodeId {
        self.start
    }

    pub(crate) fn end(&self) -> NodeId {
        self.end
    }
}

impl fmt::Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}..{}]", self.start, self.end)
    }
}
