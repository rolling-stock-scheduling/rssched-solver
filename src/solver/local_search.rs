pub mod swaps;
use swaps::PathExchange;

pub mod swap_factory;

pub mod local_improver;

use crate::locations::Locations;
use crate::units::Units;
use crate::network::Network;
use crate::solver::Solver;
use crate::schedule::Schedule;
use crate::time::Duration;
use std::rc::Rc;

use swap_factory::{LimitedExchanges, AllExchanges};
use local_improver::{LocalImprover, Minimizer, TakeFirst, TakeFirstRecursion};
use super::greedy_1::Greedy1;








pub struct LocalSearch1 {
    loc: Rc<Locations>,
    units: Rc<Units>,
    nw: Rc<Network>
}

impl Solver for LocalSearch1 {

    fn initialize(loc: Rc<Locations>, units: Rc<Units>, nw: Rc<Network>) -> LocalSearch1 {
        LocalSearch1{loc, units, nw}
    }

    fn solve(&self) -> Schedule {
        // let mut schedule = Schedule::initialize(self.loc.clone(), self.units.clone(), self.nw.clone());
        let greedy = Greedy1::initialize(self.loc.clone(), self.units.clone(), self.nw.clone());
        let mut schedule = greedy.solve();







        // Phase 1: limited exchanges:
        let segment_limit = Duration::new("3:00");
        let overhead_threshold = Duration::new("0:30"); // tours of real-unit-providers are not splitted at nodes under these duration
        let only_dummy_provider = false;
        let swap_factory = LimitedExchanges::new(Some(segment_limit), Some(overhead_threshold), only_dummy_provider, self.nw.clone());

        let recursion_depth = 3;
        let recursion_width = 50;
        let limited_local_improver = TakeFirstRecursion::new(swap_factory,recursion_depth, Some(recursion_width));
        // let limited_local_improver = TakeFirst::new(swap_factory);

        schedule = self.find_local_optimum(schedule, limited_local_improver);


        // Phase 2: unlimited exchanges:
        // let swap_factory = AllExchanges::new();
        let swap_factory = LimitedExchanges::new(None, None, false, self.nw.clone());
        // let unlimited_local_improver = TakeFirst::new(swap_factory);
        let unlimited_local_improver = TakeFirstRecursion::new(swap_factory,1,Some(2));

        schedule = self.find_local_optimum(schedule, unlimited_local_improver);



        schedule

    }
}

impl LocalSearch1 {
    fn find_local_optimum(&self, schedule: Schedule, local_improver: impl LocalImprover) -> Schedule {
        let mut old_schedule = schedule;
        while let Some(new_schedule) = local_improver.improve(&old_schedule) {
            new_schedule.objective_value().print();
            // schedule.print();
            if new_schedule.number_of_dummy_units() < 5 {
                for dummy in new_schedule.dummy_iter(){
                    println!("{}: {}", dummy, new_schedule.tour_of(dummy));
                }
            }
            println!("");
            old_schedule = new_schedule;
        }
        old_schedule

    }
}
