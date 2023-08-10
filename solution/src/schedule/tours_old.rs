impl Tour {
    // pub(super) fn insert_single_node(&self, node: NodeId) -> Result<Tour,String> {
    // self.insert(Path::new(vec!(node), self.nw.clone()))
    // }

    // pub(crate) fn remove_single_node(&self, node: NodeId) -> Result<Tour,String> {
    // self.remove(Segment::new(node, node)).map(|tuple| tuple.0)
    // }

    // pub(super) fn conflict_single_node(&self, node: NodeId) -> Result<Path,String> {
    // self.conflict(Segment::new(node, node))
    // }

    // pub(crate) fn removable_single_node(&self, node: NodeId) -> bool {
    // self.removable(Segment::new(node, node))
    // }
}

// one tour is bigger than the other if the number of nodes are bigger.
// Ties are broken by comparing nodes from start to finish.
// Nodes are compared by start time (ties are by end_time then id)
impl Ord for Tour {
    fn cmp(&self, other: &Self) -> Ordering {
        self.nodes.len().cmp(&other.nodes.len()).then(
            match self
                .nodes
                .iter()
                .zip(other.nodes.iter())
                .map(|(node, other_node)| {
                    self.nw
                        .node(*node)
                        .cmp_start_time(self.nw.node(*other_node))
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
