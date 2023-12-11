use sbb_model::base_types::{NodeId, VehicleId, VehicleTypeId};

use crate::{
    path::{Path, Segment},
    tour::Tour,
    train_formation::TrainFormation,
    vehicle::Vehicle,
    Schedule,
};

impl Schedule {
    pub fn spawn_vehicle_to_replace_dummy_tour(
        &self,
        dummy_id: VehicleId,
        vehicle_type_id: VehicleTypeId,
    ) -> Result<Schedule, String> {
        let nodes: Vec<NodeId> = self
            .dummy_tours
            .get(&dummy_id)
            .unwrap()
            .all_nodes_iter()
            .collect();
        let intermediate_schedule = self.delete_vehicle(dummy_id)?;
        intermediate_schedule.spawn_vehicle_for_path(vehicle_type_id, nodes)
    }

    /// Spawn new vehicle.
    /// If path does not start with a depot the vehicle is spawned from the nearest availabe depot
    /// (from the start location of the first trip).
    /// Similarly, if path does not end with a depot the vehicle is spawned to the nearest depot
    /// (from the end location of the last trip).
    /// If no depot is available, an error is returned.
    /// If the depot given in the path in not available, an error is returned.
    pub fn spawn_vehicle_for_path(
        &self,
        vehicle_type_id: VehicleTypeId,
        path_as_vec: Vec<NodeId>,
    ) -> Result<Schedule, String> {
        let nodes = self.add_suitable_start_and_end_depot_to_path(vehicle_type_id, path_as_vec)?;

        let mut vehicles = self.vehicles.clone();
        let mut tours = self.tours.clone();
        let mut train_formations = self.train_formations.clone();
        let mut vehicle_ids_sorted = self.vehicle_ids_sorted.clone();

        let vehicle_id = VehicleId::from(format!("veh{:03}", self.vehicle_counter).as_str());

        let tour = Tour::new(nodes, self.network.clone())?;

        // update vehicles
        let vehicle = Vehicle::new(vehicle_id, vehicle_type_id, self.vehicle_types.clone());
        vehicles.insert(vehicle_id, vehicle.clone());

        // update train_formations
        for node in tour.all_non_depot_nodes_iter() {
            let new_formation = train_formations
                .get(&node)
                .unwrap()
                .add_at_tail(vehicle.clone());
            train_formations.insert(node, new_formation);
        }

        // update tours
        tours.insert(vehicle_id, tour);

        // update vehicle_ids_sorted
        vehicle_ids_sorted.insert(
            vehicle_ids_sorted
                .binary_search(&vehicle_id)
                .unwrap_or_else(|e| e),
            vehicle_id,
        );

        Ok(Schedule {
            vehicles,
            tours,
            train_formations,
            dummy_tours: self.dummy_tours.clone(),
            vehicle_ids_sorted,
            dummy_ids_sorted: self.dummy_ids_sorted.clone(),
            vehicle_counter: self.vehicle_counter + 1,
            config: self.config.clone(),
            vehicle_types: self.vehicle_types.clone(),
            network: self.network.clone(),
        })
    }

    /// Delete vehicle (and its tour) from schedule.
    pub fn delete_vehicle(&self, vehicle: VehicleId) -> Result<Schedule, String> {
        if self.is_dummy(vehicle) {
            return Err(format!(
                "Cannot delete dummy vehicle {} from schedule.",
                vehicle
            ));
        }
        let mut vehicles = self.vehicles.clone();
        let mut tours = self.tours.clone();
        let mut train_formations = self.train_formations.clone();
        let mut vehicle_ids_sorted = self.vehicle_ids_sorted.clone();

        // remove vehicle and tour
        vehicles.remove(&vehicle);
        tours.remove(&vehicle);
        vehicle_ids_sorted.remove(vehicle_ids_sorted.binary_search(&vehicle).unwrap());

        // remove vehicle from train formations
        for node in self.tours.get(&vehicle).unwrap().all_nodes_iter() {
            let new_formation = train_formations
                .get(&node)
                .unwrap()
                .remove(vehicle)
                .unwrap();
            train_formations.insert(node, new_formation);
        }

        Ok(Schedule {
            vehicles,
            tours,
            train_formations,
            dummy_tours: self.dummy_tours.clone(),
            vehicle_ids_sorted,
            dummy_ids_sorted: self.dummy_ids_sorted.clone(),
            vehicle_counter: self.vehicle_counter,
            config: self.config.clone(),
            vehicle_types: self.vehicle_types.clone(),
            network: self.network.clone(),
        })
    }

    /// Add a path to the tour of a vehicle. If the path causes conflicts, the conflicting nodes of
    /// the old tour are removed.
    pub fn add_path_to_vehicle_tour(
        &self,
        vehicle: VehicleId,
        path: Path,
    ) -> Result<Schedule, String> {
        let mut tours = self.tours.clone();
        let mut train_formations = self.train_formations.clone();

        // add vehicle to train_formations for nodes of new path
        for node in path.iter() {
            let new_formation = train_formations
                .get(&node)
                .unwrap()
                .add_at_tail(self.vehicles.get(&vehicle).cloned().unwrap());
            train_formations.insert(node, new_formation);
        }

        let (new_tour, removed_path_opt) = tours.get(&vehicle).unwrap().insert_path(path);

        // remove vehicle from train formations for nodes of removed path
        if let Some(removed_path) = removed_path_opt {
            for node in removed_path.iter() {
                let new_formation = train_formations
                    .get(&node)
                    .unwrap()
                    .remove(vehicle)
                    .unwrap();
                train_formations.insert(node, new_formation);
            }
        }

        // update tours
        tours.insert(vehicle, new_tour);

        Ok(Schedule {
            vehicles: self.vehicles.clone(),
            tours,
            train_formations,
            dummy_tours: self.dummy_tours.clone(),
            vehicle_ids_sorted: self.vehicle_ids_sorted.clone(),
            dummy_ids_sorted: self.dummy_ids_sorted.clone(),
            vehicle_counter: self.vehicle_counter,
            config: self.config.clone(),
            vehicle_types: self.vehicle_types.clone(),
            network: self.network.clone(),
        })
    }

    /// Reassigns a path (given by a segment and a provider) to the tour of receiver.
    /// Aborts if there are any conflicts.
    pub fn cautious_reassign(
        &self,
        segment: Segment,
        provider: VehicleId,
        receiver: VehicleId,
    ) -> Result<Schedule, String> {
        if self.conflict(segment, receiver).is_some() {
            return Err(format!(
                "There are conflcits. Abort cautious_reassign()! Segment: {} Provider: {} Receiver: {}", segment, provider, receiver
            ));
        }
        self.override_reassign(segment, provider, receiver)
            .map(|tuple| tuple.0)
    }

    /// Tries to insert all nodes of provider's segment into receiver's tour.
    /// Nodes that causes conflcits are rejected and stay in provider's tour.
    /// Nodes that do not cause a conflict are reassigned to the receiver.
    pub fn fit_reassign(
        &self,
        segment: Segment,
        provider: VehicleId,
        receiver: VehicleId,
    ) -> Result<Schedule, String> {
        // do lazy clones of schedule:
        let mut vehicles = self.vehicles.clone();
        let mut tours = self.tours.clone();
        let mut dummy_tours = self.dummy_tours.clone();
        let mut train_formations = self.train_formations.clone();
        let mut vehicle_ids_sorted = self.vehicle_ids_sorted.clone();
        let mut dummy_ids_sorted = self.dummy_ids_sorted.clone();

        let tour_provider = self.tour_of(provider)?;
        let tour_receiver = self.tour_of(receiver)?;
        let mut new_tour_provider = Some(tour_provider.clone());
        let mut new_tour_receiver = tour_receiver.clone();
        let mut remaining_path = Some(tour_provider.sub_path(segment)?);
        let mut moved_nodes = Vec::new();

        // go through the path that should be inserted without causing conflcits.
        // As dead_head_trips might be longer than service trips we do not iterate over all nodes
        // individually but instead cut the path into maximal segments that could be reassigned.
        //
        // Hence we iteratively consider the first node of the remaining_path as the start of a
        // segment and take the biggest segment that can be reassigned.
        // Afterwards this segment is removed

        while let Some(path) = remaining_path {
            let sub_segment_start = path.first();
            let (end_pos, sub_segment_end) =
                match new_tour_receiver.latest_not_reaching_node(sub_segment_start) {
                    None => (path.len() - 1, path.last()),
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

        // update reduced tour of the provider
        match new_tour_provider {
            Some(new_tour) => {
                if self.is_dummy(provider) {
                    dummy_tours.insert(provider, new_tour);
                } else {
                    tours.insert(provider, new_tour);
                }
            }
            None => {
                if self.is_dummy(provider) {
                    dummy_tours.remove(&provider); // old_dummy_tour is completely removed
                    dummy_ids_sorted.remove(dummy_ids_sorted.binary_search(&provider).unwrap());
                } else {
                    vehicles.remove(&provider);
                    tours.remove(&provider); // old_tour is completely removed
                    vehicle_ids_sorted.remove(vehicle_ids_sorted.binary_search(&provider).unwrap());
                }
            }
        }

        // update extended tour of the receiver
        if self.is_dummy(receiver) {
            // dummy_objective_info.insert(receiver, new_tour_receiver.overhead_time());
            dummy_tours.insert(receiver, new_tour_receiver);
        } else {
            // vehicle_objective_info.insert(
            // receiver,
            // ObjectiveInfo::new(self.vehicles.get_vehicle(receiver), &new_tour_receiver),
            // );
            tours.insert(receiver, new_tour_receiver);
        }

        // update train_formations:
        for node in moved_nodes.iter() {
            if self.network.node(*node).is_depot() {
                continue;
            }
            train_formations.insert(
                *node,
                self.vehicle_replacement_in_train_formation(provider, receiver, *node)
                    .unwrap(),
            );
        }

        Ok(Schedule {
            vehicles,
            tours,
            train_formations,
            dummy_tours,
            vehicle_ids_sorted,
            dummy_ids_sorted,
            vehicle_counter: self.vehicle_counter,
            config: self.config.clone(),
            vehicle_types: self.vehicle_types.clone(),
            network: self.network.clone(),
        })
    }

    /// Remove segment from provider's tour and inserts the nodes into the tour of receiver vehicle.
    /// All conflicting nodes are removed from the tour and in the case that there are conflicts
    /// a new dummy tour is created.
    pub fn override_reassign(
        &self,
        segment: Segment,
        provider: VehicleId,
        receiver: VehicleId,
    ) -> Result<(Schedule, Option<VehicleId>), String> {
        // do lazy clones of schedule:
        let mut vehicles = self.vehicles.clone();
        let mut tours = self.tours.clone();
        let mut dummy_tours = self.dummy_tours.clone();
        let mut train_formations = self.train_formations.clone();
        let mut vehicle_ids_sorted = self.vehicle_ids_sorted.clone();
        let mut dummy_ids_sorted = self.dummy_ids_sorted.clone();
        let mut dummy_counter = self.vehicle_counter;
        // let mut vehicle_objective_info = self.vehicle_objective_info.clone();
        // let mut dummy_objective_info = self.dummy_objective_info.clone();

        let tour_provider = self.tour_of(provider)?;
        let tour_receiver = self.tour_of(receiver)?;

        // remove segment for provider
        let (shrinked_tour_provider, path) = tour_provider.remove(segment)?;

        // update train_formations:
        for node in path.iter() {
            if self.network.node(node).is_depot() {
                continue;
            }
            train_formations.insert(
                node,
                self.vehicle_replacement_in_train_formation(provider, receiver, node)
                    .unwrap(),
            );
        }

        // insert path into tour
        let (new_tour_receiver, replaced_path) = tour_receiver.insert_path(path);

        // update shrinked tour of the provider
        match shrinked_tour_provider {
            Some(new_tour) => {
                if self.is_dummy(provider) {
                    dummy_tours.insert(provider, new_tour);
                } else {
                    tours.insert(provider, new_tour);
                }
            }
            None => {
                if self.is_dummy(provider) {
                    dummy_tours.remove(&provider); // old_dummy_tour is completely removed
                    dummy_ids_sorted.remove(dummy_ids_sorted.binary_search(&provider).unwrap());
                } else {
                    vehicles.remove(&provider);
                    tours.remove(&provider); // old_tour is completely removed
                    vehicle_ids_sorted.remove(vehicle_ids_sorted.binary_search(&provider).unwrap());
                }
            }
        }

        // update extended tour of the receiver
        if self.is_dummy(receiver) {
            // dummy_objective_info.insert(receiver, new_tour_receiver.overhead_time());
            dummy_tours.insert(receiver, new_tour_receiver);
        } else {
            // vehicle_objective_info.insert(
            // receiver,
            // ObjectiveInfo::new(self.vehicles.get_vehicle(receiver), &new_tour_receiver),
            // );
            tours.insert(receiver, new_tour_receiver);
        }

        let mut new_dummy_opt = None;
        // insert new dummy tour consisting of conflicting nodes removed from receiver's tour
        if let Some(new_path) = replaced_path {
            let new_dummy = VehicleId::from(format!("dummy{:05}", dummy_counter).as_str());

            new_dummy_opt = Some(new_dummy);

            if self.is_vehicle(receiver) {
                // in this case receiver needs to be removed from the train formations of the
                // removed nodes
                for node in new_path.iter() {
                    if self.network.node(node).is_depot() {
                        continue;
                    }
                    train_formations.insert(
                        node,
                        train_formations
                            .get(&node)
                            .unwrap()
                            .remove(receiver)
                            .unwrap(),
                    );
                }
            }

            let new_dummy_tour = Tour::new_dummy_by_path(new_path, self.network.clone());

            // dummy_objective_info.insert(new_dummy, new_dummy_tour.overhead_time());
            dummy_tours.insert(new_dummy, new_dummy_tour);
            dummy_ids_sorted.insert(
                dummy_ids_sorted
                    .binary_search(&new_dummy)
                    .unwrap_or_else(|e| e),
                new_dummy,
            );
            dummy_counter += 1;
        }

        Ok((
            Schedule {
                vehicles,
                tours,
                train_formations,
                dummy_tours,
                vehicle_ids_sorted,
                dummy_ids_sorted,
                vehicle_counter: dummy_counter,
                config: self.config.clone(),
                vehicle_types: self.vehicle_types.clone(),
                network: self.network.clone(),
            },
            new_dummy_opt,
        ))
    }

    /// Replace a vehicle in the train formation of a node.
    /// Provider or receiver can be a dummy vehicle.
    fn vehicle_replacement_in_train_formation(
        &self,
        provider: VehicleId,
        receiver: VehicleId,
        node: NodeId,
    ) -> Result<TrainFormation, String> {
        let old_formation = self
            .train_formations
            .get(&node)
            .expect(format!("Node {} has no train formations.", node).as_str());

        if self.is_dummy(receiver) {
            if self.is_dummy(provider) {
                Ok(old_formation.clone())
            } else {
                old_formation.remove(provider)
            }
        } else {
            let receiver_vehicle = self.vehicles.get(&receiver).cloned().ok_or(
                format!(
                    "Receiver {} is no vehicle in the current schedule.",
                    receiver
                )
                .as_str(),
            )?;
            if self.is_dummy(provider) {
                Ok(old_formation.add_at_tail(receiver_vehicle))
            } else {
                old_formation.replace(provider, receiver_vehicle)
            }
        }
    }

    fn add_suitable_start_and_end_depot_to_path(
        &self,
        vehicle_type_id: VehicleTypeId,
        mut nodes: Vec<NodeId>,
    ) -> Result<Vec<NodeId>, String> {
        let first_node = *nodes.first().unwrap();
        let last_node = *nodes.last().unwrap();

        // check if depot is available
        if self.network.node(first_node).is_depot()
            && !self.can_depot_spawn_vehicle(first_node, vehicle_type_id)
        {
            return Err(format!(
                "Cannot spawn vehicle of type {} for tour {:?} at start_depot {}. No capacities available.",
                vehicle_type_id,
                nodes,
                first_node,
            ));
        }

        // TODO check if vehicle can be despawned at given end_depot

        // if path does not start with a depot, insert the nearest available start_depot
        if !self.network.node(first_node).is_depot() {
            match self.find_best_start_depot_for_spawning(vehicle_type_id, first_node) {
                Ok(depot) => nodes.insert(0, depot),
                Err(e) => return Err(e),
            };
        }

        // if path does not end with a depot, insert the nearest available end_depot
        if !self.network.node(last_node).is_depot() {
            match self.find_best_end_depot_for_despawning(vehicle_type_id, last_node) {
                Ok(depot) => nodes.push(depot),
                Err(e) => return Err(e),
            };
        }

        Ok(nodes)
    }

    fn find_best_start_depot_for_spawning(
        &self,
        vehicle_type_id: VehicleTypeId,
        first_node: NodeId,
    ) -> Result<NodeId, String> {
        let start_location = self.network.node(first_node).start_location();
        let start_depot = self
            .network
            .start_depots_sorted_by_distance_to(start_location)
            .iter()
            .copied()
            .find(|depot| self.can_depot_spawn_vehicle(*depot, vehicle_type_id));
        match start_depot {
            Some(depot) => Ok(depot),
            None => Err(format!(
                "Cannot spawn vehicle of type {} for start_node {}. No start_depot available.",
                vehicle_type_id, first_node,
            )),
        }
    }

    fn find_best_end_depot_for_despawning(
        &self,
        vehicle_type_id: VehicleTypeId,
        last_node: NodeId,
    ) -> Result<NodeId, String> {
        let end_location = self.network.node(last_node).end_location();
        let end_depot = self
            .network
            .end_depots_sorted_by_distance_from(end_location)
            .first()
            .copied();
        // .find(|depot| self.can_depot_despawn_vehicle(*depot, vehicle_type_id)); // TODO check if depot can de-spawn vehicle
        match end_depot {
            Some(depot) => Ok(depot),
            None => Err(format!(
                "Cannot de-spawn vehicle of type {} for end_node {}. No end_depot available.",
                vehicle_type_id, last_node,
            )),
        }
    }
}
