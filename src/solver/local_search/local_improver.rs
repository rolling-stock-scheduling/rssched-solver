use crate::base_types::Cost;
use crate::schedule::Schedule;

use super::swap_factory::SwapFactory;
use crate::schedule::objective::ObjectiveValue;

use rayon::iter::ParallelBridge;
use rayon::prelude::*;
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
    swap_factory: F,
}

impl<F: SwapFactory> Minimizer<F> {
    pub(crate) fn new(swap_factory: F) -> Minimizer<F> {
        Minimizer { swap_factory }
    }
}

impl<F: SwapFactory> LocalImprover for Minimizer<F> {
    fn improve(&self, schedule: &Schedule) -> Option<Schedule> {
        let swap_iterator = self
            .swap_factory
            .create_swap_iterator(schedule)
            .par_bridge();
        let (best_objective_value, best_schedule) = swap_iterator
            .filter_map(|swap| swap.apply(schedule).ok())
            .map(|sched| (sched.objective_value(), sched))
            .min_by(|(o1, _), (o2, _)| o1.partial_cmp(o2).unwrap())
            .unwrap();

        if best_objective_value < schedule.objective_value() {
            Some(best_schedule)
        } else {
            None
        }
    }
}

///////////////////////////////////////////////////////////
///////////////// TakeFirstRecursion //////////////////////
///////////////////////////////////////////////////////////

/// Create the swaps for each given schedule and took them into a long sequence. Find the first
/// improving schedule in this sequence.
/// As there is no parallelization this improver is fully deterministic.
pub(crate) struct TakeFirstRecursion<F: SwapFactory> {
    swap_factory: F,
    recursion_depth: u8,
    recursion_width: Option<usize>, // number of schedule that are considered for recursion (the one with best value are taken)
    soft_objective_threshold: Cost,
}

impl<F: SwapFactory> LocalImprover for TakeFirstRecursion<F> {
    fn improve(&self, schedule: &Schedule) -> Option<Schedule> {
        let old_objective = schedule.objective_value();
        self.improve_recursion(vec![schedule.clone()], old_objective, self.recursion_depth)
    }
}

impl<F: SwapFactory> TakeFirstRecursion<F> {
    pub(crate) fn new(
        swap_factory: F,
        recursion_depth: u8,
        recursion_width: Option<usize>,
        soft_objective_threshold: Cost,
    ) -> TakeFirstRecursion<F> {
        TakeFirstRecursion {
            swap_factory,
            recursion_depth,
            recursion_width,
            soft_objective_threshold,
        }
    }

    fn improve_recursion(
        &self,
        schedules: Vec<Schedule>,
        objective_to_beat: ObjectiveValue,
        remaining_recursion: u8,
    ) -> Option<Schedule> {
        let swap_iterator = schedules.iter().flat_map(|sched| {
            self.swap_factory
                .create_swap_iterator(sched)
                .map(move |swap| (swap, sched))
        });

        let mut counter = 0;
        let mut schedules_for_recursion: Vec<Schedule> = Vec::new();

        let result = swap_iterator
            .filter_map(|(swap, old_sched)| {
                counter += 1;
                swap.apply(old_sched).ok()
            })
            .find(|new_sched| {
                if remaining_recursion > 0 {
                    schedules_for_recursion.push(new_sched.clone());
                    if let Some(width) = self.recursion_width {
                        schedules_for_recursion.sort();
                        schedules_for_recursion.dedup();
                        // schedules_for_recursion.dedup_by(|s1,s2| s1.cmp_objective_values(s2).is_eq()); //remove dublicates
                        let width = width.min(schedules_for_recursion.len());
                        schedules_for_recursion.truncate(width);
                    }
                }
                new_sched
                    .objective_value()
                    .cmp_with_threshold(&objective_to_beat, self.soft_objective_threshold)
                    .is_lt()
            });

        if result.is_none() {
            println!("No improvement found after {} swaps.", counter);

            if remaining_recursion > 0 {
                println!(
                    "Going into recursion. Remaining depth: {}. Schedule-count: {}",
                    remaining_recursion,
                    schedules_for_recursion.len()
                );

                self.improve_recursion(
                    schedules_for_recursion,
                    objective_to_beat,
                    remaining_recursion - 1,
                )
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

///////////////////////////////////////////////////////////
///////////// TakeFirstParallelRecursion //////////////////
///////////////////////////////////////////////////////////

/// This improver uses multiple threads for recursions steps. One thread per provided schedule.
/// For each schedule the swaps are provided sequencially and the first improvement is taken. The
/// index is then send to the main thread, where it is broadcast to all other threads. In the end
/// the improving schedule with the lowest index are taken. (Ties are broken by schedule-ordering.)
/// Im no improving schedule is found the depth-many schedules of each thread are take to recursion
/// (dublicates are removed)
/// This improver is deterministic.
pub(crate) struct TakeFirstParallelRecursion<F: SwapFactory + Send + Sync> {
    swap_factory: F,
    recursion_depth: u8,
    recursion_width: Option<usize>, // number of schedule that are considered per schedule for the next recursion (the one with best objectivevalue are taken for each schedule, dublicates are removed)
    soft_objective_threshold: Cost, // improvement must be better than this threshold
}

impl<F: SwapFactory + Send + Sync> LocalImprover for TakeFirstParallelRecursion<F> {
    fn improve(&self, schedule: &Schedule) -> Option<Schedule> {
        let old_objective = schedule.objective_value();
        self.improve_recursion(vec![schedule.clone()], old_objective, self.recursion_depth)
    }
}

impl<F: SwapFactory + Send + Sync> TakeFirstParallelRecursion<F> {
    pub(crate) fn new(
        swap_factory: F,
        recursion_depth: u8,
        recursion_width: Option<usize>,
        soft_objective_threshold: Cost,
    ) -> TakeFirstParallelRecursion<F> {
        TakeFirstParallelRecursion {
            swap_factory,
            recursion_depth,
            recursion_width,
            soft_objective_threshold,
        }
    }

    fn improve_recursion(
        &self,
        schedules: Vec<Schedule>,
        objective_to_beat: ObjectiveValue,
        remaining_recursion: u8,
    ) -> Option<Schedule> {
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
                    let result = self
                        .swap_factory
                        .create_swap_iterator(sched)
                        .enumerate()
                        .filter_map(|(i, swap)| {
                            swap.apply(sched).ok().map(move |new_sched| (i, new_sched))
                        })
                        .find(|(i, new_sched)| {
                            if remaining_recursion > 0 {
                                schedules.push(new_sched.clone());

                                // if there is a recursion_width truncate schedules to the best width many
                                if let Some(width) = self.recursion_width {
                                    schedules.sort();
                                    // schedules.dedup(); //remove dublicates
                                    schedules
                                        .dedup_by(|s1, s2| s1.cmp_objective_values(s2).is_eq()); //remove dublicates according to objective_value

                                    let width = width.min(schedules.len());
                                    schedules.truncate(width);
                                }
                            }

                            if let Ok(best_i) = found_receiver.try_recv() {
                                counter_limit = best_i;
                            }
                            new_sched
                                .objective_value()
                                .cmp_with_threshold(
                                    &objective_to_beat,
                                    self.soft_objective_threshold,
                                )
                                .is_lt()
                                || *i > counter_limit
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
                if result.is_none()
                    || i < best_i
                    || (i == best_i
                        && new_sched.objective_value() < result.as_ref().unwrap().objective_value())
                {
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
        });

        if result.is_none() {
            println!("No improvement found.");

            if remaining_recursion > 0 {
                let mut schedules_for_recursion: Vec<Schedule> =
                    schedule_collection.into_iter().flatten().collect();

                schedules_for_recursion.sort();
                // schedules_for_recursion.dedup(); //remove dublicates
                schedules_for_recursion.dedup_by(|s1, s2| s1.cmp_objective_values(s2).is_eq()); //remove dublicates according to objective_value

                println!(
                    "Going into recursion. Remaining depth: {}. Schedule-count: {}",
                    remaining_recursion,
                    schedules_for_recursion.len()
                );

                self.improve_recursion(
                    schedules_for_recursion,
                    objective_to_beat,
                    remaining_recursion - 1,
                )
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

///////////////////////////////////////////////////////////
/////////////// TakeAnyParallelRecursion //////////////////
///////////////////////////////////////////////////////////

/// This improver uses parallel computation at two steps. In the recursion when multiple schedules
/// are given, each schedule get its own thread. Within each thread the swap-iterator is tranformed
/// to a ParallelIterator (messes up the ordering) and search for ANY improving schedule in
/// parallel.
/// As soon as an improving schedule is found a terminus-signal is broadcast to all other schedules.
/// Im no improving schedule is found the depth-many schedules of each thread are take to recursion
/// (dublicates are removed)
/// Due to the parallel computation and find_any() this improver is the fastest but not
/// deterministic.
pub(crate) struct TakeAnyParallelRecursion<F: SwapFactory + Send + Sync> {
    swap_factory: F,
    recursion_depth: u8,
    recursion_width: Option<usize>, // number of schedule that are considered per schedule for the next recursion (the one with best objectivevalue are taken for each schedule, dublicates are removed)
    soft_objective_threshold: Cost, // improvement must be better than this threshold
}

impl<F: SwapFactory + Send + Sync> LocalImprover for TakeAnyParallelRecursion<F> {
    fn improve(&self, schedule: &Schedule) -> Option<Schedule> {
        let old_objective = schedule.objective_value();
        self.improve_recursion(vec![schedule.clone()], old_objective, self.recursion_depth)
    }
}

impl<F: SwapFactory + Send + Sync> TakeAnyParallelRecursion<F> {
    pub(crate) fn new(
        swap_factory: F,
        recursion_depth: u8,
        recursion_width: Option<usize>,
        soft_objective_threshold: Cost,
    ) -> TakeAnyParallelRecursion<F> {
        TakeAnyParallelRecursion {
            swap_factory,
            recursion_depth,
            recursion_width,
            soft_objective_threshold,
        }
    }

    fn improve_recursion(
        &self,
        schedules: Vec<Schedule>,
        objective_to_beat: ObjectiveValue,
        remaining_recursion: u8,
    ) -> Option<Schedule> {
        println!(
            "Recursion: remaining depth: {}. schedule-count: {}",
            remaining_recursion,
            schedules.len()
        );
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
                    let schedules_mutex: Arc<Mutex<&mut Vec<Schedule>>> =
                        Arc::new(Mutex::new(&mut schedules));

                    let result = self
                        .swap_factory
                        .create_swap_iterator(sched)
                        .par_bridge()
                        .filter_map(|swap| swap.apply(sched).ok())
                        .find_any(|new_sched| {
                            if remaining_recursion > 0 {
                                let mut schedules_mutex = schedules_mutex.lock().unwrap();

                                schedules_mutex.push(new_sched.clone());

                                // if there is a recursion_width truncate schedules to the best width many
                                if let Some(width) = self.recursion_width {
                                    schedules_mutex.sort();
                                    // schedules_mutex.dedup(); //remove dublicates
                                    schedules_mutex
                                        .dedup_by(|s1, s2| s1.cmp_objective_values(s2).is_eq()); //remove dublicates according to objective_value
                                    let width = width.min(schedules_mutex.len());
                                    schedules_mutex.truncate(width);
                                }
                            }

                            let found_receiver_mutex = found_receiver_mutex.lock().unwrap();
                            let found = found_receiver_mutex.try_recv();
                            new_sched
                                .objective_value()
                                .cmp_with_threshold(
                                    &objective_to_beat,
                                    self.soft_objective_threshold,
                                )
                                .is_lt()
                                || found.is_ok()
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
                if result.is_none()
                    || new_sched.objective_value() < result.as_ref().unwrap().objective_value()
                {
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
            println!("No improvement found.");

            if remaining_recursion > 0 {
                let mut schedules_for_recursion: Vec<Schedule> =
                    schedule_collection.into_iter().flatten().collect();

                schedules_for_recursion.sort();
                // schedules_for_recursion.dedup(); //remove dublicates
                schedules_for_recursion.dedup_by(|s1, s2| s1.cmp_objective_values(s2).is_eq()); //remove dublicates according to objective_value

                self.improve_recursion(
                    schedules_for_recursion,
                    objective_to_beat,
                    remaining_recursion - 1,
                )
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
