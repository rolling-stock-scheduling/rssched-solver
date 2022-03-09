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
use local_improver::{LocalImprover, Greedy, Minimizer};








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
        // let swap_factory = AllExchanges::new();
        // let swap_factory = LimitedExchanges::new(None, self.nw.clone());
        let segment_limit = Duration::new("6:00");
        let overhead_threshold = Duration::new("0:30");
        let swap_factory = LimitedExchanges::new(Some(segment_limit), Some(overhead_threshold) , self.nw.clone());
        let local_improver = Greedy::new(swap_factory);
        // let local_improver = Minimizer::new(swap_factory);

        let mut schedule = Schedule::initialize(self.loc.clone(), self.units.clone(), self.nw.clone());

        let optimal = self.nw.minimal_overhead();
        while let Some(sched) = local_improver.improve(&schedule) {
            schedule = sched;
            println!("min_overhead: {}", optimal);
            schedule.objective_value().print();
            // schedule.print();
            if schedule.number_of_dummy_units() < 5 {
                for dummy in schedule.dummy_iter(){
                    println!("{}: {}", dummy, schedule.tour_of(dummy));
                }
            }
            println!("");
        }
        schedule
    }
}
