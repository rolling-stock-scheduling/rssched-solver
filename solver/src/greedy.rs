use crate::Solution;
use crate::Solver;
use model::base_types::VehicleId;
use model::config::Config;
use model::network::Network;
use model::vehicle_types::VehicleTypes;
use objective_framework::Objective;
use solution::path::Path;
use solution::Schedule;
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::sync::Arc;
use time::DateTime;

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

        // vehicles sorted by the end time of the last trip of their tour (late to early) ties are broken by
        // vehicleId.
        let mut vehicles_sorted: BTreeMap<(Reverse<DateTime>, VehicleId), VehicleId> =
            BTreeMap::new();

        for service_trip in self.network.service_nodes() {
            let start_time = self.network.node(service_trip).start_time();
            while !schedule.is_fully_covered(service_trip) {
                let candidate = vehicles_sorted
                    .range((Reverse(start_time), VehicleId::from(""))..)
                    .find(
                        |(_, &v)| match schedule.tour_of(v).unwrap().last_non_depot() {
                            Some(last) => self.network.can_reach(last, service_trip),
                            None => false, // not possible as tours always have at least one
                                           // non_depot
                        },
                    )
                    .map(|(_, &v)| v);

                match candidate {
                    Some(v) => {
                        vehicles_sorted.remove(&(
                            Reverse(
                                self.network
                                    .node(schedule.tour_of(v).unwrap().last_non_depot().unwrap())
                                    .end_time(),
                            ),
                            v,
                        ));
                        schedule = schedule
                            .add_path_to_vehicle_tour(
                                v,
                                Path::new_from_single_node(service_trip, self.network.clone()),
                            )
                            .unwrap();
                        vehicles_sorted
                            .insert((Reverse(self.network.node(service_trip).end_time()), v), v);
                    }
                    None => {
                        // no vehicle can reach the service trip, spawn a new one

                        // TODO decide on which vehicle type (biggest or best fitting)
                        // for now: take biggest vehicles (good for a small count, as it might be reused for
                        // later trips)
                        let vehicle_type = self.vehicles.iter().last().unwrap();

                        let result =
                            schedule.spawn_vehicle_for_path(vehicle_type, vec![service_trip]);

                        match result {
                            Ok((new_schedule, v)) => {
                                schedule = new_schedule;
                                vehicles_sorted.insert(
                                    (Reverse(self.network.node(service_trip).end_time()), v),
                                    v,
                                );
                            }
                            Err(_) => {
                                println!(
                                    "Greedy: not enough depots space to cover service trip {}",
                                    service_trip
                                );
                            }
                        }
                    }
                }
            }
        }

        schedule = schedule.reassign_end_depots_greedily().unwrap();

        self.objective.evaluate(schedule)
    }
}
