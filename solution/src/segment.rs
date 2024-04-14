use std::fmt;

use model::base_types::NodeIdx;

#[derive(Debug, Clone, Copy)]

/// A segment is a pair of non-depot node ids that represent a slice of a tour or path.
/// Depot nodes can cause unexpected behavior.
pub struct Segment {
    start: NodeIdx,
    end: NodeIdx,
}

// static functions
impl Segment {
    pub fn new(start: NodeIdx, end: NodeIdx) -> Segment {
        Segment { start, end }
    }
}

// methods
impl Segment {
    pub fn start(&self) -> NodeIdx {
        self.start
    }

    pub fn end(&self) -> NodeIdx {
        self.end
    }
}

impl fmt::Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}..{}]", self.start, self.end)
    }
}
