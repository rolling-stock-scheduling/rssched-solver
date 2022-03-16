use crate::schedule::Schedule;

use super::swap_factory::SwapFactory;
use crate::schedule::objective::ObjectiveValue;

use rayon::prelude::*;
use rayon::iter::ParallelBridge;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};


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
        let swap_iterator = self.swap_factory.create_swap_iterator(schedule).par_bridge();
        let (best_objective_value, best_schedule) =
            swap_iterator.filter_map(|swap| swap.apply(schedule).ok())
            .map(|sched| (sched.objective_value(),sched))
            .min_by(|(o1,_), (o2,_)| o1.partial_cmp(o2).unwrap()).unwrap();

        if best_objective_value < schedule.objective_value() {
            Some(best_schedule)
        } else {
            None
        }
    }
}


///////////////////////////////////////////////////////////
/////////////////// TakeAnyRecursion //////////////////////
///////////////////////////////////////////////////////////


pub(crate) struct TakeAnyRecursion<F: SwapFactory + Send + Sync> {
    swap_factory: F,
    recursion_depth: u8,
    recursion_width: Option<usize>, // number of schedule that are considered for recursion (the one with best value are taken)
}


impl<F: SwapFactory + Send + Sync> LocalImprover for TakeAnyRecursion<F> {
    fn improve(&self, schedule: &Schedule) -> Option<Schedule> {
        let old_objective = schedule.objective_value();
        self.improve_recursion(vec!(schedule.clone()), old_objective, self.recursion_depth)
    }
}

impl<F: SwapFactory + Send + Sync> TakeAnyRecursion<F> {

    pub(crate) fn new(swap_factory: F, recursion_depth: u8, recursion_width: Option<usize>) -> TakeAnyRecursion<F> {
        TakeAnyRecursion{swap_factory, recursion_depth, recursion_width}
    }


    fn improve_recursion(&self, schedules: Vec<Schedule>, objective_to_beat: ObjectiveValue, remaining_recursion: u8) -> Option<Schedule> {

        println!("Recursion: remaining depth: {}. schedule-count: {}", remaining_recursion, schedules.len());
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
                    let found_receiver_mutex = Arc::new(Mutex::new(found_receiver));

                    let mut schedules: Vec<Schedule> = Vec::new();
                    let schedules_mutex: Arc<Mutex<&mut Vec<Schedule>>> = Arc::new(Mutex::new(&mut schedules));

                    let result = self.swap_factory.create_swap_iterator(sched).par_bridge()
                    .filter_map(|swap| {
                        swap.apply(sched).ok()
                    }).find_any(|new_sched| {

                        let found_receiver_mutex = found_receiver_mutex.lock().unwrap();
                        let mut schedules_mutex = schedules_mutex.lock().unwrap();

                        schedules_mutex.push(new_sched.clone());

                        // if there is a recursion_width truncate schedules to the best width many
                        if let Some(width) = self.recursion_width {
                            schedules_mutex.sort();
                            schedules_mutex.dedup_by(|s1,s2| s1.cmp_objective_values(s2).is_eq()); //remove dublicates
                            let width = width.min(schedules_mutex.len());
                            schedules_mutex.truncate(width);
                        }

                        let found = found_receiver_mutex.try_recv();
                        new_sched.objective_value() < objective_to_beat || found.is_ok()
                    });


                    match result {

                        Some(sched) => {
                            if sched.objective_value() < objective_to_beat {
                                succ_sender.send(sched).unwrap();
                            }
                            // if there is a Some result but the objective is not better, that means
                            // another thread was successful first. So there is nothing
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


            while let Ok(new_sched) = success_receiver.recv() {
                for s in found_senders.iter() {
                    s.send(true).ok();
                }
                if result.is_none() || new_sched.objective_value() < result.as_ref().unwrap().objective_value() {
                    result = Some(new_sched);
                }
            }
            if result.is_none() {
                for v in failure_receiver.into_iter() {
                    schedule_collection.push(v);
                }
            }
        });


        if result.is_none() {
            let number_of_schedules: usize = schedule_collection.iter().map(|v| v.len()).sum();
            println!("No improvement found after {} swaps.", number_of_schedules);

            if remaining_recursion > 0 {
                let mut schedules_for_recursion: Vec<Schedule> = schedule_collection.into_iter().flatten().collect();

                schedules_for_recursion.sort();
                // schedules_for_recursion.dedup();
                schedules_for_recursion.dedup_by(|s1,s2| s1.cmp_objective_values(s2).is_eq()); //remove dublicates




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

