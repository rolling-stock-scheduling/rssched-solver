use crate::schedule::Schedule;

use super::swap_factory::SwapFactory;
use super::swaps::Swap;
use crate::schedule::objective::ObjectiveValue;

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
        let mut counter = 0;
        let (best_objective_value, best_schedule) =
            swap_iterator.filter_map(|swap| {counter += 1; swap.apply(schedule).ok()})
            .map(|sched| (sched.objective_value(),sched))
            .min_by(|(o1,_), (o2,_)| o1.partial_cmp(o2).unwrap()).unwrap();

        if best_objective_value < schedule.objective_value() {
            println!("Improvement found after {} swaps.", counter);
            Some(best_schedule)
        } else {
            println!("No improvement found after {} swaps.", counter);
            None
        }
    }
}



///////////////////////////////////////////////////////////
////////////////////// TakeFirst //////////////////////////
///////////////////////////////////////////////////////////


pub(crate) struct TakeFirst<F: SwapFactory> {
    swap_factory: F,
}


impl<F: SwapFactory> TakeFirst<F> {
    pub(crate) fn new(swap_factory: F) -> TakeFirst<F> {
        TakeFirst{swap_factory}
    }
}

impl<F: SwapFactory> LocalImprover for TakeFirst<F> {
    fn improve(&self, schedule: &Schedule) -> Option<Schedule> {
        let old_objective = schedule.objective_value();

        let swap_iterator = self.swap_factory.create_swap_iterator(schedule);

        // swap_iterator.filter_map(|swap| swap.apply(schedule).ok())
            // .find(|sched| sched.objective_value() < old_objective)
        let mut counter = 0;
        let result = swap_iterator.filter_map(|swap| {counter += 1; swap.apply(schedule).ok()})
            .find(|sched| sched.objective_value() < old_objective);
        if result.is_none() {
            println!("No improvement found after {} swaps.", counter);
        } else {
            println!("Improvement found after {} swaps.", counter);

        }

        result

    }
}

///////////////////////////////////////////////////////////
///////////////// TakeFirstRecursion //////////////////////
///////////////////////////////////////////////////////////


pub(crate) struct TakeFirstRecursion<F: SwapFactory> {
    swap_factory: F,
    recursion_depth: u8,
    recursion_width: Option<usize>, // number of schedule that are considered for recursion (the one with best value are taken)
}


impl<F: SwapFactory> TakeFirstRecursion<F> {
    pub(crate) fn new(swap_factory: F, recursion_depth: u8, recursion_width: Option<usize>) -> TakeFirstRecursion<F> {
        TakeFirstRecursion{swap_factory, recursion_depth, recursion_width}
    }
}

impl<F: SwapFactory> LocalImprover for TakeFirstRecursion<F> {
    fn improve(&self, schedule: &Schedule) -> Option<Schedule> {
        let old_objective = schedule.objective_value();
        self.improve_recursion(vec!(schedule.clone()), old_objective, self.recursion_depth)
    }
}

impl<F: SwapFactory> TakeFirstRecursion<F> {
    fn improve_recursion(&self, schedules: Vec<Schedule>, objective_to_beat: ObjectiveValue, remaining_recursion: u8) -> Option<Schedule> {
        let swap_iterator = schedules.iter().flat_map(|sched| self.swap_factory.create_swap_iterator(sched).map(move |swap| (swap, sched)));

        let mut counter = 0;
        let mut schedule_collection: Vec<Schedule> = Vec::new();

        let result = swap_iterator.filter_map(|(swap,old_sched)| {
            counter += 1;
            swap.apply(old_sched).ok()
        }).find(|new_sched| {
            schedule_collection.push(new_sched.clone());
            new_sched.objective_value() < objective_to_beat
        });

        if result.is_none() {
            println!("No improvement found after {} swaps.", counter);

            if remaining_recursion > 0 {
                let compare = |sched1:&Schedule, sched2:&Schedule| sched1.objective_value().cmp(&sched2.objective_value());
                if let Some(width) = self.recursion_width {
                    let width = width.min(schedule_collection.len());
                    schedule_collection.select_nth_unstable_by(width-1, compare);
                    schedule_collection.truncate(width);
                }
                
                println!("Going into recursion. Remaining depth: {}. Schedule-count: {}", remaining_recursion, schedule_collection.len());
                schedule_collection.sort_by(compare);
                
                self.improve_recursion(schedule_collection, objective_to_beat, remaining_recursion-1)
            } else {
                println!("No recursion-depth left.");
                None
            }
        } else {
            println!("Improvement found after {} swaps.", counter);
            result
        }
    }
}

