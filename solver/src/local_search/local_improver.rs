use crate::Solution;

use super::swap_factory::SwapFactory;

use model::base_types::VehicleId;
use objective_framework::{Objective, ObjectiveValue};
use rayon::iter::ParallelBridge;
use rayon::prelude::*;
use solution::Schedule;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::sync::Mutex;

/// Computes for a given schedule the best new schedule that has better objective function.
/// Returns None if there is no better schedule in the neighborhood.
pub trait LocalImprover {
    fn improve(&mut self, solution: &Solution) -> Option<Solution>;
}

///////////////////////////////////////////////////////////
////////////////////// Minimizer //////////////////////////
///////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Minimizer<T: SwapFactory> {
    swap_factory: T,
    objective: Arc<Objective<Schedule>>,
}

impl<T: SwapFactory> Minimizer<T> {
    pub fn new(swap_factory: T, objective: Arc<Objective<Schedule>>) -> Minimizer<T> {
        Minimizer {
            swap_factory,
            objective,
        }
    }
}

impl<T: SwapFactory> LocalImprover for Minimizer<T> {
    fn improve(&mut self, solution: &Solution) -> Option<Solution> {
        let schedule = solution.solution();

        let swap_iterator = self
            .swap_factory
            .create_swap_iterator(schedule, None)
            .par_bridge();
        let best_solution_opt = swap_iterator
            .filter_map(|swap| swap.apply(schedule).ok())
            .map(|sched| self.objective.evaluate(sched))
            .min_by(|s1, s2| {
                s1.objective_value()
                    .partial_cmp(s2.objective_value())
                    .unwrap()
            });
        match best_solution_opt {
            Some(best_solution) => {
                if best_solution.objective_value() < solution.objective_value() {
                    Some(best_solution)
                } else {
                    None // no improvement found
                }
            }
            None => {
                println!("\x1b[31mWARNING: NO SWAP POSSIBLE.\x1b[0m");
                None
            }
        }
    }
}

///////////////////////////////////////////////////////////
///////////////// TakeFirstRecursion //////////////////////
///////////////////////////////////////////////////////////

/// Create the swaps for each given schedule and took them into a long sequence. Find the first
/// improving schedule in this sequence.
/// As there is no parallelization this improver is fully deterministic.
#[derive(Clone)]
pub struct TakeFirstRecursion<T: SwapFactory> {
    swap_factory: T,
    recursion_depth: u8,
    recursion_width: Option<usize>, // number of schedule that are considered for recursion (the one with best value are taken)
    objective: Arc<Objective<Schedule>>,
    start_provider: Option<VehicleId>, // stores as start provider the provider of the last improving swap
}

impl<T: SwapFactory> LocalImprover for TakeFirstRecursion<T> {
    fn improve(&mut self, solution: &Solution) -> Option<Solution> {
        let old_objective_value = solution.objective_value();
        let result = self.improve_recursion(
            vec![solution.clone()],
            old_objective_value,
            self.recursion_depth,
        );

        result.map(|(sol, provider)| {
            self.start_provider = Some(provider);
            sol
        })
    }
}

impl<T: SwapFactory> TakeFirstRecursion<T> {
    pub fn new(
        swap_factory: T,
        recursion_depth: u8,
        recursion_width: Option<usize>,
        objective: Arc<Objective<Schedule>>,
    ) -> TakeFirstRecursion<T> {
        TakeFirstRecursion {
            swap_factory,
            recursion_depth,
            recursion_width,
            objective,
            start_provider: None,
        }
    }

    /// Returns the first improving schedule in the given sequence of schedules.
    /// and the provider of the last swap that lead to the improvement.
    /// If no improvement is found, None is returned.
    fn improve_recursion(
        &self,
        solution: Vec<Solution>,
        objective_to_beat: &ObjectiveValue,
        remaining_recursion: u8,
    ) -> Option<(Solution, VehicleId)> {
        let swap_iterator = solution.iter().flat_map(|sol| {
            self.swap_factory
                .create_swap_iterator(sol.solution(), self.start_provider)
                .map(move |swap| (swap, sol))
        });

        let mut counter = 0;
        let mut solutions_for_recursion: Vec<Solution> = Vec::new();

        let result = swap_iterator
            .filter_map(|(swap, old_sol)| {
                // match swap.apply(old_sol.solution()) {
                // Ok(_) => println!("OK: {}", swap),
                // Err(err) => println!("ERR for {}: {}", swap, err),
                // };
                counter += 1;
                swap.apply(old_sol.solution())
                    .ok()
                    .map(|new_schedule| (self.objective.evaluate(new_schedule), swap.provider()))
            })
            .find(|(new_sol, _)| {
                if remaining_recursion > 0 {
                    solutions_for_recursion.push(new_sol.clone());
                    if let Some(width) = self.recursion_width {
                        solutions_for_recursion.sort();
                        solutions_for_recursion.dedup();
                        // schedules_for_recursion.dedup_by(|s1,s2| s1.cmp_objective_values(s2).is_eq()); //remove dublicates
                        let width = width.min(solutions_for_recursion.len());
                        solutions_for_recursion.truncate(width);
                    }
                }
                new_sol.objective_value() < objective_to_beat
            });

        if result.is_none() {
            println!("No improvement found after {} swaps.", counter);

            if remaining_recursion > 0 {
                println!(
                    "Going into recursion. Remaining depth: {}. Schedule-count: {}",
                    remaining_recursion,
                    solutions_for_recursion.len()
                );

                self.improve_recursion(
                    solutions_for_recursion,
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

/*

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
pub(crate) struct TakeFirstParallelRecursion<F: SwapFactory> {
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
}*/

///////////////////////////////////////////////////////////
/////////////// TakeAnyParallelRecursion //////////////////
///////////////////////////////////////////////////////////

/// This improver uses parallel computation at two steps. In the recursion when multiple schedules
/// are given, each schedule get its own thread. Within each thread the swap-iterator is tranformed
/// to a ParallelIterator (messes up the ordering) and search for ANY improving schedule in
/// parallel.
/// As soon as an improving schedule is found a terminus-signal is broadcast to all other schedules.
/// If no improving schedule is found the width-many schedules of each thread are take to recursion
/// (dublicates are removed)
/// Due to the parallel computation and find_any() this improver is the fastest but not
/// deterministic.
#[derive(Clone)]
pub struct TakeAnyParallelRecursion<T: SwapFactory> {
    swap_factory: T,
    recursion_depth: u8,
    recursion_width: Option<usize>, // number of schedule that are considered per schedule for the next recursion (the one with best objectivevalue are taken for each schedule, dublicates are removed)
    objective: Arc<Objective<Schedule>>,
    start_provider: Option<VehicleId>, // stores as start provider the provider of the last improving swap
}

impl<T: SwapFactory> LocalImprover for TakeAnyParallelRecursion<T> {
    fn improve(&mut self, solution: &Solution) -> Option<Solution> {
        let old_objective = solution.objective_value();
        let start_provider = self
            .start_provider
            .unwrap_or_else(|| solution.solution().vehicles_iter().next().unwrap());
        let result = self.improve_recursion(
            vec![(solution.clone(), start_provider)],
            old_objective,
            self.recursion_depth,
        );

        result.map(|(sol, provider)| {
            self.start_provider = Some(provider);
            sol
        })
    }
}

impl<T: SwapFactory> TakeAnyParallelRecursion<T> {
    pub fn new(
        swap_factory: T,
        recursion_depth: u8,
        recursion_width: Option<usize>,
        objective: Arc<Objective<Schedule>>,
    ) -> TakeAnyParallelRecursion<T> {
        TakeAnyParallelRecursion {
            swap_factory,
            recursion_depth,
            recursion_width,
            objective,
            start_provider: None,
        }
    }

    fn improve_recursion(
        &self,
        solutions: Vec<(Solution, VehicleId)>,
        objective_to_beat: &ObjectiveValue,
        remaining_recursion: u8,
    ) -> Option<(Solution, VehicleId)> {
        // println!(
        // "Recursion: remaining depth: {}. schedule-count: {}",
        // remaining_recursion,
        // solutions.len()
        // );
        let mut solution_collection: Vec<Vec<(Solution, VehicleId)>> = Vec::new();
        let mut result: Option<(Solution, VehicleId)> = None;
        rayon::scope(|s| {
            let mut found_senders = Vec::new();
            let (success_sender, success_receiver) = channel();
            let (failure_sender, failure_receiver) = channel();

            for sol in solutions.iter() {
                let sched = sol.0.solution();
                let (found_sender, found_receiver) = channel();
                found_senders.push(found_sender);

                let succ_sender = success_sender.clone();
                let fail_sender = failure_sender.clone();
                s.spawn(move |_| {
                    let found_receiver_mutex = Arc::new(Mutex::new(found_receiver));

                    let mut new_solutions: Vec<(Solution, VehicleId)> = Vec::new();
                    let new_solutions_mutex: Arc<Mutex<&mut Vec<(Solution, VehicleId)>>> =
                        Arc::new(Mutex::new(&mut new_solutions));

                    let result = self
                        .swap_factory
                        .create_swap_iterator(sol.0.solution(), Some(sol.1))
                        .par_bridge()
                        .filter_map(|swap| {
                            swap.apply(sched)
                                .map(move |sched| (sched, swap.provider()))
                                .ok()
                        })
                        .map(|(new_sched, prov)| (self.objective.evaluate(new_sched), prov))
                        .find_any(|(new_sol, prov)| {
                            if remaining_recursion > 0 {
                                let mut schedules_mutex = new_solutions_mutex.lock().unwrap();

                                schedules_mutex.push((new_sol.clone(), *prov));

                                // if there is a recursion_width truncate schedules to the best width many
                                if let Some(width) = self.recursion_width {
                                    schedules_mutex.sort();
                                    // schedules_mutex.dedup(); //remove dublicates
                                    schedules_mutex.dedup_by(|s1, s2| {
                                        s1.0.objective_value().cmp(s2.0.objective_value()).is_eq()
                                    }); //remove dublicates according to objective_value
                                    let width = width.min(schedules_mutex.len());
                                    schedules_mutex.truncate(width);
                                }
                            }

                            let found_receiver_mutex = found_receiver_mutex.lock().unwrap();
                            let found = found_receiver_mutex.try_recv();
                            new_sol.objective_value().cmp(objective_to_beat).is_lt()
                                || found.is_ok()
                        });

                    match result {
                        Some((sol, prov)) => {
                            if sol.objective_value() < objective_to_beat {
                                succ_sender.send((sol, prov)).unwrap();
                            }
                            // if there is a Some result but the objective is not better, that means
                            // another thread was successful first. So there is nothing
                            // left to do for this thread.
                        }
                        None => {
                            fail_sender.send(new_solutions).unwrap();
                        }
                    }
                });
            }

            drop(success_sender);
            drop(failure_sender);

            while let Ok(new_sol_pair) = success_receiver.recv() {
                for s in found_senders.iter() {
                    s.send(true).ok();
                }
                if result.is_none()
                    || new_sol_pair.0.objective_value()
                        < result.as_ref().unwrap().0.objective_value()
                {
                    result = Some(new_sol_pair);
                }
            }
            if result.is_none() {
                for v in failure_receiver.into_iter() {
                    solution_collection.push(v);
                }
            }
        });

        if result.is_none() {
            // println!("No improvement found.");

            if remaining_recursion > 0 {
                let mut schedules_for_recursion: Vec<(Solution, VehicleId)> =
                    solution_collection.into_iter().flatten().collect();

                schedules_for_recursion.sort();
                // schedules_for_recursion.dedup(); //remove dublicates
                schedules_for_recursion.dedup_by(|s1, s2| s1.cmp(&s2).is_eq()); //remove dublicates according to objective_value

                self.improve_recursion(
                    schedules_for_recursion,
                    objective_to_beat,
                    remaining_recursion - 1,
                )
            } else {
                // println!("No recursion-depth left.");
                None
            }
        } else {
            // println!("Improvement found.");
            result
        }
    }
}
