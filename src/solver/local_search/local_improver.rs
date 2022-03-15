use crate::schedule::Schedule;

use super::swap_factory::SwapFactory;
use crate::schedule::objective::ObjectiveValue;

use rayon::prelude::*;
use rayon::iter::ParallelBridge;
use std::sync::mpsc::channel;


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


pub(crate) struct TakeFirst<F: SwapFactory + Send> {
    swap_factory: F,
}


impl<F: SwapFactory + Send> TakeFirst<F> {
    pub(crate) fn new(swap_factory: F) -> TakeFirst<F> {
        TakeFirst{swap_factory}
    }
}

impl<F: SwapFactory + Send> LocalImprover for TakeFirst<F> {
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


pub(crate) struct TakeFirstRecursion<F: SwapFactory + Send + Sync> {
    swap_factory: F,
    recursion_depth: u8,
    recursion_width: Option<usize>, // number of schedule that are considered for recursion (the one with best value are taken)
}


impl<F: SwapFactory + Send + Sync> LocalImprover for TakeFirstRecursion<F> {
    fn improve(&self, schedule: &Schedule) -> Option<Schedule> {
        let old_objective = schedule.objective_value();
        self.improve_recursion(vec!(schedule.clone()), old_objective, self.recursion_depth)
    }
}

impl<F: SwapFactory + Send + Sync> TakeFirstRecursion<F> {

    pub(crate) fn new(swap_factory: F, recursion_depth: u8, recursion_width: Option<usize>) -> TakeFirstRecursion<F> {
        TakeFirstRecursion{swap_factory, recursion_depth, recursion_width}
    }


    fn improve_recursion(&self, schedules: Vec<Schedule>, objective_to_beat: ObjectiveValue, remaining_recursion: u8) -> Option<Schedule> {
        // let swap_iterator = schedules.iter().flat_map(|sched| self.swap_factory.create_swap_iterator(sched).map(move |swap| (swap, sched)));
        // let swap_iterator = schedules.par_iter().flat_map(|sched| self.swap_factory.create_swap_iterator(sched).par_bridge().map(move |swap| (swap, sched)));
        println!("schedules beginning of recursion: {}", schedules.len());
        let mut schedule_collection: Vec<Vec<Schedule>> = Vec::new();
        let mut result: Option<Schedule> = None;
        rayon::scope(|s| {
            let mut found_senders = Vec::new();
            let (success_sender, success_receiver) = channel();
            let (failure_sender, failure_receiver) = channel();

            for sched in schedules.iter() {
                let (found_sender, found_receiver) = channel();
                found_senders.push(found_sender);
                let succ_sender = success_sender.clone();
                let fail_sender = failure_sender.clone();
                s.spawn(move |_| {
                    let mut schedules: Vec<Schedule> = Vec::new();
                    let mut counter_limit = usize::MAX;
                    let result = self.swap_factory.create_swap_iterator(sched).enumerate()
                    .filter_map(|(i,swap)| {
                        swap.apply(sched).ok().map(move |new_sched| (i, new_sched))
                    }).find(|(i,new_sched)| {
                        schedules.push(new_sched.clone());
                        if let Ok(best_i) = found_receiver.try_recv() {
                            counter_limit = best_i;
                        }
                        new_sched.objective_value() < objective_to_beat || *i > counter_limit
                    });

                    match result {

                        Some(pair) => {
                            if pair.1.objective_value() < objective_to_beat {
                                succ_sender.send(pair).unwrap();
                            }
                            // if there is a Some result but the objective is not better, that means
                            // another thread was successful with smaller index. So there is nothing
                            // left to do for this thread.
                        }
                        None => {
                            fail_sender.send(schedules).unwrap();
                        }
                    }
                });
            }

            drop(success_sender);
            drop(failure_sender);


            let mut best_i = usize::MAX;
            while let Ok((i, new_sched)) = success_receiver.recv() {
                if result.is_none() || i < best_i || (i==best_i && new_sched.objective_value() < result.as_ref().unwrap().objective_value()) {
                    // index is smaller or if there is a tie the new schedule has better objective
                    best_i = i;
                    for s in found_senders.iter() {
                        s.send(best_i).ok();
                    }
                    result = Some(new_sched);
                }
            }
            if result.is_none() {
                for v in failure_receiver.into_iter() {
                    schedule_collection.push(v);
                }
            }
            println!("size of schedule_collection: {}", schedule_collection.len());
        });


        if result.is_none() {
            // let swap_iterator = schedules.par_iter().flat_map(|sched| self.swap_factory.create_swap_iterator(sched).par_bridge().map(move |swap| (swap, sched)));
            // let mut schedule_collection: Vec<Schedule> = swap_iterator.filter_map(|(swap,old_sched)| swap.apply(old_sched).ok()).collect();
            let number_of_schedules: usize = schedule_collection.iter().map(|v| v.len()).sum();
            println!("No improvement found after {} swaps.", number_of_schedules);

            if remaining_recursion > 0 {
                println!("collection: {}", schedule_collection.len());
                let mut schedules_for_recursion = Vec::new();
                let compare = |sched1:&Schedule, sched2:&Schedule| sched1.cmp(sched2);
                if let Some(width) = self.recursion_width {
                    for mut schedules in schedule_collection.into_iter() {
                        let width = width.min(schedules.len());
                        schedules.select_nth_unstable_by(width-1, compare);
                        schedules.truncate(width);
                        schedules_for_recursion.append(&mut schedules);
                    }
                }
                schedules_for_recursion.sort_by(compare);

                println!("Going into recursion. Remaining depth: {}. Schedule-count: {}", remaining_recursion, schedules_for_recursion.len());

                self.improve_recursion(schedules_for_recursion, objective_to_beat, remaining_recursion-1)
            } else {
                println!("No recursion-depth left.");
                None
            }
        } else {
            println!("Improvement found.");
            result
        }
    }
}

