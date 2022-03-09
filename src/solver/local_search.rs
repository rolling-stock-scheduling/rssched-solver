pub mod swaps;
use swaps::PathExchange;

pub mod swap_factory;

pub mod local_improver;

use crate::locations::Locations;
use crate::units::Units;
use crate::network::Network;
use crate::solver::Solver;
use crate::schedule::Schedule;
use std::rc::Rc;

use swap_factory::AllExchanges;
use local_improver::{LocalImprover, Greedy};








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
        let swap_factory = AllExchanges::new();
        let local_improver = Greedy::new(swap_factory);
        // let local_improver = Minimizer::new(swap_factory);

        let mut schedule = Schedule::initialize(self.loc.clone(), self.units.clone(), self.nw.clone());

        let optimal = self.nw.minimal_overhead();
        while let Some(sched) = local_improver.improve(&schedule) {
            schedule = sched;
            println!("");
            println!("min_overhead: {}", optimal);
            schedule.objective_value().print();
            // schedule.print();
            if schedule.number_of_dummy_units() < 10 {
                for dummy in schedule.dummy_iter(){
                    println!("{}: {}", dummy, schedule.tour_of(dummy));
                }
            }
        }
        schedule
    }
}
