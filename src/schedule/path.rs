use crate::base_types::NodeId;
use crate::locations::Locations;
use crate::network::Network;
use itertools::Itertools;

use std::rc::Rc;

use std::iter::Iterator;
/// Path is similar as Tour a sequence of nodes that form a path in the network.
/// Instead of a Tour a Path does not need to start with a StartNode nor end with an EndNode
/// It is mainly used for sequence of uncovered nodes.
#[derive(Clone)]
pub(crate) struct Path {
    node_sequence: Vec<NodeId>,

    loc: Rc<Locations>,
    nw: Rc<Network>
}

impl Path{

    /// crates a new Path and asserts that it is a path in the network
    pub(crate) fn new(node_sequence: Vec<NodeId>, loc: Rc<Locations>, nw: Rc<Network>) -> Path {
        for (&a,&b) in node_sequence.iter().tuple_windows() {
            assert!(nw.can_reach(a,b),"Not a valid Path");
        }
        Path{node_sequence, loc, nw}
    }

    /// crates a new Path but does NOT assert that it is a path in the network
    pub(crate) fn new_trusted(node_sequence: Vec<NodeId>, loc: Rc<Locations>, nw: Rc<Network>) -> Path {
        Path{node_sequence, loc, nw}
    }
}

impl Path {
    pub(crate) fn iter(&self) -> impl Iterator<Item=&NodeId> + '_ {
        self.node_sequence.iter()
    }

    pub(crate) fn first(&self) -> NodeId {
        self.node_sequence[0]
    }

    pub(crate) fn last(&self) -> NodeId {
        self.node_sequence[self.node_sequence.len()-1]
    }

    pub(crate) fn consume(self) -> Vec<NodeId> {
        self.node_sequence
    }
}
