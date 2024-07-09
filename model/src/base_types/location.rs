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

use super::LocationIdx;
use std::fmt;

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub enum Location {
    Station(LocationIdx),
    Nowhere, // distance to Nowhere is always infinity
}

impl Location {
    pub fn idx(&self) -> LocationIdx {
        match self {
            Location::Station(s) => *s,
            Location::Nowhere => panic!("Location::Nowhere has no idx."),
        }
    }

    pub fn of(idx: LocationIdx) -> Location {
        Location::Station(idx)
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Location::Station(s) => write!(f, "{}", s),
            Location::Nowhere => write!(f, "NOWHERE!"),
        }
    }
}
