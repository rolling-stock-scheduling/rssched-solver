use std::fmt;

use model::base_types::NodeId;

#[derive(Debug, Clone, Copy)]

/// A segment is a pair of node ids that represent a slice of a tour or path.
pub struct Segment {
    start: NodeId,
    end: NodeId,
}

// static functions
impl Segment {
    pub fn new(start: NodeId, end: NodeId) -> Segment {
        Segment { start, end }
    }
}

// methods
impl Segment {
    pub fn start(&self) -> NodeId {
        self.start
    }

    pub fn end(&self) -> NodeId {
        self.end
    }
}

impl fmt::Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}..{}]", self.start, self.end)
    }
}
