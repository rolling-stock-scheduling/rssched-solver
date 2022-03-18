pub mod swaps;

pub mod swap_factory;

pub mod local_improver;

use crate::locations::Locations;
use crate::units::Units;
use crate::network::Network;
use crate::solver::Solver;
use crate::schedule::Schedule;
use crate::time::Duration;
use std::sync::Arc;

use swap_factory::LimitedExchanges;
use local_improver::{LocalImprover, TakeFirstRecursion, TakeAnyParallelRecursion, TakeFirstParallelRecursion, Minimizer};
use super::greedy_2::Greedy2;








pub struct LocalSearch1 {
    loc: Arc<Locations>,
    units: Arc<Units>,
    nw: Arc<Network>
}

impl Solver for LocalSearch1 {

    fn initialize(loc: Arc<Locations>, units: Arc<Units>, nw: Arc<Network>) -> LocalSearch1 {
        LocalSearch1{loc, units, nw}
    }

    fn solve(&self) -> Schedule {
        // empty schedule:
        // let mut schedule = Schedule::initialize(self.loc.clone(), self.units.clone(), self.nw.clone());


        // greedy schedule:
        let greedy = Greedy2::initialize(self.loc.clone(), self.units.clone(), self.nw.clone());
        let mut schedule = greedy.solve();


        // load SBB-schedule:
        // let mut schedule = Schedule::load_from_csv("test_instances/21-10-tage-2/SBB_leistungsketten.csv", self.loc.clone(), self.units.clone(), self.nw.clone());







        // Phase 1: limited exchanges:
        println!("\n\n\n*** Phase 1: limited exchanges with recursion ***");
        let segment_limit = Duration::new("3:00");
        let overhead_threshold = Duration::new("0:40"); // tours of real-unit-providers are not splitted at nodes under these duration
        let only_dummy_provider = false;
        let swap_factory = LimitedExchanges::new(Some(segment_limit), Some(overhead_threshold), only_dummy_provider, self.nw.clone());

        let recursion_depth = 5;
        let recursion_width = 5;

        // let limited_local_improver = Minimizer::new(swap_factory);
        // let limited_local_improver = TakeFirstRecursion::new(swap_factory,recursion_depth, Some(25));
        // let limited_local_improver = TakeFirstParallelRecursion::new(swap_factory,recursion_depth, Some(recursion_width));
        let limited_local_improver = TakeAnyParallelRecursion::new(swap_factory,recursion_depth, Some(recursion_width));

        schedule = self.find_local_optimum(schedule, limited_local_improver);
        // self.find_local_optimum(schedule, limited_local_improver)


        // Phase 2: less-limited exchanges:
        println!("\n\n*** Phase 2: less-limited exchanges without recursion ***");
        let segment_limit = Duration::new("24:00");
        let swap_factory = LimitedExchanges::new(Some(segment_limit), None, false, self.nw.clone());

        // let unlimited_local_improver = Minimizer::new(swap_factory);
        // let unlimited_local_improver = TakeFirstRecursion::new(swap_factory,0,None);
        // let unlimited_local_improver = TakeFirstParallelRecursion::new(swap_factory,0,None);
        let unlimited_local_improver = TakeAnyParallelRecursion::new(swap_factory,0,None);

        self.find_local_optimum(schedule, unlimited_local_improver)

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
            println!();
            old_schedule = new_schedule;
        }
        old_schedule

    }
}
