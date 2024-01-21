pub mod local_improver;
pub mod swap_factory;
pub mod swaps;

use std::sync::Arc;
use std::time as stdtime;

use super::Solver;
use crate::one_node_per_tour::OneNodePerTour;
use crate::Solution;
use local_improver::LocalImprover;
use model::{config::Config, network::Network, vehicle_types::VehicleTypes};
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

enum SearchResult {
    Improvement(Solution),
    NoImprovement(Solution),
}

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

impl Solver for LocalSearch {
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
        // if there is no start schedule, create new empty schedule:
        let init_solution = self.initial_solution.clone().unwrap_or({
            let one_node_per_tour = OneNodePerTour::initialize(
                self.vehicles.clone(),
                self.network.clone(),
                self.config.clone(),
                self.objective.clone(),
            );
            one_node_per_tour.solve()
        });

        // let segment_limit = Duration::new("3:00:00");
        // let overhead_threshold = Duration::new("0:05:00"); // tours of real-vehicle-providers are not splitted at nodes under these duration
        let only_dummy_provider = false;

        let swap_factory = LimitedExchanges::new(
            None, //Some(segment_limit),
            None, //Some(overhead_threshold),
            only_dummy_provider,
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

        let mut current_result = Improvement(init_solution);

        while let Improvement(current_solution) = current_result {
            println!("\n* LOCAL SEARCH *");
            current_result = self.find_local_optimum(
                current_solution,
                // _minimizer.clone(),
                // _take_first.clone(),
                _take_any.clone(),
                true,
                Some(start_time),
            );
            println!("\n* DIVERSIFICATION *");
            current_result = self.diversify(current_result.unwrap(), true, Some(start_time));
        }

        current_result.unwrap()
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
                self.objective
                    .print_objective_value(new_solution.objective_value());
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

    /// Diversification: Remove each vehicle and try to find a better solution by local search with
    /// only dummy providers.
    fn diversify(
        &self,
        initial_solution: Solution,
        verbose: bool,
        start_time: Option<stdtime::Instant>,
    ) -> SearchResult {
        let initial_schedule = initial_solution.solution();
        let take_any_dummy_provier_only = TakeAnyParallelRecursion::new(
            LimitedExchanges::new(None, None, true, self.network.clone()),
            0,
            Some(5),
            self.objective.clone(),
        );

        // TODO : parallelize
        for vehicle in initial_schedule.vehicles_iter() {
            println!("\n* Remove Vehicle {}", vehicle);
            let reduced_schedule = initial_schedule.replace_vehicle_by_dummy(vehicle).unwrap();
            let reduced_solution = self.objective.evaluate(reduced_schedule);
            if verbose {
                println!("start:");
                self.objective
                    .print_objective_value(reduced_solution.objective_value());
            }
            let improved_reduced_solution = self
                .find_local_optimum(
                    reduced_solution,
                    take_any_dummy_provier_only.clone(),
                    false,
                    None,
                )
                .unwrap();

            if verbose {
                println!("end:");
                self.objective
                    .print_objective_value(improved_reduced_solution.objective_value());
                println!(
                    "elapsed time: {:0.2}",
                    stdtime::Instant::now()
                        .duration_since(start_time.unwrap())
                        .as_secs_f32()
                );
            }

            if improved_reduced_solution < initial_solution {
                println!("Diversification: found better solution");
                return Improvement(improved_reduced_solution);
            }
        }
        NoImprovement(initial_solution)
    }
}
