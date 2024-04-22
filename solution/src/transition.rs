use std::collections::HashSet;

use im::HashMap;
use itertools::Itertools;
use model::{
    base_types::{MaintenanceCounter, NodeIdx, VehicleIdx, MAX_DISTANCE},
    network::Network,
};

use crate::tour::Tour;

#[derive(Clone)]
pub struct Transition {
    cycles: Vec<TransitionCycle>,
    total_maintenance_violation: MaintenanceCounter,

    cycle_lookup: HashMap<VehicleIdx, usize>,
    empty_cycles: Vec<usize>, // as the indices need to stay as they are, sometimes a cycle become
                              // empty but it cannot be removed.
                              // New vehicles can use these empty cycles instead of creating a new
                              // one.
}

impl Transition {
    pub fn new_fast(
        vehicles: &[VehicleIdx],
        tours: &HashMap<VehicleIdx, Tour>,
        network: &Network,
    ) -> Transition {
        Transition::one_cluster_per_maintenance(vehicles, tours, network)
    }

    // TODO write test for this function
    /// Assigns each vehicle greedily to a cluster with the goal of minimizing the total maintenance violation.
    /// It is assumed that all vehicles are of the same type.
    /// It is assumed that each vehicle has a tour.
    /// tours might contain tours of vehicle of other types.
    fn one_cluster_per_maintenance(
        vehicles: &[VehicleIdx],
        tours: &HashMap<VehicleIdx, Tour>,
        network: &Network,
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
            let tour: &Tour = tours.get(&vehicle).unwrap();
            let maintenance_counter_of_tour = tour.maintenance_counter();

            // find the cluster with the biggest maintenance counter that can accommodate the vehicle
            let best_cluster_opt = sorted_clusters.iter_mut().find(|(_, maintenance_counter)| {
                *maintenance_counter + maintenance_counter_of_tour <= 0
            });
            match best_cluster_opt {
                Some((best_cluster, maintenance_counter)) => {
                    Transition::push_vehicle_to_end_of_cluster(
                        best_cluster,
                        maintenance_counter,
                        vehicle,
                        tours,
                        network,
                    );
                }
                None => {
                    // if no cluster can accommodate the vehicle, put vehicle into the cluster with
                    // the smallest maintenance counter
                    let last_cluster_opt = sorted_clusters.last_mut();
                    match last_cluster_opt {
                        Some((last_cluster, maintenance_counter)) => {
                            Transition::push_vehicle_to_end_of_cluster(
                                last_cluster,
                                maintenance_counter,
                                vehicle,
                                tours,
                                network,
                            );
                        }
                        None => {
                            // there is no cluster yet, create a new cluster
                            // note that the dead_head_trip from end_depot to start_depot is added
                            // later on.
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
            .map(|(vehicles, mut maintenance_counter)| {
                let last_end_depot_to_first_start_depot = network
                    .dead_head_distance_between(
                        tours
                            .get(vehicles.last().unwrap())
                            .unwrap()
                            .end_depot()
                            .unwrap(),
                        tours
                            .get(vehicles.first().unwrap())
                            .unwrap()
                            .start_depot()
                            .unwrap(),
                    )
                    .in_meter()
                    .unwrap_or(MAX_DISTANCE)
                    as MaintenanceCounter;
                maintenance_counter += last_end_depot_to_first_start_depot;

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
            empty_cycles: Vec::new(),
        }
    }

    // TODO write test for this function (with multiple vehicles being updated in a raw)
    pub fn update_vehicle(
        &self,
        vehicle: VehicleIdx,
        new_tour: &Tour,
        old_tours: &HashMap<&VehicleIdx, &Tour>,
        network: &Network,
    ) -> Transition {
        let old_tour = old_tours.get(&vehicle).unwrap();
        let mut cycles = self.cycles.clone();
        let cycle_idx = self.cycle_lookup.get(&vehicle).unwrap();
        let old_cycle = &mut cycles.get(*cycle_idx).unwrap();
        let mut total_maintenance_violation = self.total_maintenance_violation;

        let new_maintenance_counter = if old_cycle.cycle.len() == 1 {
            new_tour.maintenance_counter()
                + network
                    .dead_head_distance_between(
                        new_tour.end_depot().unwrap(),
                        new_tour.start_depot().unwrap(),
                    )
                    .in_meter()
                    .unwrap_or(MAX_DISTANCE) as MaintenanceCounter
        } else {
            let (end_depot_of_predecessor, start_depot_of_successor) =
                self.end_depot_of_predecessor_and_start_depot_of_successor(vehicle, old_tours);

            let maintenance_counter_for_removal = self
                .maintenance_counter_of_tour_plus_dead_head_trips_before_and_after(
                    old_tour,
                    end_depot_of_predecessor,
                    start_depot_of_successor,
                    network,
                );
            let maintenance_counter_for_addition = self
                .maintenance_counter_of_tour_plus_dead_head_trips_before_and_after(
                    new_tour,
                    end_depot_of_predecessor,
                    start_depot_of_successor,
                    network,
                );

            old_cycle.maintenance_counter - maintenance_counter_for_removal
                + maintenance_counter_for_addition
        };

        let new_cycle = TransitionCycle::new(old_cycle.cycle.clone(), new_maintenance_counter);

        total_maintenance_violation +=
            new_maintenance_counter.max(0) - old_cycle.maintenance_counter.max(0);

        cycles[*cycle_idx] = new_cycle;

        Transition {
            cycles,
            total_maintenance_violation,
            cycle_lookup: self.cycle_lookup.clone(),
            empty_cycles: self.empty_cycles.clone(),
        }
    }

    // TODO write test for this function
    pub fn add_vehicle_to_own_cycle(
        &self,
        vehicle: VehicleIdx,
        new_tour: &Tour,
        network: &Network,
    ) -> Transition {
        let mut cycles = self.cycles.clone();
        let mut total_maintenance_violation = self.total_maintenance_violation;
        let mut cycle_lookup = self.cycle_lookup.clone();
        let mut empty_cycles = self.empty_cycles.clone();

        let maintenance_counter_of_tour = new_tour.maintenance_counter()
            + network
                .dead_head_distance_between(
                    new_tour.end_depot().unwrap(),
                    new_tour.start_depot().unwrap(),
                )
                .in_meter()
                .unwrap_or(MAX_DISTANCE) as MaintenanceCounter;

        let new_cycle = TransitionCycle::new(vec![vehicle], maintenance_counter_of_tour);
        total_maintenance_violation += maintenance_counter_of_tour.max(0);

        if empty_cycles.is_empty() {
            cycles.push(new_cycle);
            cycle_lookup.insert(vehicle, cycles.len() - 1);
        } else {
            let empty_cycle_idx = empty_cycles.pop().unwrap();
            cycles[empty_cycle_idx] = new_cycle;
            cycle_lookup.insert(vehicle, empty_cycle_idx);
        }

        Transition {
            cycles,
            total_maintenance_violation,
            cycle_lookup,
            empty_cycles,
        }
    }

    // TODO write test for this function (with multiple vehicles being removed in a raw)
    pub fn remove_vehicle(
        &self,
        vehicle: VehicleIdx,
        old_tours: &HashMap<&VehicleIdx, &Tour>,
        network: &Network,
    ) -> Transition {
        let mut cycles = self.cycles.clone();
        let mut total_maintenance_violation = self.total_maintenance_violation;
        let mut cycle_lookup = self.cycle_lookup.clone();
        let mut empty_cycles = self.empty_cycles.clone();

        let cycle_idx = cycle_lookup.get(&vehicle).unwrap();
        let old_cycle = cycles.get(*cycle_idx).unwrap();
        let new_cycle_vec: Vec<_> = old_cycle
            .cycle
            .iter()
            .filter(|&&v| v != vehicle)
            .cloned()
            .collect();
        if new_cycle_vec.is_empty() {
            total_maintenance_violation -= old_cycle.maintenance_counter.max(0);
            cycles[*cycle_idx] = TransitionCycle::new(vec![], 0);
            empty_cycles.push(*cycle_idx);
        } else {
            let (end_depot_of_predecessor, start_depot_of_successor) =
                self.end_depot_of_predecessor_and_start_depot_of_successor(vehicle, old_tours);

            let maintenance_counter_for_removal = self
                .maintenance_counter_of_tour_plus_dead_head_trips_before_and_after(
                    old_tours.get(&vehicle).unwrap(),
                    end_depot_of_predecessor,
                    start_depot_of_successor,
                    network,
                );

            let maintenance_counter_for_addition = network
                .dead_head_distance_between(end_depot_of_predecessor, start_depot_of_successor)
                .in_meter()
                .unwrap_or(MAX_DISTANCE)
                as MaintenanceCounter;

            let new_maintenance_counter = old_cycle.maintenance_counter
                - maintenance_counter_for_removal
                + maintenance_counter_for_addition;
            let new_cycle = TransitionCycle::new(new_cycle_vec, new_maintenance_counter);

            total_maintenance_violation +=
                new_maintenance_counter.max(0) - old_cycle.maintenance_counter.max(0);

            cycles[*cycle_idx] = new_cycle;
        }

        cycle_lookup.remove(&vehicle);

        Transition {
            cycles,
            total_maintenance_violation,
            cycle_lookup,
            empty_cycles,
        }
    }

    pub fn maintenance_violation(&self) -> MaintenanceCounter {
        self.total_maintenance_violation
    }

    pub fn print(&self) {
        println!("Transition:");
        for transition_cycle in self.cycles.iter() {
            if !transition_cycle.cycle.is_empty() {
                println!("{}", transition_cycle);
            }
        }
        println!(
            "Total maintenance violation: {}",
            self.total_maintenance_violation
        );
    }

    /// Verifies that the transition is consistent with the tours.
    /// Note that the tours must be the tours of this vehicle type group.
    pub fn verify_consistency(&self, tours: &HashMap<VehicleIdx, Tour>, network: &Network) {
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
            let mut computed_maintenance_counter: MaintenanceCounter = transition_cycle
                .cycle
                .iter()
                .map(|&vehicle_id| tours.get(&vehicle_id).unwrap().maintenance_counter())
                .sum();

            let dead_head_distance_between_depots = match transition_cycle.cycle.len() {
                0 => 0,
                1 => {
                    let vehicle = transition_cycle.cycle.first().unwrap();
                    let tour = tours.get(vehicle).unwrap();
                    network
                        .dead_head_distance_between(
                            tour.end_depot().unwrap(),
                            tour.start_depot().unwrap(),
                        )
                        .in_meter()
                        .unwrap_or(MAX_DISTANCE)
                        as MaintenanceCounter
                }
                _ => transition_cycle
                    .cycle
                    .iter()
                    .circular_tuple_windows()
                    .map(|(vehicle_1, vehicle_2)| {
                        let end_depot_of_vehicle_1 =
                            tours.get(vehicle_1).unwrap().end_depot().unwrap();
                        let start_depot_of_vehicle_2 =
                            tours.get(vehicle_2).unwrap().start_depot().unwrap();
                        network
                            .dead_head_distance_between(
                                end_depot_of_vehicle_1,
                                start_depot_of_vehicle_2,
                            )
                            .in_meter()
                            .unwrap_or(MAX_DISTANCE)
                            as MaintenanceCounter
                    })
                    .sum::<MaintenanceCounter>(),
            };
            computed_maintenance_counter += dead_head_distance_between_depots;

            assert_eq!(
                computed_maintenance_counter,
                transition_cycle.maintenance_counter,
            );
            computed_total_maintenance_violation += computed_maintenance_counter.max(0);
        }
        assert_eq!(
            computed_total_maintenance_violation,
            self.total_maintenance_violation
        );

        // verify cycle lookup
        for (vehicle, cycle_idx) in self.cycle_lookup.iter() {
            assert!(self.cycles[*cycle_idx].cycle.contains(vehicle));
        }

        // verify empty cycles
        for empty_cycle_idx in self.empty_cycles.iter() {
            assert!(self.cycles[*empty_cycle_idx].cycle.is_empty());
        }
    }

    fn end_depot_of_predecessor_and_start_depot_of_successor(
        &self,
        vehicle: VehicleIdx,
        old_tours: &HashMap<&VehicleIdx, &Tour>,
    ) -> (NodeIdx, NodeIdx) {
        let cycle_idx = self.cycle_lookup.get(&vehicle).unwrap();

        let vehicle_idx = self.cycles[*cycle_idx]
            .cycle
            .iter()
            .position(|&v| v == vehicle)
            .unwrap();
        let predecessor = if vehicle_idx == 0 {
            self.cycles[*cycle_idx].cycle.last().unwrap()
        } else {
            self.cycles[*cycle_idx].cycle.get(vehicle_idx - 1).unwrap()
        };

        let successor = if vehicle_idx == self.cycles[*cycle_idx].cycle.len() - 1 {
            self.cycles[*cycle_idx].cycle.first().unwrap()
        } else {
            self.cycles[*cycle_idx].cycle.get(vehicle_idx + 1).unwrap()
        };

        (
            old_tours.get(predecessor).unwrap().end_depot().unwrap(),
            old_tours.get(successor).unwrap().start_depot().unwrap(),
        )
    }

    fn maintenance_counter_of_tour_plus_dead_head_trips_before_and_after(
        &self,
        tour: &Tour,
        end_depot_of_predecessor: NodeIdx,
        start_depot_of_successor: NodeIdx,
        network: &Network,
    ) -> MaintenanceCounter {
        tour.maintenance_counter()
            + network
                .dead_head_distance_between(end_depot_of_predecessor, tour.start_depot().unwrap())
                .in_meter()
                .unwrap_or(MAX_DISTANCE) as MaintenanceCounter
            + network
                .dead_head_distance_between(tour.end_depot().unwrap(), start_depot_of_successor)
                .in_meter()
                .unwrap_or(MAX_DISTANCE) as MaintenanceCounter
    }

    fn push_vehicle_to_end_of_cluster(
        cluster: &mut Vec<VehicleIdx>,
        maintenance_counter: &mut MaintenanceCounter,
        vehicle: VehicleIdx,
        tours: &HashMap<VehicleIdx, Tour>,
        network: &Network,
    ) {
        let tour = tours.get(&vehicle).unwrap();
        let dist_between_end_depot_to_start_depot = network
            .dead_head_distance_between(
                tours
                    .get(cluster.last().unwrap())
                    .unwrap()
                    .end_depot()
                    .unwrap(),
                tour.start_depot().unwrap(),
            )
            .in_meter()
            .unwrap_or(MAX_DISTANCE)
            as MaintenanceCounter;
        cluster.push(vehicle);
        *maintenance_counter += tour.maintenance_counter() + dist_between_end_depot_to_start_depot;
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
