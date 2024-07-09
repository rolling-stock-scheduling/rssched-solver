// © 2023-2024 ETH Zurich
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// any later version.
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use std::fmt;

use model::base_types::NodeIdx;

/// A segment is a pair of non-depot node ids that represent a slice of a tour or path.
/// Depot nodes can cause unexpected behavior.
#[derive(Copy, Clone)]
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
        if self.start == self.end {
            write!(f, "[{}]", self.start)
        } else {
            write!(f, "[{}..{}]", self.start, self.end)
        }
    }
}
