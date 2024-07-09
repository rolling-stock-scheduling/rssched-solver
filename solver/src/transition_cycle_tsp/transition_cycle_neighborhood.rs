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

use std::sync::Arc;

use im::HashMap;
use model::{base_types::VehicleIdx, network::Network};
use rapid_solve::heuristics::common::Neighborhood;
use solution::tour::Tour;

use super::TransitionCycleWithInfo;

pub struct TransitionCycleNeighborhood {
    tours: HashMap<VehicleIdx, Tour>,
    network: Arc<Network>,
}

impl TransitionCycleNeighborhood {
    pub fn new(
        tours: HashMap<VehicleIdx, Tour>,
        network: Arc<Network>,
    ) -> TransitionCycleNeighborhood {
        TransitionCycleNeighborhood { tours, network }
    }
}

impl Neighborhood<TransitionCycleWithInfo> for TransitionCycleNeighborhood {
    fn neighbors_of<'a>(
        &'a self,
        transition_cycle_with_info: &'a TransitionCycleWithInfo,
    ) -> Box<dyn Iterator<Item = TransitionCycleWithInfo> + Send + Sync + 'a> {
        let cycle = transition_cycle_with_info.get_cycle();
        let cycle_length = cycle.len();
        Box::new((0..cycle_length - 2).flat_map(move |i| {
            (i + 1..cycle_length - 1).flat_map(move |j| {
                (j + 1..cycle_length).map(move |k| {
                    TransitionCycleWithInfo::new(
                        cycle.three_opt(i, j, k, &self.tours, &self.network),
                        format!("3Opt: {}, {}, {}", i, j, k),
                    )
                })
            })
        }))
    }
}
