pub mod swaps;

pub mod swap_factory;

pub mod local_improver;

use crate::config::Config;
use crate::network::Network;
use crate::schedule::Schedule;
use crate::solver::Solver;
use crate::time::Duration;
use crate::units::Units;
use std::sync::Arc;

use local_improver::LocalImprover;
use swap_factory::LimitedExchanges;
// use local_improver::Minimizer;
// use local_improver::TakeFirstRecursion;
// use local_improver::TakeFirstParallelRecursion;
use local_improver::TakeAnyParallelRecursion;

pub struct LocalSearch {
    config: Arc<Config>,
    units: Arc<Units>,
    nw: Arc<Network>,
    initial_schedule: Option<Schedule>,
}

impl LocalSearch {
    pub(crate) fn set_initial_schedule(&mut self, schedule: Schedule) {
        self.initial_schedule = Some(schedule);
    }
}

impl Solver for LocalSearch {
    fn initialize(config: Arc<Config>, units: Arc<Units>, nw: Arc<Network>) -> LocalSearch {
        LocalSearch {
            config,
            units,
            nw,
            initial_schedule: None,
        }
    }

    fn solve(&self) -> Schedule {
        // if there is not start schedule, create new empty schedule:
        let mut schedule: Schedule = match &self.initial_schedule {
            Some(sched) => sched.clone(),
            None => Schedule::initialize(self.config.clone(), self.units.clone(), self.nw.clone()),
        };

        // Phase 1: limited exchanges:
        println!("\n\n\n*** Phase 1: limited exchanges with recursion ***");
        let segment_limit = Duration::new("3:00");
        let overhead_threshold = Duration::new("0:40"); // tours of real-unit-providers are not splitted at nodes under these duration
        let only_dummy_provider = false;
        let swap_factory = LimitedExchanges::new(
            Some(segment_limit),
            Some(overhead_threshold),
            only_dummy_provider,
            self.nw.clone(),
        );

        let recursion_depth = 5;
        let recursion_width = 5;
        let soft_objective_threshold = 10.0;

        // let limited_local_improver = Minimizer::new(swap_factory);
        // let limited_local_improver = TakeFirstRecursion::new(swap_factory,recursion_depth, Some(25), soft_objective_threshold);
        // let limited_local_improver = TakeFirstParallelRecursion::new(swap_factory,recursion_depth, Some(recursion_width), soft_objective_threshold);
        let limited_local_improver = TakeAnyParallelRecursion::new(
            swap_factory,
            recursion_depth,
            Some(recursion_width),
            soft_objective_threshold,
        );

        schedule = self.find_local_optimum(schedule, limited_local_improver);
        // self.find_local_optimum(schedule, limited_local_improver)

        // Phase 2: less-limited exchanges:
        println!("\n\n*** Phase 2: less-limited exchanges without recursion ***");
        let segment_limit = Duration::new("24:00");
        let swap_factory = LimitedExchanges::new(Some(segment_limit), None, false, self.nw.clone());

        // let unlimited_local_improver = Minimizer::new(swap_factory);
        // let unlimited_local_improver = TakeFirstRecursion::new(swap_factory,0,None,soft_objective_threshold);
        // let unlimited_local_improver = TakeFirstParallelRecursion::new(swap_factory,0,None,soft_objective_threshold);
        let unlimited_local_improver = TakeAnyParallelRecursion::new(
            swap_factory,
            0,
            Some(recursion_width),
            soft_objective_threshold,
        );

        self.find_local_optimum(schedule, unlimited_local_improver)
    }
}

impl LocalSearch {
    fn find_local_optimum(
        &self,
        schedule: Schedule,
        local_improver: impl LocalImprover,
    ) -> Schedule {
        let mut old_schedule = schedule;
        while let Some(new_schedule) = local_improver.improve(&old_schedule) {
            new_schedule
                .objective_value()
                .print(Some(&old_schedule.objective_value()));
            // schedule.print();
            if new_schedule.number_of_dummy_units() < 5 {
                for dummy in new_schedule.dummy_iter() {
                    println!("{}: {}", dummy, new_schedule.tour_of(dummy));
                }
            }
            println!();
            old_schedule = new_schedule;
        }
        old_schedule
    }
}
