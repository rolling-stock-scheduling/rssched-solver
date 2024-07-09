mod transition_neighborhood;
mod transition_objective;
use std::time as stdtime;
use std::{sync::Arc, time::Instant};

use model::network::Network;
use rapid_solve::heuristics::parallel_local_search::ParallelLocalSearchSolver;
use rapid_solve::objective::{EvaluatedSolution, Objective};
use solution::{transition::Transition, Schedule};

use crate::transition_cycle_tsp;

use self::transition_neighborhood::TransitionNeighborhood;

pub struct TransitionWithInfo {
    transition: Transition,
    print_text: String,
}

impl TransitionWithInfo {
    pub fn new(transition: Transition, print_text: String) -> TransitionWithInfo {
        TransitionWithInfo {
            transition,
            print_text,
        }
    }

    pub fn get_transition(&self) -> &Transition {
        &self.transition
    }

    pub fn unwrap_transition(self) -> Transition {
        self.transition
    }

    pub fn get_print_text(&self) -> &str {
        &self.print_text
    }
}

pub fn build_transition_local_search_solver(
    schedule: &Schedule,
    network: Arc<Network>,
) -> ParallelLocalSearchSolver<TransitionWithInfo> {
    let transition_cycle_tsp_solver =
        transition_cycle_tsp::build_transition_cycle_tsp_solver(schedule, network.clone());

    let objective = Arc::new(transition_objective::build());

    let neighborhood = Arc::new(TransitionNeighborhood::new(
        schedule.get_tours().clone(),
        transition_cycle_tsp_solver,
        network.clone(),
    ));

    let function_between_steps = Box::new(
        |iteration_counter: u32,
         current_solution: &EvaluatedSolution<TransitionWithInfo>,
         previous_solution: Option<&EvaluatedSolution<TransitionWithInfo>>,
         objective: Arc<Objective<TransitionWithInfo>>,
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
