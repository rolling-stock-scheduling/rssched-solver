use std::sync::Arc;

use heuristic_framework::local_search::Neighborhood;
use im::HashMap;
use itertools::Itertools;
use model::{base_types::VehicleIdx, network::Network};
use solution::{tour::Tour, transition::Transition};

pub struct TransitionNeighborhood {
    tours: HashMap<VehicleIdx, Tour>,
    network: Arc<Network>,
}

impl TransitionNeighborhood {
    pub fn new(tours: HashMap<VehicleIdx, Tour>, network: Arc<Network>) -> TransitionNeighborhood {
        TransitionNeighborhood { tours, network }
    }
}

impl Neighborhood<Transition> for TransitionNeighborhood {
    fn neighbors_of<'a>(
        &'a self,
        transition: &'a Transition,
    ) -> Box<dyn Iterator<Item = Transition> + Send + Sync + 'a> {
        Box::new(
            transition
                .cycles_iter()
                .enumerate()
                .combinations(2)
                .flat_map(move |combination| {
                    let (first_cycle_idx, first_cycle) = combination[0];
                    let (second_cycle_idx, second_cycle) = combination[1];
                    first_cycle
                        .iter()
                        .map(move |v| Some(v))
                        .chain(std::iter::once(None)) // add none for no vehicle
                        .flat_map(move |first_vehicle_opt| {
                            second_cycle
                                .iter()
                                .map(move |v| Some(v))
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
                            (Some(first_vehicle), Some(second_vehicle)) => transition
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
                            (Some(first_vehicle), None) => transition.move_vehicle(
                                first_vehicle,
                                second_cycle_idx,
                                &self.tours,
                                &self.network,
                            ),
                            (None, Some(second_vehicle)) => transition.move_vehicle(
                                second_vehicle,
                                first_cycle_idx,
                                &self.tours,
                                &self.network,
                            ),
                            (None, None) => transition.clone(),
                        }
                    },
                ),
        )
    }
}
