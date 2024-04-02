pub mod local_improver;
pub mod swap_factory;
pub mod swaps;

use std::sync::Arc;
use std::time as stdtime;

use super::Solver;
use crate::one_node_per_tour::OneNodePerTour;
use local_improver::LocalImprover;
use model::{config::Config, network::Network, vehicle_types::VehicleTypes};
use objective_framework::EvaluatedSolution;
use objective_framework::Objective;
use solution::Schedule;
use swap_factory::LimitedExchanges;

#[allow(unused_imports)]
use local_improver::Minimizer;

#[allow(unused_imports)]
use local_improver::TakeFirstRecursion;

// #[allow(unused_imports)]
// use local_improver::TakeFirstParallelRecursion;

#[allow(unused_imports)]
use local_improver::TakeAnyParallelRecursion;

pub type Solution = EvaluatedSolution<Schedule>;

enum SearchResult {
    Improvement(Solution),
    NoImprovement(Solution),
}

use time::Duration;
use SearchResult::*;

impl SearchResult {
    fn unwrap(self) -> Solution {
        match self {
            SearchResult::Improvement(solution) => solution,
            SearchResult::NoImprovement(solution) => solution,
        }
    }

    fn as_ref(&self) -> &Solution {
        match self {
            SearchResult::Improvement(solution) => solution,
            SearchResult::NoImprovement(solution) => solution,
        }
    }
}

pub struct LocalSearch {
    vehicles: Arc<VehicleTypes>,
    network: Arc<Network>,
    config: Arc<Config>,
    objective: Arc<Objective<Schedule>>,
    initial_solution: Option<Solution>,
}

impl LocalSearch {
    pub fn set_initial_solution(&mut self, solution: Solution) {
        self.initial_solution = Some(solution);
    }
}

impl Solver<Schedule> for LocalSearch {
    fn initialize(
        vehicles: Arc<VehicleTypes>,
        network: Arc<Network>,
        config: Arc<Config>,
        objective: Arc<Objective<Schedule>>,
    ) -> Self {
        Self {
            vehicles,
            network,
            config,
            objective,
            initial_solution: None,
        }
    }

    fn solve(&self) -> Solution {
        let start_time = stdtime::Instant::now();
        // if there is no start schedule, create new schedule, where each vehicle has exactly one tour and the demand is covered.
        let init_solution = self.initial_solution.clone().unwrap_or_else(|| {
            let one_node_per_tour = OneNodePerTour::initialize(
                self.vehicles.clone(),
                self.network.clone(),
                self.config.clone(),
                self.objective.clone(),
            );
            one_node_per_tour.solve()
        });

        let segment_limit = Duration::new("3:00:00");
        let overhead_threshold = Duration::new("0:05:00"); // tours of real-vehicle-providers are not splitted at nodes under these duration

        let swap_factory = LimitedExchanges::new(
            Some(segment_limit),
            // None,
            Some(overhead_threshold),
            // None,
            false,
            self.network.clone(),
        );

        let _minimizer = Minimizer::new(swap_factory.clone(), self.objective.clone());

        let recursion_depth = 0;
        let recursion_width = 5;

        let _take_first = TakeFirstRecursion::new(
            swap_factory.clone(),
            recursion_depth,
            Some(recursion_width),
            self.objective.clone(),
        );

        /*
        let local_improver = TakeFirstParallelRecursion::new(
            swap_factory,
            recursion_depth,
            Some(recursion_width),
            soft_objective_threshold,
        );
        */

        let _take_any = TakeAnyParallelRecursion::new(
            swap_factory,
            recursion_depth,
            Some(recursion_width),
            self.objective.clone(),
        );

        self.find_local_optimum(
            init_solution,
            // _minimizer.clone(),
            // _take_first.clone(),
            _take_any.clone(),
            true,
            Some(start_time),
        )
        .unwrap()
    }
}

impl LocalSearch {
    fn find_local_optimum(
        &self,
        start_solution: Solution,
        mut local_improver: impl LocalImprover,
        verbose: bool,
        start_time: Option<stdtime::Instant>,
    ) -> SearchResult {
        let mut result = NoImprovement(start_solution);
        while let Some(new_solution) = local_improver.improve(result.as_ref()) {
            #[cfg(debug_assertions)]
            new_solution.solution().verify_consistency();
            if verbose {
                self.objective.print_objective_value_with_comparison(
                    new_solution.objective_value(),
                    result.as_ref().objective_value(),
                );
                if let Some(start_time) = start_time {
                    println!(
                        "elapsed time for local search: {:0.2}sec",
                        stdtime::Instant::now()
                            .duration_since(start_time)
                            .as_secs_f32()
                    );
                }
                println!();
            }
            result = Improvement(new_solution);
        }
        result
    }
}
