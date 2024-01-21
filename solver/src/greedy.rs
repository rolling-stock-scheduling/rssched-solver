use crate::Solution;
use crate::Solver;
use model::base_types::VehicleId;
use model::config::Config;
use model::network::Network;
use model::vehicle_types::VehicleTypes;
use objective_framework::Objective;
use solution::path::Path;
use solution::Schedule;
use std::sync::Arc;

pub struct Greedy {
    vehicles: Arc<VehicleTypes>,
    network: Arc<Network>,
    config: Arc<Config>,
    objective: Arc<Objective<Schedule>>,
}

impl Solver for Greedy {
    fn initialize(
        vehicles: Arc<VehicleTypes>,
        network: Arc<Network>,
        config: Arc<Config>,
        objective: Arc<Objective<Schedule>>,
    ) -> Greedy {
        Greedy {
            vehicles,
            network,
            config,
            objective,
        }
    }

    fn solve(&self) -> Solution {
        let mut schedule = Schedule::empty(
            self.vehicles.clone(),
            self.network.clone(),
            self.config.clone(),
        );

        for service_trip in self.network.service_nodes() {
            while !schedule.is_fully_covered(service_trip) {
                let vehicle_candidates: Vec<VehicleId> = schedule
                    .vehicles_iter()
                    .filter(|&v| match schedule.tour_of(v).unwrap().last_non_depot() {
                        Some(last) => self.network.can_reach(last, service_trip),
                        None => false, // vehicle goes from depot to depot (does not happen here)
                    })
                    .collect();

                // pick the vehicle which tour ends the latest
                let final_candidate = vehicle_candidates.iter().max_by_key(|&&v| {
                    let last_trip = schedule.tour_of(v).unwrap().last_non_depot().unwrap();
                    self.network.node(last_trip).end_time()
                });

                match final_candidate {
                    Some(&v) => {
                        schedule = schedule
                            .add_path_to_vehicle_tour(
                                v,
                                Path::new_from_single_node(service_trip, self.network.clone()),
                            )
                            .unwrap();
                    }
                    None => {
                        // no vehicle can reach the service trip, spawn a new one

                        // TODO decide on which vehicle type (biggest or best fitting)

                        // let vehicle_type = self
                        // .vehicles
                        // .best_for(schedule.unserved_passengers_at(service_trip));

                        // take biggest vehicles (good for a small count, as it might be reused for
                        // later trips)
                        let vehicle_type = self.vehicles.iter().last().unwrap();

                        schedule = schedule
                            .spawn_vehicle_for_path(vehicle_type, vec![service_trip])
                            .unwrap()
                            .0;
                    }
                }
            }
        }

        schedule = schedule.reassign_end_depots_greedily().unwrap();

        self.objective.evaluate(schedule)
    }
}
