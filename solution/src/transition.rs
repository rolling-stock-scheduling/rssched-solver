use std::collections::HashSet;

use im::HashMap;
use model::base_types::{MaintenanceCounter, VehicleIdx};

use crate::tour::Tour;

#[derive(Clone)]
pub struct Transition {
    cycles: Vec<TransitionCycle>,
    total_maintenance_violation: MaintenanceCounter,

    cycle_lookup: HashMap<VehicleIdx, usize>,
}

impl Transition {
    pub fn new_fast(vehicles: &[VehicleIdx], tours: &HashMap<VehicleIdx, Tour>) -> Transition {
        Transition::one_cluster_per_maintenance(vehicles, tours)
    }

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

        // sort vehicles by maintenance counter in descending order
        sorted_unassigned_vehicles
            .sort_by_key(|&vehicle| -tours.get(&vehicle).unwrap().maintenance_counter());
        // sort clusters by maintenance counter in decending order
        sorted_clusters.sort_by_key(|&(_, maintenance_counter)| -maintenance_counter);

        for vehicle in sorted_unassigned_vehicles {
            let maintenance_counter_of_tour = tours.get(&vehicle).unwrap().maintenance_counter();

            // find the cluster with the biggest maintenance counter that can accommodate the vehicle
            let best_cluster_opt = sorted_clusters.iter_mut().find(|(_, maintenance_counter)| {
                *maintenance_counter + maintenance_counter_of_tour <= 0
            });
            match best_cluster_opt {
                Some((best_cluster, maintenance_counter)) => {
                    best_cluster.push(vehicle);
                    *maintenance_counter += maintenance_counter_of_tour;
                }
                None => {
                    // if no cluster can accommodate the vehicle, put vehicle into the cluster with
                    // the smallest maintenance counter
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
        let cycles: Vec<_> = sorted_clusters
            .into_iter()
            .map(|(vehicles, maintenance_counter)| {
                total_maintenance_violation += maintenance_counter.max(0);
                TransitionCycle::new(vehicles, maintenance_counter)
            })
            .collect();

        let cycle_lookup = cycles
            .iter()
            .enumerate()
            .flat_map(|(idx, cycle)| cycle.cycle.iter().map(move |&vehicle| (vehicle, idx)))
            .collect();

        Transition {
            cycles,
            total_maintenance_violation,
            cycle_lookup,
        }
    }

    pub fn update_vehicle(
        &self,
        vehicle: VehicleIdx,
        old_maintenace_counter_of_tour: MaintenanceCounter,
        new_maintenance_counter_of_tour: MaintenanceCounter,
    ) -> Transition {
        let mut cycles = self.cycles.clone();
        let mut total_maintenance_violation = self.total_maintenance_violation;

        let cycle_idx = self.cycle_lookup.get(&vehicle).unwrap();

        let old_cycle = &mut cycles.get(*cycle_idx).unwrap();
        let new_maintenance_counter = old_cycle.maintenance_counter
            + new_maintenance_counter_of_tour
            - old_maintenace_counter_of_tour;
        let new_cycle = TransitionCycle::new(old_cycle.cycle.clone(), new_maintenance_counter);

        total_maintenance_violation +=
            new_maintenance_counter.max(0) - old_cycle.maintenance_counter.max(0);

        cycles[*cycle_idx] = new_cycle;

        Transition {
            cycles,
            total_maintenance_violation,
            cycle_lookup: self.cycle_lookup.clone(),
        }
    }

    pub fn add_vehicle_to_own_cycle(
        &self,
        vehicle: VehicleIdx,
        maintenance_counter_of_tour: MaintenanceCounter,
    ) -> Transition {
        let mut cycles = self.cycles.clone();
        let mut total_maintenance_violation = self.total_maintenance_violation;
        let mut cycle_lookup = self.cycle_lookup.clone();

        let new_cycle = TransitionCycle::new(vec![vehicle], maintenance_counter_of_tour);
        total_maintenance_violation += maintenance_counter_of_tour.max(0);
        cycles.push(new_cycle);
        cycle_lookup.insert(vehicle, cycles.len() - 1);

        Transition {
            cycles,
            total_maintenance_violation,
            cycle_lookup,
        }
    }

    pub fn remove_vehicle(
        &self,
        vehicle: VehicleIdx,
        maintenance_counter_of_old_tour: MaintenanceCounter,
    ) -> Transition {
        let mut cycles = self.cycles.clone();
        let mut total_maintenance_violation = self.total_maintenance_violation;
        let mut cycle_lookup = self.cycle_lookup.clone();

        let cycle_idx = cycle_lookup.get(&vehicle).unwrap();
        let old_cycle = cycles.get(*cycle_idx).unwrap();
        let new_cycle_vec = old_cycle
            .cycle
            .iter()
            .filter(|&&v| v != vehicle)
            .cloned()
            .collect();
        let new_maintenance_counter =
            old_cycle.maintenance_counter - maintenance_counter_of_old_tour;
        let new_cycle = TransitionCycle::new(new_cycle_vec, new_maintenance_counter);

        total_maintenance_violation +=
            new_maintenance_counter.max(0) - old_cycle.maintenance_counter.max(0);

        cycles[*cycle_idx] = new_cycle;

        cycle_lookup.remove(&vehicle);

        Transition {
            cycles,
            total_maintenance_violation,
            cycle_lookup,
        }
    }

    pub fn maintenance_violation(&self) -> MaintenanceCounter {
        self.total_maintenance_violation
    }

    pub fn print(&self) {
        println!("Transition:");
        for transition_cycle in self.cycles.iter() {
            println!("{}", transition_cycle);
        }
        println!(
            "Total maintenance violation: {}",
            self.total_maintenance_violation
        );
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
            let computed_maintenance_counter: MaintenanceCounter = transition_cycle
                .cycle
                .iter()
                .map(|&vehicle_id| tours.get(&vehicle_id).unwrap().maintenance_counter())
                .sum();
            assert_eq!(
                computed_maintenance_counter,
                transition_cycle.maintenance_counter
            );
            computed_total_maintenance_violation += computed_maintenance_counter.max(0);
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
    maintenance_counter: MaintenanceCounter,
}

impl TransitionCycle {
    pub fn new(cycle: Vec<VehicleIdx>, maintenance_counter: MaintenanceCounter) -> TransitionCycle {
        TransitionCycle {
            cycle,
            maintenance_counter,
        }
    }
}

impl std::fmt::Display for TransitionCycle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Cycle: ({}), Maintenance violation: {}",
            self.cycle
                .iter()
                .map(|&idx| format!("{}", idx.idx()))
                .collect::<Vec<String>>()
                .join(", "),
            self.maintenance_counter.max(0)
        )
    }
}
