use crate::schedule::Schedule;

use super::swap_factory::SwapFactory;
use super::swaps::Swap;

/// Computes for a given schedule the best new schedule that has better objective function.
/// Returns None if there is no better schedule in the neighborhood.
pub(crate) trait LocalImprover {
    fn improve(&self, schedule: &Schedule) -> Option<Schedule>;
}



///////////////////////////////////////////////////////////
////////////////////// Minimizer //////////////////////////
///////////////////////////////////////////////////////////

pub(crate) struct Minimizer<F: SwapFactory> {
    swap_factory: F
}

impl<F: SwapFactory> Minimizer<F> {
    pub(crate) fn new(swap_factory: F) -> Minimizer<F> {
        Minimizer{swap_factory}
    }
}

impl<F: SwapFactory> LocalImprover for Minimizer<F> {
    fn improve(&self, schedule: &Schedule) -> Option<Schedule> {
        let swap_iterator = self.swap_factory.create_swap_iterator(schedule);
        let (best_objective_value, best_schedule) =
            swap_iterator.filter_map(|swap| swap.apply(schedule).ok())
            .map(|sched| (sched.objective_value(),sched))
            .min_by(|(o1,_), (o2,_)| o1.partial_cmp(o2).unwrap()).unwrap();

        if best_objective_value < schedule.objective_value() {Some(best_schedule)} else {None}
    }
}



///////////////////////////////////////////////////////////
//////////////////////// Greedy ///////////////////////////
///////////////////////////////////////////////////////////


pub(crate) struct Greedy<F: SwapFactory> {
    swap_factory: F,
}


impl<F: SwapFactory> Greedy<F> {
    pub(crate) fn new(swap_factory: F) -> Greedy<F> {
        Greedy{swap_factory}
    }
}

impl<F: SwapFactory> LocalImprover for Greedy<F> {
    fn improve(&self, schedule: &Schedule) -> Option<Schedule> {
        let old_objective = schedule.objective_value();

        let swap_iterator = self.swap_factory.create_swap_iterator(schedule);

        swap_iterator.filter_map(|swap| swap.apply(schedule).ok())
            .find(|sched| sched.objective_value() < old_objective)

    }
}

