use crate::base_types::{NodeId, UnitId};
use crate::schedule::Schedule;
use crate::schedule::path::Segment;
use std::fmt;
use std::rc::Rc;

use itertools::Itertools;

/// An elementary modification. Defining the "neighborhood" for the local search.
pub(crate) trait Swap: fmt::Display {
    fn apply(&self, schedule: &Schedule) -> Result<Schedule, String>;
    // TODO maybe add something like, get_improvement()
}

/// Computes for a given schedule all Swaps in the neighborhood.
// pub(crate) trait SwapIterator: Iterator<Item=PathExchange> {
pub(crate) trait SwapIterator {
    fn new(schedule: Rc<Schedule>) -> Self;
}

/// Computes for a given schedule the best new schedule that has better objective function.
/// Returns None if there is no better schedule in the neighborhood.
pub(crate) trait LocalImprover {
    fn improve(&self, schedule: &Schedule) -> Option<Schedule>;
}




/////////////////////////////////////////////////////////////
////////////////////////// Swaps ////////////////////////////
/////////////////////////////////////////////////////////////

/// Removes the path from the provider's Tour and insert it into the receiver's Tour.
/// All removed nodes that are removed from receiver's Tour (due to conflicts) are tried to insert conflict-free into
/// the provider's Tour.
pub(crate) struct PathExchange {
    segment: Segment,
    provider: UnitId,
    receiver: UnitId,
}

impl PathExchange {
    pub(crate) fn new(segment: Segment, provider: UnitId, receiver: UnitId) -> PathExchange {
        PathExchange{segment, provider, receiver}
    }
}

impl Swap for PathExchange {
    fn apply(&self, schedule: &Schedule) -> Result<Schedule, String> {

        let (intermediate_schedule, new_dummy_opt) = schedule.override_reassign(self.segment, self.provider, self.receiver)?;

        match new_dummy_opt {
            None => Ok(intermediate_schedule),
            Some(new_dummy) =>
                if schedule.is_dummy(self.provider) && !intermediate_schedule.is_dummy(self.provider) {
                    // provider was dummy but got removed -> so no need for fit_reassign
                    Ok(intermediate_schedule)
                } else {
                    Ok(intermediate_schedule.fit_reassign_all(new_dummy, self.provider)?)
                }
        }
    }
}

impl fmt::Display for PathExchange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "swap {} from {} to {}", self.segment, self.provider, self.receiver)
    }
}



/////////////////////////////////////////////////////////////
//////////////////////// SwapIterator ///////////////////////
/////////////////////////////////////////////////////////////

// pub(crate) struct AllExchanges {
    // swap_iterator: Box<dyn Iterator<Item=PathExchange>>
// }

// impl Iterator for AllExchanges {
    // type Item = Box<PathExchange>;
    // fn next(&mut self) -> Option<Self::Item> {
        // self.swap_iterator.next().map(|s| Box::new(s))
    // }
// }

// impl SwapIterator for AllExchanges {
    
    // fn new(schedule: Schedule) -> AllExchanges {
     
        // AllExchanges{swap_iterator: Box::new(swap_iterator)}
    // }
// }


/////////////////////////////////////////////////////////////
/////////////////////// Local Improver //////////////////////
/////////////////////////////////////////////////////////////

// pub(crate) struct Minimizer<F: SwapFactory> {
    // swap_factory: F
// }

// impl<F: SwapFactory> Minimizer<F> {
    // pub(crate) fn new(swap_factory: F) -> Minimizer<F> {
        // Minimizer{swap_factory}
    // }
// }

// impl<F: SwapFactory> LocalImprover for Minimizer<F> {
    // fn improve(&self, schedule: &Schedule) -> Option<Schedule> {
        // let swaps = self.swap_factory.create_swaps(schedule);
        // let (best_objective_value, best_schedule) = swaps.iter()
            // .filter_map(|swap| swap.apply(schedule).ok())
            // .map(|sched| (sched.objective_value(),sched))
            // .min_by(|(o1,_), (o2,_)| o1.partial_cmp(o2).unwrap()).unwrap();

        // if best_objective_value < schedule.objective_value() {Some(best_schedule)
        // } else {None}
    // }
// }



// pub(crate) struct Greedy<F: SwapFactory> {
//     swap_factory: F
// }

// impl<F: SwapFactory> Greedy<F> {
//     pub(crate) fn new(swap_factory: F) -> Greedy<F> {
//         Greedy{swap_factory}
//     }
// }

// impl<F: SwapFactory> LocalImprover for Greedy<F> {
//     fn improve(&self, schedule: &Schedule) -> Option<Schedule> {
//         let old_objective = schedule.objective_value();
//         let swaps = self.swap_factory.create_swaps(schedule);
//         swaps.iter()
//             .filter_map(|swap| swap.apply(schedule).ok())
//             .find(|sched| sched.objective_value() < old_objective)
//     }
// }


pub(crate) struct Greedy {
}

impl Greedy {
    pub(crate) fn new() -> Greedy {
        Greedy{}
    }
}

impl LocalImprover for Greedy {
    fn improve(&self, schedule: &Schedule) -> Option<Schedule> {
        let old_objective = schedule.objective_value();
        // let mut swap_iter = AllExchanges::new(schedule.clone());
        let provider_info: Vec<(UnitId,Vec<_>)> = schedule.dummy_iter().chain(schedule.real_units_iter())
            .map(|prov| (prov, schedule.tour_of(prov).nodes_iter().collect())).collect();


        let swap_iterator = provider_info.iter().flat_map(|(provider, nodes)|
                nodes.iter().tuple_combinations().chain(nodes.iter().map(|n| (n, n)))
                .map(|(&s, &e)| Segment::new(*s, *e))
                .filter(|seg| schedule.tour_of(*provider).removable(*seg))
                .flat_map(move |segment| 
                    schedule.real_units_iter().chain(schedule.dummy_iter())
                    .filter(move |&u| u != *provider && schedule.conflict(segment, u).is_ok())
                    .map(move |receiver|
                    PathExchange::new(segment, *provider, receiver)))
            );


        swap_iterator
            .filter_map(|swap| swap.apply(schedule).ok())
            .find(|sched| sched.objective_value() < old_objective)
    }
}


