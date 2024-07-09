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

pub mod transition_cycle_neighborhood;
pub mod transition_cycle_objective;

use std::sync::Arc;

use model::network::Network;
use rapid_solve::heuristics::local_search::LocalSearchSolver;
use solution::{transition::transition_cycle::TransitionCycle, Schedule};

use self::transition_cycle_neighborhood::TransitionCycleNeighborhood;

pub struct TransitionCycleWithInfo {
    cycle: TransitionCycle,
    print_text: String,
}

impl TransitionCycleWithInfo {
    pub fn new(cycle: TransitionCycle, print_text: String) -> TransitionCycleWithInfo {
        TransitionCycleWithInfo { cycle, print_text }
    }

    pub fn get_cycle(&self) -> &TransitionCycle {
        &self.cycle
    }

    pub fn unwrap_cycle(self) -> TransitionCycle {
        self.cycle
    }

    pub fn get_print_text(&self) -> &str {
        &self.print_text
    }
}

pub fn build_transition_cycle_tsp_solver(
    schedule: &Schedule,
    network: Arc<Network>,
) -> LocalSearchSolver<TransitionCycleWithInfo> {
    let objective = Arc::new(transition_cycle_objective::build());

    let neighborhood = Arc::new(TransitionCycleNeighborhood::new(
        schedule.get_tours().clone(),
        network.clone(),
    ));

    LocalSearchSolver::with_options(
        neighborhood,
        objective,
        None,
        Some(Box::new(|_, _, _, _, _, _, _| {})), // no output
        None,
        None,
    )
}
