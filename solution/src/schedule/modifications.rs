// © 2023-2024 ETH Zurich
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// any later version.
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use im::{HashMap, HashSet};
use itertools::Itertools;
use model::base_types::{
    Cost, Idx, MaintenanceCounter, NodeIdx, PassengerCount, VehicleIdx, VehicleTypeIdx,
};

use crate::{
    path::Path, segment::Segment, tour::Tour, train_formation::TrainFormation,
    transition::Transition, vehicle::Vehicle, Schedule,
};

use super::DepotUsage;

impl Schedule {
    pub fn spawn_vehicle_to_replace_dummy_tour(
        &self,
        dummy_idx: VehicleIdx,
        vehicle_type_idx: VehicleTypeIdx,
    ) -> Result<(Schedule, VehicleIdx), String> {
        let nodes: Vec<NodeIdx> = self
            .dummy_tours
            .get(&dummy_idx)
            .ok_or(format!(
                "Cannot spawn vehicle to replace dummy tour {}. Dummy tour does not exist.",
                dummy_idx
            ))?
            .all_nodes_iter()
            .collect();
        if nodes.iter().any(|n| {
            !self
                .network
                .compatible_with_vehicle_type(*n, vehicle_type_idx)
        }) {
            return Err(format!(
                "Cannot spawn vehicle to replace dummy tour {}. Nodes are not compatible with vehicle type {}.",
                dummy_idx, vehicle_type_idx,
            ));
        }
        let intermediate_schedule = self.delete_dummy(dummy_idx)?;
        intermediate_schedule.spawn_vehicle_for_path(vehicle_type_idx, nodes)
    }

    /// Spawn new vehicle for the given path.
    ///
    /// If path does not start with a depot the vehicle is spawned from the nearest availabe depot
    /// (from the start location of the first trip).
    /// Similarly, if path does not end with a depot the vehicle is spawned to the nearest depot
    /// (from the end location of the last trip).
    /// If the depot given in the path is not available, spawn vehicle from overflow depot instead.
    ///
    /// # Errors
    /// If no depot is available, an error is returned (should not be possible as there is always
    /// the overflow depot).
    /// If some node on the path is not compatible with the vehicle type an error is returned.
    /// If a train formation of some node on the path is full, an error is returned.
    pub fn spawn_vehicle_for_path(
        &self,
        vehicle_type_idx: VehicleTypeIdx,
        path_as_vec: Vec<NodeIdx>,
    ) -> Result<(Schedule, VehicleIdx), String> {
        if path_as_vec.iter().any(|n| {
            !self
                .network
                .compatible_with_vehicle_type(*n, vehicle_type_idx)
        }) {
            return Err(format!(
                "Cannot spawn vehicle for path {:?}. Nodes are not compatible with vehicle type {}.",
                path_as_vec, vehicle_type_idx,
            ));
        }

        let nodes = self.add_suitable_start_and_end_depot_to_path(vehicle_type_idx, path_as_vec)?;

        let mut vehicles = self.vehicles.clone();
        let mut tours = self.tours.clone();
        let mut next_period_transitions = self.next_period_transitions.clone();
        let mut train_formations = self.train_formations.clone();
        let mut depot_usage = self.depot_usage.clone();
        let mut vehicle_ids_grouped_and_sorted = self.vehicle_ids_grouped_and_sorted.clone();
        let mut unserved_passengers = self.unserved_passengers;
        let mut maintenance_violation = self.maintenance_violation;
        let mut costs = self.costs;

        let vehicle_id = VehicleIdx::vehicle_from(self.vehicle_counter as Idx);
        let tour = Tour::new(nodes, self.network.clone())?;
        let vehicle = Vehicle::new(vehicle_id, vehicle_type_idx, self.network.vehicle_types());

        vehicles.insert(vehicle_id, vehicle.clone());

        let position = vehicle_ids_grouped_and_sorted[&vehicle_type_idx]
            .binary_search(&vehicle_id)
            .unwrap_or_else(|e| e);
        vehicle_ids_grouped_and_sorted[&vehicle_type_idx].insert(position, vehicle_id);

        self.update_train_formation(
            &mut train_formations,
            &mut unserved_passengers,
            None,
            Some(vehicle.clone()),
            tour.all_nodes_iter(),
        )?;

        costs += tour.costs();

        tours.insert(vehicle_id, tour);

        self.update_depot_usage(&mut depot_usage, &vehicles, &tours, vehicle_id);

        self.update_transitions_and_violation_fast(
            &mut next_period_transitions,
            &mut maintenance_violation,
            vec![vehicle_id],
            &vehicles,
            &tours,
        );

        Ok((
            Schedule::new(
                vehicles,
                tours,
                next_period_transitions,
                train_formations,
                depot_usage,
                self.dummy_tours.clone(),
                self.vehicle_counter + 1,
                vehicle_ids_grouped_and_sorted,
                self.dummy_ids_sorted.clone(),
                unserved_passengers,
                maintenance_violation,
                costs,
                self.network.clone(),
            ),
            vehicle_id,
        ))
    }

    /// Delete vehicle (and its tour) from schedule.
    /// # Errors
    /// If the vehicle is not a real vehicle an error is returned.
    pub fn replace_vehicle_by_dummy(&self, vehicle_idx: VehicleIdx) -> Result<Schedule, String> {
        if !self.is_vehicle(vehicle_idx) {
            return Err(format!(
                "Cannot delete vehicle {} from schedule.",
                vehicle_idx
            ));
        }

        let vehicle_type_id = self.vehicle_type_of(vehicle_idx)?;

        let mut vehicles = self.vehicles.clone();
        let mut tours = self.tours.clone();
        let mut next_period_transitions = self.next_period_transitions.clone();
        let mut train_formations = self.train_formations.clone();
        let mut depot_usage = self.depot_usage.clone();
        let mut vehicle_ids_grouped_and_sorted = self.vehicle_ids_grouped_and_sorted.clone();
        let mut dummy_tours = self.dummy_tours.clone();
        let mut dummy_ids_sorted = self.dummy_ids_sorted.clone();
        let mut unserved_passengers = self.unserved_passengers;
        let mut maintenance_violation = self.maintenance_violation;
        let mut costs = self.costs;
        let mut vehicle_counter = self.vehicle_counter;

        vehicles.remove(&vehicle_idx);

        let position = vehicle_ids_grouped_and_sorted[&vehicle_type_id]
            .binary_search(&vehicle_idx)
            .unwrap();
        vehicle_ids_grouped_and_sorted[&vehicle_type_id].remove(position);

        self.update_train_formation(
            &mut train_formations,
            &mut unserved_passengers,
            Some(vehicle_idx),
            None,
            tours.get(&vehicle_idx).unwrap().all_nodes_iter(),
        )?;

        let tour = tours.remove(&vehicle_idx).unwrap();

        self.update_depot_usage(&mut depot_usage, &vehicles, &tours, vehicle_idx);

        costs -= tour.costs();

        if let Ok(dummy_tour) = Tour::new_dummy(
            tour.sub_path(Segment::new(tour.first_node(), tour.last_node()))?,
            self.network.clone(),
        ) {
            self.add_dummy_tour(
                &mut dummy_tours,
                &mut dummy_ids_sorted,
                VehicleIdx::dummy_from(self.vehicle_counter as Idx),
                dummy_tour,
            );
            vehicle_counter += 1;
        }

        self.update_transitions_and_violation_fast(
            &mut next_period_transitions,
            &mut maintenance_violation,
            vec![vehicle_idx],
            &vehicles,
            &tours,
        );

        Ok(Schedule::new(
            vehicles,
            tours,
            next_period_transitions,
            train_formations,
            depot_usage,
            dummy_tours,
            vehicle_counter,
            vehicle_ids_grouped_and_sorted,
            dummy_ids_sorted,
            unserved_passengers,
            maintenance_violation,
            costs,
            self.network.clone(),
        ))
    }

    /// Add a path to the tour of a vehicle (dummy or real). If the path causes conflicts, the conflicting nodes of
    /// the old tour are removed. Return the new schedule and the removed path as Option.
    /// # Errors
    /// If some node on the path is not compatible with the vehicle type (if real vehicle) an error is returned.
    /// If a train formation of some node on the path is full, an error is returned.
    pub fn add_path_to_vehicle_tour(
        &self,
        vehicle_idx: VehicleIdx,
        path: Path,
    ) -> Result<(Schedule, Option<Path>), String> {
        if let Ok(vehicle_type_id) = self.vehicle_type_of(vehicle_idx) {
            if path.iter().any(|n| {
                !self
                    .network
                    .compatible_with_vehicle_type(n, vehicle_type_id)
            }) {
                return Err(format!(
                "Cannot add path {} to vehicle tour {}. Nodes are not compatible with vehicle type {}.",
                path, vehicle_idx, vehicle_type_id,
            ));
            }
        }
        if self.network.node(path.first()).is_depot() {
            // path starts with a depot, so we need to ensure that the new depot has capacities
            // available
            let new_start_depot = path.first();
            let old_start_depot = self.tour_of(vehicle_idx).unwrap().start_depot().unwrap();

            if new_start_depot != old_start_depot
                && !self.can_depot_spawn_vehicle(
                    new_start_depot,
                    self.vehicle_type_of(vehicle_idx)
                        .expect("Vehicle must be real, as it starts with a depot"),
                )
            {
                return Err(format!(
                    "Cannot add path {} to vehicle tour {}. New start depot has no capacity available.",
                    path, vehicle_idx
                ));
            }
        }
        let mut tours = self.tours.clone();
        let mut next_period_transitions = self.next_period_transitions.clone();
        let mut train_formations = self.train_formations.clone();
        let mut depot_usage = self.depot_usage.clone();
        let mut unserved_passengers = self.unserved_passengers;
        let mut maintenance_violation = self.maintenance_violation;
        let mut costs = self.costs;

        // add vehicle to train_formations for nodes of new path
        self.update_train_formation(
            &mut train_formations,
            &mut unserved_passengers,
            None,
            Some(self.vehicles.get(&vehicle_idx).cloned().unwrap()),
            path.iter(),
        )?;

        let (new_tour, removed_path_opt) = tours.get(&vehicle_idx).unwrap().insert_path(path);

        // remove vehicle from train formations for nodes of removed path
        if let Some(ref removed_path) = removed_path_opt {
            self.update_train_formation(
                &mut train_formations,
                &mut unserved_passengers,
                Some(vehicle_idx),
                None,
                removed_path.iter(),
            )?;
        }

        costs = (costs + new_tour.costs()) - self.tours.get(&vehicle_idx).unwrap().costs();

        tours.insert(vehicle_idx, new_tour);

        self.update_depot_usage(&mut depot_usage, &self.vehicles, &tours, vehicle_idx);

        self.update_transitions_and_violation_fast(
            &mut next_period_transitions,
            &mut maintenance_violation,
            vec![vehicle_idx],
            &self.vehicles,
            &tours,
        );

        Ok((
            Schedule::new(
                self.vehicles.clone(),
                tours,
                next_period_transitions,
                train_formations,
                depot_usage,
                self.dummy_tours.clone(),
                self.vehicle_counter,
                self.vehicle_ids_grouped_and_sorted.clone(),
                self.dummy_ids_sorted.clone(),
                unserved_passengers,
                maintenance_violation,
                costs,
                self.network.clone(),
            ),
            removed_path_opt,
        ))
    }

    /// Remove segment from tour of real vehicle.
    /// All service trips are added to a new dummy tour.
    /// If the segment contains all non-depot nodes of the tour, the vehicle is replaced by a dummy.
    /// # Errors
    /// If the vehicle is not a real vehicle an error is returned.
    pub fn remove_segment(
        // TEST this
        &self,
        segment: Segment,
        vehicle_idx: VehicleIdx,
    ) -> Result<Schedule, String> {
        if !self.is_vehicle(vehicle_idx) {
            return Err(format!(
                "Cannot remove segment {} from vehicle {}. Vehicle is not a real vehicle.",
                segment, vehicle_idx,
            ));
        }
        let vehicles = self.vehicles.clone();
        let mut tours = self.tours.clone();
        let mut next_period_transitions = self.next_period_transitions.clone();
        let mut train_formations = self.train_formations.clone();
        let mut depot_usage = self.depot_usage.clone();
        let mut dummy_tours = self.dummy_tours.clone();
        let vehicle_ids_grouped_and_sorted = self.vehicle_ids_grouped_and_sorted.clone();
        let mut dummy_ids_sorted = self.dummy_ids_sorted.clone();
        let mut unserved_passengers = self.unserved_passengers;
        let mut maintenance_violation = self.maintenance_violation;
        let mut costs = self.costs;
        let mut vehicle_counter = self.vehicle_counter;

        let tour = self.tour_of(vehicle_idx).unwrap();
        let (shrinked_tour, removed_path) = tour.remove(segment)?;

        match shrinked_tour {
            None => self.replace_vehicle_by_dummy(vehicle_idx), // segment was the whole tour (except for depots)
            Some(new_tour) => {
                self.update_train_formation(
                    &mut train_formations,
                    &mut unserved_passengers,
                    Some(vehicle_idx),
                    None,
                    removed_path.iter(),
                )?;

                self.update_tour_and_costs(
                    &mut tours,
                    &mut dummy_tours,
                    &mut costs,
                    vehicle_idx,
                    new_tour,
                );

                self.update_depot_usage(&mut depot_usage, &vehicles, &tours, vehicle_idx);

                if let Ok(new_dummy_tour) = Tour::new_dummy(removed_path, self.network.clone()) {
                    self.add_dummy_tour(
                        &mut dummy_tours,
                        &mut dummy_ids_sorted,
                        VehicleIdx::dummy_from(self.vehicle_counter as Idx),
                        new_dummy_tour,
                    );
                    vehicle_counter += 1;
                }

                self.update_transitions_and_violation_fast(
                    &mut next_period_transitions,
                    &mut maintenance_violation,
                    vec![vehicle_idx],
                    &vehicles,
                    &tours,
                );

                Ok(Schedule::new(
                    vehicles,
                    tours,
                    next_period_transitions,
                    train_formations,
                    depot_usage,
                    dummy_tours,
                    vehicle_counter,
                    vehicle_ids_grouped_and_sorted,
                    dummy_ids_sorted,
                    unserved_passengers,
                    maintenance_violation,
                    costs,
                    self.network.clone(),
                ))
            }
        }
    }

    /// Tries to insert all nodes of provider's segment into receiver's tour.
    /// Nodes that causes conflcits are rejected and stay in provider's tour.
    /// Nodes that do not cause a conflict are removed from provider's tour and assigned to the receiver.
    /// # Errors
    /// If some node of the segment is not compatible with the receivers type an error is returned.
    /// (So normally receiver and provider should have the same vehicle type. Otherwise the segment
    /// should only contain depots and maintenance nodes.)
    pub fn fit_reassign(
        &self,
        segment: Segment,
        provider: VehicleIdx,
        receiver: VehicleIdx,
    ) -> Result<Schedule, String> {
        if !self.check_receiver_type_compatibility(provider, receiver, segment) {
            return Err(format!(
                "Cannot fit_reassign segment {} from vehicle {} to vehicle {}. Vehicle types do not match and segment contains service trip.",
                segment, provider, receiver,
            ));
        }
        let mut vehicles = self.vehicles.clone();
        let mut tours = self.tours.clone();
        let mut next_period_transitions = self.next_period_transitions.clone();
        let mut train_formations = self.train_formations.clone();
        let mut depot_usage = self.depot_usage.clone();
        let mut dummy_tours = self.dummy_tours.clone();
        let mut vehicle_ids_grouped_and_sorted = self.vehicle_ids_grouped_and_sorted.clone();
        let mut unserved_passengers = self.unserved_passengers;
        let mut maintenance_violation = self.maintenance_violation;
        let mut costs = self.costs;
        let mut dummy_ids_sorted = self.dummy_ids_sorted.clone();

        let (new_tour_provider, new_tour_receiver, moved_nodes) = self.fit_path_into_tour(
            self.tour_of(provider).unwrap().sub_path(segment)?,
            provider,
            receiver,
        );

        self.update_tours(
            &mut vehicles,
            &mut tours,
            &mut train_formations,
            &mut depot_usage,
            &mut dummy_tours,
            &mut vehicle_ids_grouped_and_sorted,
            &mut dummy_ids_sorted,
            &mut unserved_passengers,
            &mut costs,
            Some(provider),
            new_tour_provider,
            receiver,
            new_tour_receiver,
            moved_nodes.iter().copied(),
        )?;

        self.update_transitions_and_violation_fast(
            &mut next_period_transitions,
            &mut maintenance_violation,
            vec![provider, receiver],
            &vehicles,
            &tours,
        );

        Ok(Schedule::new(
            vehicles,
            tours,
            next_period_transitions,
            train_formations,
            depot_usage,
            dummy_tours,
            self.vehicle_counter,
            vehicle_ids_grouped_and_sorted,
            dummy_ids_sorted,
            unserved_passengers,
            maintenance_violation,
            costs,
            self.network.clone(),
        ))
    }

    /// Remove segment from provider's tour and inserts the nodes into the tour of receiver vehicle.
    /// All conflicting nodes are removed from the tour and in the case that there are conflicts
    /// a new dummy tour is created.
    /// Provider tour must be valid after removing the segment. In particular a segment including a
    /// depot can only be moved if all non-depot nodes are moved.
    /// If all non-depot of the provider are moved, the provider is deleted.
    /// # Errors
    /// If some node of the segment is not compatible with the receivers type an error is returned.
    /// (So normally receiver and provider should have the same vehicle type. Otherwise the segment
    /// should only contain depots and maintenance nodes.)
    pub fn override_reassign(
        &self,
        segment: Segment,
        provider: VehicleIdx,
        receiver: VehicleIdx,
    ) -> Result<(Schedule, Option<VehicleIdx>), String> {
        if !self.check_receiver_type_compatibility(provider, receiver, segment) {
            return Err(format!(
                "Cannot override_reassign segment {} from vehicle {} to vehicle {}. Vehicle types do not match and segment contains service trip.",
                segment, provider, receiver,
            ));
        }
        let mut vehicles = self.vehicles.clone();
        let mut tours = self.tours.clone();
        let mut next_period_transitions = self.next_period_transitions.clone();
        let mut dummy_tours = self.dummy_tours.clone();
        let mut train_formations = self.train_formations.clone();
        let mut depot_usage = self.depot_usage.clone();
        let mut vehicle_ids_grouped_and_sorted = self.vehicle_ids_grouped_and_sorted.clone();
        let mut dummy_ids_sorted = self.dummy_ids_sorted.clone();
        let mut unserved_passengers = self.unserved_passengers;
        let mut maintenance_violation = self.maintenance_violation;
        let mut costs = self.costs;
        let mut vehicle_counter = self.vehicle_counter;

        let tour_provider = self.tour_of(provider).unwrap();
        let tour_receiver = self.tour_of(receiver).unwrap();

        // remove segment for provider
        let (shrinked_tour_provider, path) = tour_provider.remove(segment)?;

        let moved_nodes: Vec<NodeIdx> = path.iter().collect();

        // insert path into tour
        let (new_tour_receiver, replaced_path) = tour_receiver.insert_path(path);

        self.update_tours(
            &mut vehicles,
            &mut tours,
            &mut train_formations,
            &mut depot_usage,
            &mut dummy_tours,
            &mut vehicle_ids_grouped_and_sorted,
            &mut dummy_ids_sorted,
            &mut unserved_passengers,
            &mut costs,
            Some(provider),
            shrinked_tour_provider,
            receiver,
            new_tour_receiver,
            moved_nodes.iter().cloned(),
        )?;

        let mut new_dummy_opt = None; // for return value

        if let Some(new_path) = replaced_path {
            if self.is_vehicle(receiver) {
                // in this case receiver needs to be removed from the train formations of the
                // removed nodes
                self.update_train_formation(
                    &mut train_formations,
                    &mut unserved_passengers,
                    Some(receiver),
                    None,
                    new_path.iter(),
                )?;
            }

            if let Ok(new_dummy_tour) = Tour::new_dummy(new_path, self.network.clone()) {
                // removed nodes contain service trips, so add a dummy tour

                let new_dummy = VehicleIdx::dummy_from(vehicle_counter as Idx);
                new_dummy_opt = Some(new_dummy);
                vehicle_counter += 1;

                self.add_dummy_tour(
                    &mut dummy_tours,
                    &mut dummy_ids_sorted,
                    new_dummy,
                    new_dummy_tour,
                );
            }
        }
        self.update_transitions_and_violation_fast(
            &mut next_period_transitions,
            &mut maintenance_violation,
            vec![provider, receiver],
            &vehicles,
            &tours,
        );

        Ok((
            Schedule::new(
                vehicles,
                tours,
                next_period_transitions,
                train_formations,
                depot_usage,
                dummy_tours,
                vehicle_counter,
                vehicle_ids_grouped_and_sorted,
                dummy_ids_sorted,
                unserved_passengers,
                maintenance_violation,
                costs,
                self.network.clone(),
            ),
            new_dummy_opt,
        ))
    }

    /// Improves the depots of all vehicles given in vehicles.
    /// If None the depots of all vehicles are improved.
    /// Assumes that vehicle are real vehicle in schedule.
    /// Panics if a vehicle is not a real vehicle.
    pub fn improve_depots(&self, vehicles: Option<Vec<VehicleIdx>>) -> Schedule {
        let mut tours = self.tours.clone();
        let mut next_period_transitions = self.next_period_transitions.clone();
        let mut depot_usage = self.depot_usage.clone();
        let mut maintenance_violation = self.maintenance_violation;
        let mut costs = self.costs;

        let recompute_all = vehicles.is_none();
        let vehicle_ids = vehicles.unwrap_or_else(|| self.vehicles_iter_all().collect());

        // first remove all considered vehicles from depots, so that there is space for reoptimization
        for vehicle_id in vehicle_ids.iter() {
            let vehicle_type_id = self.vehicle_type_of(*vehicle_id).unwrap();
            let old_tour = self.tour_of(*vehicle_id).unwrap();
            depot_usage
                .get_mut(&(
                    self.network.get_depot_idx(old_tour.start_depot().unwrap()),
                    vehicle_type_id,
                ))
                .unwrap()
                .0
                .remove(vehicle_id)
                .unwrap();
            depot_usage
                .get_mut(&(
                    self.network.get_depot_idx(old_tour.end_depot().unwrap()),
                    vehicle_type_id,
                ))
                .unwrap()
                .1
                .remove(vehicle_id)
                .unwrap();
        }

        for vehicle_id in vehicle_ids.iter() {
            let tour = self.tour_of(*vehicle_id).unwrap();
            let vehicle_type_id = self.vehicle_type_of(*vehicle_id).unwrap();
            let new_tour = self.improve_depots_of_tour(tour, vehicle_type_id, &depot_usage);

            costs = (costs + new_tour.costs()) - tour.costs();

            // add vehicle to depot_usage
            depot_usage
                .entry((
                    self.network.get_depot_idx(new_tour.start_depot().unwrap()),
                    vehicle_type_id,
                ))
                .or_insert((HashSet::new(), HashSet::new()))
                .0
                .insert(*vehicle_id);
            depot_usage
                .entry((
                    self.network.get_depot_idx(new_tour.end_depot().unwrap()),
                    vehicle_type_id,
                ))
                .or_insert((HashSet::new(), HashSet::new()))
                .1
                .insert(*vehicle_id);

            // add tour
            tours.insert(*vehicle_id, new_tour);
        }
        if recompute_all {
            // if all depots are improved, recompute transitions for all vehicle types
            self.recompute_transitions_and_violation_fast(
                &mut next_period_transitions,
                &mut maintenance_violation,
                &self.vehicle_ids_grouped_and_sorted,
                &tours,
                self.network.vehicle_types().iter().collect(), // recompute transition for all vehicle types
            );
        } else {
            // otherwise only update the changed vehicles
            self.update_transitions_and_violation_fast(
                &mut next_period_transitions,
                &mut maintenance_violation,
                vehicle_ids,
                &self.vehicles,
                &tours,
            );
        }

        Schedule::new(
            self.vehicles.clone(),
            tours,
            next_period_transitions,
            self.train_formations.clone(),
            depot_usage,
            self.dummy_tours.clone(),
            self.vehicle_counter,
            self.vehicle_ids_grouped_and_sorted.clone(),
            self.dummy_ids_sorted.clone(),
            self.unserved_passengers,
            maintenance_violation,
            costs,
            self.network.clone(),
        )
    }

    /// Reassigns the end depots of all vehicles greedily. Capacties of depots are ignored.
    pub fn reassign_end_depots_greedily(&self) -> Result<Schedule, String> {
        let mut tours = self.tours.clone();
        let mut next_period_transitions = self.next_period_transitions.clone();
        let mut depot_usage = self.depot_usage.clone();
        let mut maintenance_violation = self.maintenance_violation;
        let mut costs = self.costs;

        for vehicle_id in self.vehicles_iter_all() {
            let tour = self.tour_of(vehicle_id).unwrap();
            let last_node_location = self
                .network
                .node(tour.last_non_depot().unwrap())
                .end_location();
            let new_end_depot_node = self
                .network
                .end_depots_sorted_by_distance_from(last_node_location)
                .first()
                .copied()
                .ok_or(format!("Cannot find end depot for vehicle {}.", vehicle_id))?;

            let new_tour = tour.replace_end_depot(new_end_depot_node).unwrap();

            costs = (costs + new_tour.costs()) - tour.costs();

            tours.insert(vehicle_id, new_tour);

            self.update_depot_usage(&mut depot_usage, &self.vehicles, &tours, vehicle_id);
        }

        self.recompute_transitions_and_violation_fast(
            &mut next_period_transitions,
            &mut maintenance_violation,
            &self.vehicle_ids_grouped_and_sorted,
            &tours,
            self.network.vehicle_types().iter().collect(), // recompute transition for all vehicle types
        );

        Ok(Schedule::new(
            self.vehicles.clone(),
            tours,
            next_period_transitions,
            self.train_formations.clone(),
            depot_usage,
            self.dummy_tours.clone(),
            self.vehicle_counter,
            self.vehicle_ids_grouped_and_sorted.clone(),
            self.dummy_ids_sorted.clone(),
            self.unserved_passengers,
            maintenance_violation,
            costs,
            self.network.clone(),
        ))
    }

    // TEST this
    pub fn recompute_transitions_for(
        &self,
        vehicle_types: Option<Vec<VehicleTypeIdx>>,
    ) -> Schedule {
        let mut next_period_transitions = self.next_period_transitions.clone();
        let mut maintenance_violation = self.maintenance_violation;

        self.recompute_transitions_and_violation_fast(
            &mut next_period_transitions,
            &mut maintenance_violation,
            &self.vehicle_ids_grouped_and_sorted,
            &self.tours,
            vehicle_types.unwrap_or_else(|| self.network.vehicle_types().iter().collect()),
        );

        Schedule::new(
            self.vehicles.clone(),
            self.tours.clone(),
            next_period_transitions,
            self.train_formations.clone(),
            self.depot_usage.clone(),
            self.dummy_tours.clone(),
            self.vehicle_counter,
            self.vehicle_ids_grouped_and_sorted.clone(),
            self.dummy_ids_sorted.clone(),
            self.unserved_passengers,
            maintenance_violation,
            self.costs,
            self.network.clone(),
        )
    }

    /// Reassign the end depots such that they are consistent with the transition.
    pub fn reassign_end_depots_consistent_with_transitions(&self) -> Schedule {
        let mut tours = self.tours.clone();
        let mut next_day_transitions = self.next_period_transitions.clone();
        let mut depot_usage = self.depot_usage.clone();
        let mut maintenance_violation = self.maintenance_violation;
        let mut costs = self.costs;

        for vehicle in self.vehicles_iter_all() {
            let tour = self.tour_of(vehicle).unwrap();
            let vehicle_type = self.vehicle_type_of(vehicle).unwrap();
            let next_vehicle = self
                .next_period_transitions
                .get(&vehicle_type)
                .unwrap()
                .get_successor_of(vehicle);
            let start_depot_of_next_vehicle =
                self.tour_of(next_vehicle).unwrap().start_depot().unwrap();
            let depot_idx = self.network.get_depot_idx(start_depot_of_next_vehicle);
            let new_end_depot = self.network.get_end_depot_node(depot_idx);

            let new_tour = tour.replace_end_depot(new_end_depot).unwrap();

            costs = (costs + new_tour.costs()) - tour.costs();

            tours.insert(vehicle, new_tour);

            self.update_depot_usage(&mut depot_usage, &self.vehicles, &tours, vehicle);
        }

        self.update_transitions_and_violation_fast(
            &mut next_day_transitions,
            &mut maintenance_violation,
            self.vehicles_iter_all().collect(),
            &self.vehicles,
            &tours,
        );

        Schedule::new(
            self.vehicles.clone(),
            tours,
            next_day_transitions,
            self.train_formations.clone(),
            depot_usage,
            self.dummy_tours.clone(),
            self.vehicle_counter,
            self.vehicle_ids_grouped_and_sorted.clone(),
            self.dummy_ids_sorted.clone(),
            self.unserved_passengers,
            maintenance_violation,
            costs,
            self.network.clone(),
        )
    }
}

// private methods
impl Schedule {
    /// Delete dummy vehicle (and its tour) from schedule.
    fn delete_dummy(&self, dummy: VehicleIdx) -> Result<Schedule, String> {
        if !self.is_dummy(dummy) {
            return Err(format!(
                "Cannot delete vehicle {} from schedule. It is not a dummy vehicle.",
                dummy
            ));
        }
        let mut dummy_tours = self.dummy_tours.clone();
        let mut dummy_ids_sorted = self.dummy_ids_sorted.clone();

        dummy_tours.remove(&dummy);
        dummy_ids_sorted.remove(dummy_ids_sorted.binary_search(&dummy).unwrap());

        Ok(Schedule::new(
            self.vehicles.clone(),
            self.tours.clone(),
            self.next_period_transitions.clone(),
            self.train_formations.clone(),
            self.depot_usage.clone(),
            dummy_tours,
            self.vehicle_counter,
            self.vehicle_ids_grouped_and_sorted.clone(),
            dummy_ids_sorted,
            self.unserved_passengers,
            self.maintenance_violation,
            self.costs,
            self.network.clone(),
        ))
    }

    fn check_receiver_type_compatibility(
        &self,
        provider: VehicleIdx,
        receiver: VehicleIdx,
        segment: Segment,
    ) -> bool {
        if let Ok(vehicle_type_of_receiver) = self.vehicle_type_of(receiver) {
            let vehicle_type_of_provider_result = self.vehicle_type_of(provider);
            if vehicle_type_of_provider_result.is_err()
                || vehicle_type_of_provider_result.unwrap() != vehicle_type_of_receiver
            {
                // vehicle types do not match, check if there are any service trip in the segment
                if self
                    .tour_of(provider)
                    .unwrap()
                    .sub_path(segment)
                    .unwrap()
                    .iter()
                    .any(|n| {
                        !self
                            .network
                            .compatible_with_vehicle_type(n, vehicle_type_of_receiver)
                    })
                {
                    return false;
                }
            }
        }
        true
    }

    /// Reassign vehicles to the new tours.
    /// The depots of the tours are improved.
    /// Updates all relevant data structures.
    /// It is assumed that provider (if some) and receiver are part of self.vehicles.
    #[allow(clippy::too_many_arguments)]
    fn update_tours(
        &self,
        vehicles: &mut HashMap<VehicleIdx, Vehicle>,
        tours: &mut HashMap<VehicleIdx, Tour>,
        train_formations: &mut HashMap<NodeIdx, TrainFormation>,
        depot_usage: &mut DepotUsage,
        dummy_tours: &mut HashMap<VehicleIdx, Tour>,
        vehicle_ids_grouped_and_sorted: &mut HashMap<VehicleTypeIdx, Vec<VehicleIdx>>,
        dummy_ids_sorted: &mut Vec<VehicleIdx>,
        unserved_passengers: &mut (PassengerCount, PassengerCount),
        costs: &mut Cost,
        provider: Option<VehicleIdx>,    // None: there is no provider
        new_tour_provider: Option<Tour>, // None: provider is deleted
        receiver: VehicleIdx,
        new_tour_receiver: Tour,
        moved_nodes: impl Iterator<Item = NodeIdx>,
    ) -> Result<(), String> {
        if let Some(provider_id) = provider {
            // update tour of the provider
            match new_tour_provider {
                Some(new_tour) => {
                    self.update_tour_and_costs(tours, dummy_tours, costs, provider_id, new_tour);
                }
                None => {
                    // there is a provider but no tour -> delete provider
                    if self.is_vehicle(provider_id) {
                        *costs -= self.tour_of(provider_id).unwrap().costs();
                    }
                    if self.is_dummy(provider_id) {
                        dummy_tours.remove(&provider_id); // old_dummy_tour is completely removed
                        dummy_ids_sorted
                            .remove(dummy_ids_sorted.binary_search(&provider_id).unwrap());
                    } else if self.is_vehicle(provider_id) {
                        let provider_vehicle_type = self.vehicle_type_of(provider_id).unwrap();
                        vehicles.remove(&provider_id);
                        tours.remove(&provider_id); // old_tour is completely removed

                        let position = vehicle_ids_grouped_and_sorted[&provider_vehicle_type]
                            .binary_search(&provider_id)
                            .unwrap();
                        vehicle_ids_grouped_and_sorted[&provider_vehicle_type].remove(position);
                    }
                }
            }
            self.update_depot_usage(depot_usage, vehicles, tours, provider_id);
        }

        // update extended tour of the receiver
        self.update_tour_and_costs(tours, dummy_tours, costs, receiver, new_tour_receiver);
        self.update_depot_usage(depot_usage, vehicles, tours, receiver);

        // update train_formations
        let receiver_vehicle = self.vehicles.get(&receiver).cloned();
        self.update_train_formation(
            train_formations,
            unserved_passengers,
            provider,
            receiver_vehicle,
            moved_nodes,
        )?;
        Ok(())
    }

    fn update_tour_and_costs(
        &self,
        tours: &mut HashMap<VehicleIdx, Tour>,
        dummy_tours: &mut HashMap<VehicleIdx, Tour>,
        costs: &mut Cost,
        vehicle: VehicleIdx,
        new_tour: Tour,
    ) {
        if self.is_dummy(vehicle) {
            dummy_tours.insert(vehicle, new_tour);
        } else {
            *costs = (*costs + new_tour.costs()) - tours.get(&vehicle).unwrap().costs();
            tours.insert(vehicle, new_tour);
        }
    }

    fn update_train_formation(
        &self,
        train_formations: &mut HashMap<NodeIdx, TrainFormation>,
        unserved_passengers: &mut (PassengerCount, PassengerCount),
        provider: Option<VehicleIdx>,      // None: only add receiver
        receiver_vehicle: Option<Vehicle>, // None: only delete provider
        moved_nodes: impl Iterator<Item = NodeIdx>,
    ) -> Result<(), String> {
        for node in moved_nodes {
            if self.network.node(node).is_depot() {
                continue;
            }
            if self.network.node(node).is_service() {
                let unserved_passengers_before = Schedule::compute_unserved_passengers_at_node(
                    &self.network,
                    node,
                    train_formations.get(&node).unwrap(),
                );
                unserved_passengers.0 -= unserved_passengers_before.0;
                unserved_passengers.1 -= unserved_passengers_before.1;
            }
            train_formations.insert(
                node,
                self.vehicle_replacement_in_train_formation(
                    train_formations,
                    provider,
                    receiver_vehicle.clone(),
                    node,
                )?,
            );
            if self.network.node(node).is_service() {
                let unserved_passengers_after = Schedule::compute_unserved_passengers_at_node(
                    &self.network,
                    node,
                    train_formations.get(&node).unwrap(),
                );
                unserved_passengers.0 += unserved_passengers_after.0;
                unserved_passengers.1 += unserved_passengers_after.1;
            }
        }
        Ok(())
    }

    /// Replace a vehicle in the train formation of a node.
    /// Provider or receiver can be a dummy vehicle.
    fn vehicle_replacement_in_train_formation(
        &self,
        train_formations: &HashMap<NodeIdx, TrainFormation>,
        provider: Option<VehicleIdx>,
        receiver_vehicle: Option<Vehicle>,
        node: NodeIdx,
    ) -> Result<TrainFormation, String> {
        let old_formation = train_formations
            .get(&node)
            .unwrap_or_else(|| panic!("Node {} has no train formations.", node));

        match receiver_vehicle {
            Some(receiver_vh) if !self.is_dummy(receiver_vh.idx()) => {
                match provider {
                    Some(prov) if !self.is_dummy(prov) => {
                        // both are real vehicles
                        old_formation.replace(prov, receiver_vh)
                    }
                    _ => {
                        // provider is None or dummy
                        if self.network.node(node).is_maintenance()
                            && old_formation.vehicle_count()
                                >= self.network.track_count_of_maintenance_slot(node)
                        {
                            return Err(format!(
                                "Cannot add vehicle {} to maintenance node {}. Maintenance slot is already full.",
                                receiver_vh.idx(),
                                node
                            ));
                        }
                        if self.network.node(node).is_service() {
                            if let Some(max_length) = self.network.maximal_formation_count_for(node)
                            {
                                if old_formation.vehicle_count() >= max_length {
                                    return Err(format!(
                                        "Cannot add vehicle {} to node {}. Formation is full.",
                                        receiver_vh.idx(),
                                        node
                                    ));
                                }
                            }
                        }
                        Ok(old_formation.add_at_tail(receiver_vh))
                    }
                }
            }
            _ => {
                // receiver_vehicle is None or dummy
                match provider {
                    Some(prov) if !self.is_dummy(prov) => {
                        // provider is real vehicle
                        old_formation.remove(prov)
                    }
                    _ => {
                        // provider is None or dummy
                        Ok(old_formation.clone())
                    }
                }
            }
        }
    }

    /// Updates the provided depot_usage data structure.
    /// The vehicle is removed from the old depots (if it was a real vehicle in self).
    /// The vehicle is added to the new depots if vehicle is real in new schedule (given by vehicles) and new_tour is Some.
    fn update_depot_usage(
        &self,
        depot_usage: &mut DepotUsage,
        vehicles: &HashMap<VehicleIdx, Vehicle>,
        tours: &HashMap<VehicleIdx, Tour>,
        vehicle_idx: VehicleIdx,
    ) {
        match vehicles.get(&vehicle_idx) {
            Some(vehicle) => self.update_depot_usage_assuming_no_dummies(
                depot_usage,
                vehicle.clone(),
                tours.get(&vehicle_idx),
            ),
            None => {
                // vehicle is dummy in new schedule or is deleted
                if let Some(vehicle) = self.vehicles.get(&vehicle_idx) {
                    self.update_depot_usage_assuming_no_dummies(depot_usage, vehicle.clone(), None)
                }
            }
        }
    }

    /// Updates the provided depot_usage data structure.
    /// The vehicle is removed from the old depots (if it was a real vehicle in self).
    /// The vehicle is added to the new depots if new_tour is Some.
    /// It is assumed that the vehicle is no dummy for the new schedule.
    fn update_depot_usage_assuming_no_dummies(
        &self,
        depot_usage: &mut DepotUsage,
        vehicle: Vehicle,
        new_tour: Option<&Tour>,
    ) {
        let new_start_depot_node = new_tour.map(|t| t.start_depot().unwrap());
        let new_end_depot_node = new_tour.map(|t| t.end_depot().unwrap());
        self.update_depot_usage_for_new_start_depot(
            depot_usage,
            vehicle.clone(),
            new_start_depot_node,
        );
        self.update_depot_usage_for_new_end_depot(depot_usage, vehicle, new_end_depot_node);
    }

    fn update_depot_usage_for_new_start_depot(
        &self,
        depot_usage: &mut DepotUsage,
        vehicle: Vehicle,
        new_start_depot_node: Option<NodeIdx>, // if None, the vehicle is deleted
    ) {
        let vehicle_type = vehicle.type_idx();
        let vehicle_id = vehicle.idx();

        if self.is_vehicle(vehicle_id) {
            let old_depot = self
                .network
                .get_depot_idx(self.tour_of(vehicle_id).unwrap().start_depot().unwrap());
            depot_usage
                .entry((old_depot, vehicle_type))
                .or_insert((HashSet::new(), HashSet::new()))
                .0
                .remove(&vehicle_id)
                .unwrap();
        }

        if let Some(start_depot_node) = new_start_depot_node {
            let new_depot = self.network.get_depot_idx(start_depot_node);
            depot_usage
                .entry((new_depot, vehicle_type))
                .or_insert((HashSet::new(), HashSet::new()))
                .0
                .insert(vehicle_id);
        }
    }

    fn update_depot_usage_for_new_end_depot(
        &self,
        depot_usage: &mut DepotUsage,
        vehicle: Vehicle,
        new_end_depot_node: Option<NodeIdx>, // if None, the vehicle is deleted
    ) {
        let vehicle_type = vehicle.type_idx();
        let vehicle_id = vehicle.idx();

        if self.is_vehicle(vehicle_id) {
            let old_depot = self
                .network
                .get_depot_idx(self.tour_of(vehicle_id).unwrap().end_depot().unwrap());
            depot_usage
                .entry((old_depot, vehicle_type))
                .or_insert((HashSet::new(), HashSet::new()))
                .1
                .remove(&vehicle_id)
                .unwrap();
        }

        if let Some(end_depot_node) = new_end_depot_node {
            let new_depot = self.network.get_depot_idx(end_depot_node);
            depot_usage
                .entry((new_depot, vehicle_type))
                .or_insert((HashSet::new(), HashSet::new()))
                .1
                .insert(vehicle_id);
        }
    }

    fn recompute_transitions_and_violation_fast(
        &self,
        transitions: &mut HashMap<VehicleTypeIdx, Transition>,
        maintenance_violation: &mut MaintenanceCounter,
        vehicle_ids_grouped_by_type: &HashMap<VehicleTypeIdx, Vec<VehicleIdx>>,
        tours: &HashMap<VehicleIdx, Tour>,
        vehicle_types: Vec<VehicleTypeIdx>,
    ) {
        for vehicle_type in vehicle_types.iter() {
            let vehicle_ids = vehicle_ids_grouped_by_type.get(vehicle_type).unwrap();
            let new_transition = Transition::new_fast(vehicle_ids, tours, &self.network);
            *maintenance_violation += new_transition.maintenance_violation();
            let old_transition = transitions
                .insert(*vehicle_type, new_transition)
                .expect("Each vehicle type must be a key in transitions.");
            *maintenance_violation -= old_transition.maintenance_violation();
        }
    }

    // if changed_vehicle was a vehicle in the old schedule, just update the maintenance counter in
    // transition
    // if changed_vehicle is a new vehicle put it in its own maintenance cycle
    fn update_transitions_and_violation_fast(
        &self,
        transitions: &mut HashMap<VehicleTypeIdx, Transition>,
        maintenance_violation: &mut MaintenanceCounter,
        changed_vehicles: Vec<VehicleIdx>,
        vehicles: &HashMap<VehicleIdx, Vehicle>,
        tours: &HashMap<VehicleIdx, Tour>,
    ) {
        // as we update the transitions one by one we need to keep track of the tours that are
        // updated so far
        let mut tours_updated_one_by_one: HashMap<VehicleIdx, &Tour> = HashMap::new();

        for vehicle in changed_vehicles.iter().filter(|v| v.is_real()) {
            let vehicle_type = vehicles
                .get(vehicle)
                .unwrap_or_else(|| self.vehicles.get(vehicle).unwrap())
                .type_idx();
            let old_transition = transitions.get(&vehicle_type).unwrap();
            let new_transition =
                match (self.is_vehicle(*vehicle), vehicles.keys().contains(vehicle)) {
                    (true, true) => {
                        // vehicle was a vehicle in the old schedule and will be in the new one
                        let new_tour = tours.get(vehicle).unwrap();
                        let new_transition = old_transition.update_vehicle(
                            *vehicle,
                            new_tour,
                            &tours_updated_one_by_one,
                            &self.tours,
                            &self.get_network(),
                        );
                        tours_updated_one_by_one.insert(*vehicle, new_tour);
                        new_transition
                    }
                    (false, true) => {
                        // vehicles was newly added
                        let new_tour = tours.get(vehicle).unwrap();
                        let new_transition = old_transition.add_vehicle_to_own_cycle(
                            *vehicle,
                            new_tour,
                            &self.get_network(),
                        );
                        tours_updated_one_by_one.insert(*vehicle, new_tour);
                        new_transition
                    }
                    (true, false) => {
                        old_transition.remove_vehicle(
                            // vehicles is removed
                            *vehicle,
                            &tours_updated_one_by_one,
                            &self.tours,
                            &self.get_network(),
                        )
                    }
                    _ => unreachable!(),
                };

            *maintenance_violation = (*maintenance_violation
                + new_transition.maintenance_violation())
                - old_transition.maintenance_violation();
            transitions.insert(vehicle_type, new_transition);
        }
    }

    fn add_dummy_tour(
        &self,
        dummy_tours: &mut HashMap<VehicleIdx, Tour>,
        dummy_ids_sorted: &mut Vec<VehicleIdx>,
        new_dummy_idx: VehicleIdx,
        new_dummy_tour: Tour,
    ) {
        dummy_tours.insert(new_dummy_idx, new_dummy_tour);
        dummy_ids_sorted.insert(
            dummy_ids_sorted
                .binary_search(&new_dummy_idx)
                .unwrap_or_else(|e| e),
            new_dummy_idx,
        );
    }

    /// go through the path that should be inserted without causing conflcits.
    /// As dead_head_trips might be longer than service trips we do not iterate over all nodes
    /// individually but instead cut the path into maximal segments that could be reassigned.
    ///
    /// Hence, we iteratively consider the first node of the remaining_path as the start of a
    /// segment and take the biggest segment that can be reassigned.
    /// Afterwards this segment is removed.
    ///
    /// Assumes that path is a sub path of the tour of provider.
    ///
    /// Returns: (new_tour_provider, new_tour_receiver, moved_nodes)
    /// None for new_tour_provider means there is no tour left.
    fn fit_path_into_tour(
        &self,
        path: Path,
        provider: VehicleIdx,
        receiver: VehicleIdx,
    ) -> (Option<Tour>, Tour, Vec<NodeIdx>) {
        let mut new_tour_provider = Some(self.tour_of(provider).unwrap().clone());
        let mut new_tour_receiver = self.tour_of(receiver).unwrap().clone();
        let mut remaining_path = Some(path);
        let mut moved_nodes = Vec::new();

        while let Some(path) = remaining_path {
            let sub_segment_start = path.first();
            let (end_pos, sub_segment_end) =
                match new_tour_receiver.latest_not_reaching_node(sub_segment_start) {
                    None => (path.length() - 1, path.last()),
                    Some(pos) => {
                        // the segment can only be inserted before the blocker
                        let blocker = new_tour_receiver.nth_node(pos).unwrap();
                        // consider all nodes that arrive before the departure of the blocker
                        // test all of them if they can reach the blocker.
                        // test all of them if this segment could be removed.
                        // take the latest node of those.
                        // If empty this segment will fail, so return path.first()
                        path.iter()
                            .enumerate()
                            .map_while(|(i, n)| {
                                if self.network.node(n).end_time()
                                    > self.network.node(blocker).start_time()
                                {
                                    None
                                } else {
                                    Some((i, n))
                                }
                            })
                            .filter(|(_, n)| self.network.can_reach(*n, blocker))
                            .filter(|(_, n)| {
                                new_tour_provider
                                    .as_ref()
                                    .unwrap()
                                    .check_removable(Segment::new(sub_segment_start, *n))
                                    .is_ok()
                            })
                            .last()
                            .unwrap_or((0, path.first()))
                    }
                };

            let mut node_sequence = path.consume();
            remaining_path =
                Path::new_trusted(node_sequence.split_off(end_pos + 1), self.network.clone());
            let sub_segment = Segment::new(sub_segment_start, sub_segment_end);
            let remove_result = new_tour_provider.as_ref().unwrap().remove(sub_segment);

            if remove_result.is_err() {
                continue;
            }

            let (new_tour_provider_candidate, path_for_insertion) = remove_result.unwrap();

            // test if inserting sub_segment would cause any conflicts (or fail for other reasons
            if new_tour_receiver.conflict(sub_segment).is_some() {
                continue;
            }
            let (receiver, _) = new_tour_receiver.insert_path(path_for_insertion);

            new_tour_provider = new_tour_provider_candidate;
            new_tour_receiver = receiver;
            moved_nodes.extend(node_sequence);
        }
        (new_tour_provider, new_tour_receiver, moved_nodes)
    }

    fn improve_depots_of_tour(
        &self,
        tour: &Tour,
        vehicle_type_idx: VehicleTypeIdx,
        depot_usage: &DepotUsage,
    ) -> Tour {
        let first_non_depot = tour.first_non_depot().unwrap();
        let new_start_depot =
            self.find_best_start_depot_for_spawning(vehicle_type_idx, first_non_depot, depot_usage);
        let intermediate_tour = if new_start_depot != tour.start_depot().unwrap() {
            tour.replace_start_depot(new_start_depot).unwrap()
        } else {
            tour.clone()
        };

        let last_non_depot = intermediate_tour.last_non_depot().unwrap();
        let new_end_depot = self
            .find_best_end_depot_for_despawning(vehicle_type_idx, last_non_depot)
            .unwrap();
        if new_end_depot != intermediate_tour.end_depot().unwrap() {
            intermediate_tour.replace_end_depot(new_end_depot).unwrap()
        } else {
            intermediate_tour
        }
    }

    fn add_suitable_start_and_end_depot_to_path(
        &self,
        vehicle_type_idx: VehicleTypeIdx,
        mut nodes: Vec<NodeIdx>,
    ) -> Result<Vec<NodeIdx>, String> {
        let first_node = *nodes.first().unwrap();
        let last_node = *nodes.last().unwrap();

        // check if depot is available
        if self.network.node(first_node).is_depot()
            && !self.can_depot_spawn_vehicle(first_node, vehicle_type_idx)
        {
            // if given depot is not available, use overflow depot
            let overflow_depot_ids = self.network.overflow_depot_idxs();
            nodes[0] = overflow_depot_ids.1;
            let tour_len = nodes.len();
            let _ = std::mem::replace(&mut nodes[tour_len - 1], overflow_depot_ids.2);

            /* println!(
                "\x1b[93mwarning:\x1b[0m Tour for {} violates depot constraints at {}. Using overflow depot instead.",
                self.network.vehicle_types().get(vehicle_type_id).unwrap(),
                self.network.node(old_start_depot)
            ); */

            return Ok(nodes);
        }

        // if path does not start with a depot, insert the nearest available start_depot
        if !self.network.node(first_node).is_depot() {
            let new_start_depot = self.find_best_start_depot_for_spawning(
                vehicle_type_idx,
                first_node,
                &self.depot_usage,
            );
            nodes.insert(0, new_start_depot);
        }

        // if path does not end with a depot, insert the nearest available end_depot
        if !self.network.node(last_node).is_depot() {
            match self.find_best_end_depot_for_despawning(vehicle_type_idx, last_node) {
                Ok(depot) => nodes.push(depot),
                Err(e) => return Err(e),
            };
        }

        Ok(nodes)
    }

    fn find_best_start_depot_for_spawning(
        &self,
        vehicle_type_idx: VehicleTypeIdx,
        first_node: NodeIdx,
        depot_usage: &DepotUsage,
    ) -> NodeIdx {
        let start_location = self.network.node(first_node).start_location();
        let start_depot = self
            .network
            .start_depots_sorted_by_distance_to(start_location)
            .iter()
            .copied()
            .find(|depot| {
                self.can_depot_spawn_vehicle_custom_usage(*depot, vehicle_type_idx, depot_usage)
            })
            .expect("There should be at least the overflow depot available.");
        /* if start_depot == self.network.overflow_depot_ids().1 {
            println!(
                "\x1b[93mwarning:\x1b[0m Tour for vehicle_type {} violates depot constraints at {}. Using overflow depot instead.",
                self.network.vehicle_types().get(vehicle_type_id).unwrap(),
                self.network.node(first_node)
            );
        } */
        start_depot
    }

    fn find_best_end_depot_for_despawning(
        &self,
        vehicle_type_idx: VehicleTypeIdx,
        last_node: NodeIdx,
    ) -> Result<NodeIdx, String> {
        let end_location = self.network.node(last_node).end_location();
        let end_depot = self
            .network
            .end_depots_sorted_by_distance_from(end_location)
            .first()
            .copied();
        match end_depot {
            Some(depot) => Ok(depot),
            None => Err(format!(
                "Cannot de-spawn vehicle of type {} for end_node {}. No end_depot available.",
                vehicle_type_idx, last_node,
            )),
        }
    }
}
