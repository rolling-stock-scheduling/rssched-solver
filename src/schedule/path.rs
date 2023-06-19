use crate::base_types::NodeId;
use crate::network::Network;
use itertools::Itertools;
use std::fmt;

use std::sync::Arc;

use std::iter::Iterator;
/// Path is similar as Tour a sequence of nodes that form a path in the network.
/// Instead of a Tour a Path does not need to start with a StartNode nor end with an EndNode
/// It is mainly used for sequence of uncovered nodes.
#[derive(Clone)]
pub(crate) struct Path {
    node_sequence: Vec<NodeId>,

    nw: Arc<Network>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Segment {
    start: NodeId,
    end: NodeId,
}

////////////////////////////////////////////
///////////////// Path /////////////////////
////////////////////////////////////////////

// static functions
impl Path {
    /// crates a new Path and asserts that it is a path in the network
    pub(crate) fn new(node_sequence: Vec<NodeId>, nw: Arc<Network>) -> Path {
        for (&a, &b) in node_sequence.iter().tuple_windows() {
            assert!(
                nw.can_reach(a, b),
                "Not a valid Path: {} cannot reach {}.",
                a,
                b
            );
            // if !nw.can_reach(a,b) {
            // println!("Not a valid Path: {} cannot reach {}.", a, b);
            // }
        }
        Path { node_sequence, nw }
    }

    /// crates a new Path but does NOT assert that it is a path in the network
    pub(crate) fn new_trusted(node_sequence: Vec<NodeId>, nw: Arc<Network>) -> Path {
        Path { node_sequence, nw }
    }
}

// methods
impl Path {
    pub(crate) fn iter(&self) -> impl Iterator<Item = &NodeId> + '_ {
        self.node_sequence.iter()
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

    pub(crate) fn is_empty(&self) -> bool {
        self.node_sequence.is_empty()
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut nodes_iter = self.node_sequence.iter();
        write!(f, "{}", self.nw.node(*nodes_iter.next().unwrap()))?;
        for node in nodes_iter {
            write!(f, " - {}", self.nw.node(*node))?;
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
