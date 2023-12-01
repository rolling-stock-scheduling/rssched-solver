pub mod swaps;

pub mod swap_factory;

pub mod local_improver;

use std::sync::Arc;

use local_improver::LocalImprover;
use local_improver::Minimizer;
use objective_framework::Objective;
use sbb_model::{config::Config, network::Network, vehicle_types::VehicleTypes};
use sbb_solution::Schedule;
use swap_factory::LimitedExchanges;
// use local_improver::TakeFirstRecursion;
// use local_improver::TakeFirstParallelRecursion;
// use local_improver::TakeAnyParallelRecursion;
use time::Duration;

use crate::Solution;

use super::Solver;

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
        // if there is not start schedule, create new empty schedule:
        let mut current_solution = self.initial_solution.clone().unwrap_or({
            let schedule = Schedule::empty(
                self.vehicles.clone(),
                self.network.clone(),
                self.config.clone(),
            );
            self.objective.evaluate(schedule)
        });

        // Phase 1: limited exchanges:
        println!("\n\n\n*** Phase 1: limited exchanges with recursion ***");
        // let segment_limit = Duration::new("3:00:00");
        // let overhead_threshold = Duration::new("0:05:00"); // tours of real-vehicle-providers are not splitted at nodes under these duration
        let only_dummy_provider = false;
        let swap_factory = LimitedExchanges::new(
            None, //Some(segment_limit),
            None, //Some(overhead_threshold),
            only_dummy_provider,
            self.network.clone(),
        );

        // let recursion_depth = 5;
        // let recursion_width = 5;
        // let soft_objective_threshold = 10.0;

        let limited_local_improver = Minimizer::new(swap_factory, self.objective.clone());
        // let limited_local_improver = TakeFirstRecursion::new(swap_factory,recursion_depth, Some(25), soft_objective_threshold);
        // let limited_local_improver = TakeFirstParallelRecursion::new(swap_factory,recursion_depth, Some(recursion_width), soft_objective_threshold);
        // let limited_local_improver = TakeAnyParallelRecursion::new(
        // swap_factory,
        // recursion_depth,
        // Some(recursion_width),
        // soft_objective_threshold,
        // );

        self.find_local_optimum(current_solution, limited_local_improver)

        // self.find_local_optimum(schedule, limited_local_improver)

        // Phase 2: less-limited exchanges:
        // println!("\n\n*** Phase 2: less-limited exchanges without recursion ***");
        // let segment_limit = Duration::new("24:00:00");
        // let swap_factory =
        // LimitedExchanges::new(Some(segment_limit), None, false, self.network.clone());

        // let unlimited_local_improver = Minimizer::new(swap_factory, self.objective.clone());
        // let unlimited_local_improver = TakeFirstRecursion::new(swap_factory,0,None,soft_objective_threshold);
        // let unlimited_local_improver = TakeFirstParallelRecursion::new(swap_factory,0,None,soft_objective_threshold);
        // let unlimited_local_improver = TakeAnyParallelRecursion::new(
        // swap_factory,
        // 0,
        // Some(recursion_width),
        // soft_objective_threshold,
        // );

        // self.find_local_optimum(current_solution, unlimited_local_improver)
    }
}

impl LocalSearch {
    fn find_local_optimum(
        &self,
        start_solution: Solution,
        local_improver: impl LocalImprover,
    ) -> Solution {
        let mut old_solution = start_solution;
        while let Some(new_solution) = local_improver.improve(&old_solution) {
            self.objective
                .print_objective_value(&new_solution.objective_value());
            println!();
            old_solution = new_solution;
        }
        old_solution
    }
}
