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

pub mod neighborhood;
use std::sync::Arc;
use std::time::{self as stdtime, Instant};

use crate::objective;
use model::network::Network;
use rapid_solve::heuristics::parallel_local_search::ParallelLocalSearchSolver;
use rapid_solve::objective::{EvaluatedSolution, Objective};
use solution::Schedule;

use rapid_time::Duration;

use self::neighborhood::swaps::SwapInfo;
use self::neighborhood::RSSchedParallelNeighborhood;

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct ScheduleWithInfo {
    schedule: Schedule,
    last_swap_info: SwapInfo,
    print_text: String,
}

impl ScheduleWithInfo {
    pub fn new(
        schedule: Schedule,
        last_swap_info: SwapInfo,
        print_text: String,
    ) -> ScheduleWithInfo {
        ScheduleWithInfo {
            schedule,
            last_swap_info,
            print_text,
        }
    }

    pub fn get_schedule(&self) -> &Schedule {
        &self.schedule
    }

    pub fn get_last_swap_info(&self) -> SwapInfo {
        self.last_swap_info
    }

    pub fn get_print_text(&self) -> &str {
        &self.print_text
    }
}

pub fn build_local_search_solver(
    network: Arc<Network>,
) -> ParallelLocalSearchSolver<ScheduleWithInfo> {
    let objective = Arc::new(objective::build());

    let segment_limit = Duration::new("3:00:00");
    let overhead_threshold = Duration::new("0:10:00"); // tours of real-vehicle-providers are not splitted at nodes under these duration

    let neighborhood = Arc::new(RSSchedParallelNeighborhood::new(
        Some(segment_limit),
        Some(overhead_threshold),
        network,
    ));

    let function_between_steps = Box::new(
        |iteration_counter: u32,
         current_solution: &EvaluatedSolution<ScheduleWithInfo>,
         previous_solution: Option<&EvaluatedSolution<ScheduleWithInfo>>,
         objective: Arc<Objective<ScheduleWithInfo>>,
         start_time: Option<Instant>,
         _: Option<stdtime::Duration>,
         _: Option<u32>| {
            println!(
                "Iteration {} - Swap: {}",
                iteration_counter,
                current_solution.solution().get_print_text()
            );
            println!("Objective value:");
            match previous_solution {
                Some(prev_solution) => {
                    objective.print_objective_value_with_comparison(
                        current_solution.objective_value(),
                        prev_solution.objective_value(),
                    );
                }
                None => {
                    objective.print_objective_value(current_solution.objective_value());
                }
            }
            if let Some(start_time) = start_time {
                println!(
                    "elapsed time for local search: {:0.2}sec",
                    stdtime::Instant::now()
                        .duration_since(start_time)
                        .as_secs_f32()
                );
            }
            println!();
        },
    );

    ParallelLocalSearchSolver::with_options(
        neighborhood,
        objective,
        None,
        Some(function_between_steps),
        None,
        None,
    )
}
