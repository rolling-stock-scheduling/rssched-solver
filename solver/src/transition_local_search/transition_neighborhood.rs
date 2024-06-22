use std::sync::Arc;

use heuristic_framework::local_search::Neighborhood;
use im::HashMap;
use itertools::Itertools;
use model::{base_types::VehicleIdx, network::Network};
use solution::tour::Tour;

use super::TransitionWithInfo;

pub struct TransitionNeighborhood {
    tours: HashMap<VehicleIdx, Tour>,
    network: Arc<Network>,
}

impl TransitionNeighborhood {
    pub fn new(tours: HashMap<VehicleIdx, Tour>, network: Arc<Network>) -> TransitionNeighborhood {
        TransitionNeighborhood { tours, network }
    }
}

impl Neighborhood<TransitionWithInfo> for TransitionNeighborhood {
    fn neighbors_of<'a>(
        &'a self,
        transition_with_info: &'a TransitionWithInfo,
    ) -> Box<dyn Iterator<Item = TransitionWithInfo> + Send + Sync + 'a> {
        Box::new(
            transition_with_info
                .get_transition()
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
                })
                .map(
                    move |(
                        first_cycle_idx,
                        first_vehicle_opt,
                        second_cycle_idx,
                        second_vehicle_opt,
                    )| {
                        match (first_vehicle_opt, second_vehicle_opt) {
                            (Some(first_vehicle), Some(second_vehicle)) => TransitionWithInfo::new(
                                transition_with_info
                                    .get_transition()
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
                                    first_vehicle,
                                    first_cycle_idx,
                                    second_vehicle,
                                    second_cycle_idx
                                ),
                            ),
                            (Some(first_vehicle), None) => TransitionWithInfo::new(
                                transition_with_info.get_transition().move_vehicle(
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
                            (None, Some(second_vehicle)) => TransitionWithInfo::new(
                                transition_with_info.get_transition().move_vehicle(
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
                            (None, None) => TransitionWithInfo::new(
                                transition_with_info.get_transition().clone(),
                                "No move".to_string(),
                            ),
                        }
                    },
                ),
        )
    }
}
