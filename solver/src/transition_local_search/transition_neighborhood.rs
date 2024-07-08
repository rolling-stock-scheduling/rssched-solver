use std::sync::Arc;

use im::HashMap;
use itertools::Itertools;
use model::{base_types::VehicleIdx, network::Network};
use rapid_solve::heuristics::common::Neighborhood;
use rapid_solve::heuristics::local_search::LocalSearchSolver;
use rapid_solve::heuristics::Solver;
use solution::tour::Tour;

use crate::transition_cycle_tsp::TransitionCycleWithInfo;

use super::TransitionWithInfo;

pub struct TransitionNeighborhood {
    tours: HashMap<VehicleIdx, Tour>,
    cycle_tsp_solver: LocalSearchSolver<TransitionCycleWithInfo>,
    network: Arc<Network>,
}

impl TransitionNeighborhood {
    pub fn new(
        tours: HashMap<VehicleIdx, Tour>,
        cycle_tsp_solver: LocalSearchSolver<TransitionCycleWithInfo>,
        network: Arc<Network>,
    ) -> TransitionNeighborhood {
        TransitionNeighborhood {
            tours,
            cycle_tsp_solver,
            network,
        }
    }
}

impl Neighborhood<TransitionWithInfo> for TransitionNeighborhood {
    fn neighbors_of<'a>(
        &'a self,
        transition_with_info: &'a TransitionWithInfo,
    ) -> Box<dyn Iterator<Item = TransitionWithInfo> + Send + Sync + 'a> {
        let transition = transition_with_info.get_transition();
        // first create a iterator over cycle pairs and all vehicles of these cycles
        let indices_iter = transition
            .cycles_iter()
            .enumerate()
            .combinations(2)
            .flat_map(move |combination| {
                let (first_cycle_idx, first_cycle) = combination[0];
                let (second_cycle_idx, second_cycle) = combination[1];
                first_cycle
                    .iter()
                    .map(Some)
                    .chain(std::iter::once(None)) // add none for no vehicle
                    .flat_map(move |first_vehicle_opt| {
                        second_cycle
                            .iter()
                            .map(Some)
                            .chain(std::iter::once(None)) // add nonce for no vehicle
                            .map(move |second_vehicle_opt| {
                                (
                                    first_cycle_idx,
                                    first_vehicle_opt,
                                    second_cycle_idx,
                                    second_vehicle_opt,
                                )
                            })
                    })
            });

        // apply the exchange of vehicles and create a string for the swap
        let swap_iter =
            indices_iter.map(
                move |(
                    first_cycle_idx,
                    first_vehicle_opt,
                    second_cycle_idx,
                    second_vehicle_opt,
                )| {
                    match (first_vehicle_opt, second_vehicle_opt) {
                        (Some(first_vehicle), Some(second_vehicle)) => (
                            first_cycle_idx,
                            second_cycle_idx,
                            transition
                                .move_vehicle(
                                    first_vehicle,
                                    second_cycle_idx,
                                    &self.tours,
                                    &self.network,
                                )
                                .move_vehicle(
                                    second_vehicle,
                                    first_cycle_idx,
                                    &self.tours,
                                    &self.network,
                                ),
                            format!(
                                "Exchange vehicles: {} from cycle {} with {} from cycle {}",
                                first_vehicle, first_cycle_idx, second_vehicle, second_cycle_idx
                            ),
                        ),
                        (Some(first_vehicle), None) => (
                            first_cycle_idx,
                            second_cycle_idx,
                            transition.move_vehicle(
                                first_vehicle,
                                second_cycle_idx,
                                &self.tours,
                                &self.network,
                            ),
                            format!(
                                "Move vehicle: {} from cycle {} to cycle {}",
                                first_vehicle, first_cycle_idx, second_cycle_idx
                            ),
                        ),
                        (None, Some(second_vehicle)) => (
                            first_cycle_idx,
                            second_cycle_idx,
                            transition.move_vehicle(
                                second_vehicle,
                                first_cycle_idx,
                                &self.tours,
                                &self.network,
                            ),
                            format!(
                                "Move vehicle: {} from cycle {} to cycle {}",
                                second_vehicle, second_cycle_idx, first_cycle_idx
                            ),
                        ),
                        (None, None) => (
                            first_cycle_idx,
                            second_cycle_idx,
                            transition_with_info.get_transition().clone(),
                            "No move".to_string(),
                        ),
                    }
                },
            );

        // apply the tsp solver to the two modified cycles
        Box::new(swap_iter.map(
            |(first_cycle_idx, second_cycle_idx, mut new_transition, description)| {
                vec![first_cycle_idx, second_cycle_idx]
                    .into_iter()
                    .for_each(|cycle_idx| {
                        let new_cycle = new_transition.get_cycle(cycle_idx);
                        let start_cycle = TransitionCycleWithInfo::new(
                            new_cycle.clone(),
                            format!("Initial cycle {}", new_cycle),
                        );
                        let improved_cycle = self
                            .cycle_tsp_solver
                            .solve(start_cycle)
                            .unwrap()
                            .unwrap_cycle();
                        new_transition = new_transition.replace_cycle(cycle_idx, improved_cycle);
                    });

                TransitionWithInfo::new(new_transition, description)
            },
        ))
    }
}
