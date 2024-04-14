use std::collections::HashSet;

use im::HashMap;
use model::base_types::{MaintenanceCounter, VehicleIdx};

use crate::tour::Tour;

#[derive(Clone)]
pub struct Transition {
    cycles: Vec<TransitionCycle>,
    total_maintenance_violation: MaintenanceCounter,
}

impl Transition {
    pub fn new_fast(vehicles: &[VehicleIdx], tours: &HashMap<VehicleIdx, Tour>) -> Transition {
        Transition::one_cluster_per_maintenance(vehicles, tours)
    }

    /* pub fn one_cylce_per_vehicle(
        vehicles: &[VehicleId],
        tours: &HashMap<VehicleId, Tour>,
    ) -> Transition {
        let mut total_maintenance_violation = 0;
        let cycles = vehicles
            .iter()
            .map(|&vehicle_id| {
                let tour = tours.get(&vehicle_id).unwrap();
                let maintenance_violation = tour.maintenance_counter().max(0);
                total_maintenance_violation += maintenance_violation;
                TransitionCycle::new(vec![vehicle_id], maintenance_violation)
            })
            .collect();

        Transition {
            cycles,
            total_maintenance_violation,
        }
    } */

    /// Assigns each vehicle greedily to a cluster with the goal of minimizing the total maintenance violation.
    /// It is assumed that all vehicles are of the same type.
    /// It is assumed that each vehicle has a tour.
    /// tours might contain tours of vehicle of other types.
    fn one_cluster_per_maintenance(
        vehicles: &[VehicleIdx],
        tours: &HashMap<VehicleIdx, Tour>,
    ) -> Transition {
        let mut sorted_clusters: Vec<(Vec<VehicleIdx>, MaintenanceCounter)> = Vec::new(); // TODO Use BTreeMap
        let mut sorted_unassigned_vehicles: Vec<VehicleIdx> = Vec::new(); // all none maintenance
                                                                         // vehicles sorted by
                                                                         // maintenance counter in descending order

        for vehicle_id in vehicles.iter() {
            let tour = tours.get(vehicle_id).unwrap();
            if tour.maintenance_counter() < 0 {
                sorted_clusters.push((vec![*vehicle_id], tour.maintenance_counter()));
            } else {
                sorted_unassigned_vehicles.push(*vehicle_id);
            }
        }

        sorted_unassigned_vehicles
            .sort_by_key(|&vehicle| -tours.get(&vehicle).unwrap().maintenance_counter());
        sorted_clusters.sort_by_key(|&(_, maintenance_counter)| maintenance_counter);

        for vehicle in sorted_unassigned_vehicles {
            let maintenance_counter_of_tour = tours.get(&vehicle).unwrap().maintenance_counter();

            let best_cluster_opt = sorted_clusters.iter_mut().find(|(_, maintenance_counter)| {
                *maintenance_counter + maintenance_counter_of_tour <= 0
            });
            match best_cluster_opt {
                Some((best_cluster, maintenance_counter)) => {
                    best_cluster.push(vehicle);
                    *maintenance_counter += maintenance_counter_of_tour;
                }
                None => {
                    let last_cluster_opt = sorted_clusters.last_mut();
                    match last_cluster_opt {
                        Some((last_cluster, maintenance_counter)) => {
                            last_cluster.push(vehicle);
                            *maintenance_counter += maintenance_counter_of_tour;
                        }
                        None => {
                            sorted_clusters.push((vec![vehicle], maintenance_counter_of_tour));
                        }
                    }
                }
            }
            sorted_clusters.sort_by_key(|&(_, maintenance_counter)| maintenance_counter);
        }

        let mut total_maintenance_violation = 0;
        let cycles = sorted_clusters
            .into_iter()
            .map(|(vehicles, maintenance_counter)| {
                let maintenance_violation = maintenance_counter.max(0);
                total_maintenance_violation += maintenance_violation;
                TransitionCycle::new(vehicles, maintenance_violation)
            })
            .collect();
        Transition {
            cycles,
            total_maintenance_violation,
        }
    }

    pub fn maintenance_violation(&self) -> MaintenanceCounter {
        self.total_maintenance_violation
    }

    /// Verifies that the transition is consistent with the tours.
    /// Note that the tours must be the tours of this vehicle type group.
    pub fn verify_consistency(&self, tours: &HashMap<VehicleIdx, Tour>) {
        // each vehicle is present in exactly one cycle
        let cycles: Vec<VehicleIdx> = self
            .cycles
            .iter()
            .flat_map(|transition_cycle| transition_cycle.cycle.iter().cloned())
            .collect();
        assert_eq!(cycles.len(), tours.len());
        let vehicles_from_tours: HashSet<VehicleIdx> = tours.keys().cloned().collect();
        let vehicles_from_cycles: HashSet<VehicleIdx> = cycles.iter().cloned().collect();
        assert_eq!(vehicles_from_tours, vehicles_from_cycles);

        // verify maintenance violations
        let mut computed_total_maintenance_violation = 0;
        for transition_cycle in self.cycles.iter() {
            let maintenance_counter: MaintenanceCounter = transition_cycle
                .cycle
                .iter()
                .map(|&vehicle_id| tours.get(&vehicle_id).unwrap().maintenance_counter())
                .sum();
            let computed_maintenance_violation = maintenance_counter.max(0);
            assert_eq!(
                computed_maintenance_violation,
                transition_cycle.maintenance_violation
            );
            computed_total_maintenance_violation += computed_maintenance_violation;
        }
        assert_eq!(
            computed_total_maintenance_violation,
            self.total_maintenance_violation
        );
    }
}

#[derive(Debug, Clone)]
pub struct TransitionCycle {
    cycle: Vec<VehicleIdx>,
    maintenance_violation: MaintenanceCounter,
}

impl TransitionCycle {
    pub fn new(
        cycle: Vec<VehicleIdx>,
        maintenance_violation: MaintenanceCounter,
    ) -> TransitionCycle {
        TransitionCycle {
            cycle,
            maintenance_violation,
        }
    }
}
