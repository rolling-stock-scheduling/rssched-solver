use sbb_model::base_types::{NodeId, VehicleId, VehicleTypeId};

use crate::{
    path::{Path, Segment},
    tour::Tour,
    vehicle::Vehicle,
    Schedule,
};

impl Schedule {
    /// Spawn vehicle from nearest depot.
    /// If path does not start with a depot the vehicle is spawned from the nearest availabe depot
    /// (from the start location of the first trip).
    /// Similarly, if path does not end with a depot the vehicle is spawned to the nearest depot
    /// (from the end location of the last trip).
    /// If no depot is available, an error is returned.
    /// If the depot given in the path in not available, an error is returned.
    ///
    pub fn spawn_vehicle_for_tour(
        &self,
        vehicle_type_id: VehicleTypeId,
        nodes: Vec<NodeId>,
    ) -> Result<Schedule, String> {
        let mut vehicles = self.vehicles.clone();
        let mut tours = self.tours.clone();
        let mut train_formations = self.train_formations.clone();
        let mut vehicle_ids_sorted = self.vehicle_ids_sorted.clone();

        let vehicle_id = VehicleId::from(format!("vehicle{:05}", self.vehicle_counter).as_str());

        if !self.network.node(*nodes.first().unwrap()).is_depot() {
            // TODO spawn vehicle from nearest depot by prepending it to the path.
        }

        if !self.network.node(*nodes.last().unwrap()).is_depot() {
            // TODO de-spawn vehicle at nearest depot by appening it to the path.
        }

        let tour = Tour::new(nodes, self.network.clone())?;

        // update vehicles
        let vehicle = Vehicle::new(vehicle_id, vehicle_type_id, self.vehicle_types.clone());
        vehicles.insert(vehicle_id, vehicle.clone());

        // update train_formations
        for node in tour.all_nodes_iter() {
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
            let new_formation = train_formations.get(&node).unwrap().remove(vehicle);
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
                    // dummy_objective_info.insert(provider, new_tour_provider.overhead_time());
                    dummy_tours.insert(provider, new_tour);
                } else {
                    // vehicle_objective_info.insert(
                    // provider,
                    // ObjectiveInfo::new(self.vehicles.get_vehicle(provider), &new_tour_provider),
                    // );
                    tours.insert(provider, new_tour);
                }
            }
            None => {
                if self.is_dummy(provider) {
                    dummy_tours.remove(&provider); // old_dummy_tour is completely removed
                    dummy_ids_sorted.remove(dummy_ids_sorted.binary_search(&provider).unwrap());
                // dummy_objective_info.remove(&provider);
                } else {
                    tours.remove(&provider); // old_tour is completely removed
                    vehicle_ids_sorted.remove(vehicle_ids_sorted.binary_search(&provider).unwrap());
                    // vehicle_objective_info.remove(&provider);
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
            let new_formation = train_formations
                .get(node)
                .unwrap()
                .replace(provider, self.vehicles.get(&receiver).cloned().unwrap());
            train_formations.insert(*node, new_formation);
        }

        Ok(Schedule {
            vehicles: self.vehicles.clone(),
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
            let new_formation = train_formations
                .get(&node)
                .unwrap()
                .replace(provider, self.vehicles.get(&receiver).cloned().unwrap());
            train_formations.insert(node, new_formation);
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
                // dummy_objective_info.remove(&provider);
                } else {
                    tours.remove(&provider); // old_tour is completely removed
                    vehicle_ids_sorted.remove(vehicle_ids_sorted.binary_search(&provider).unwrap());
                    // vehicle_objective_info.remove(&provider);
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
                    let new_formation = train_formations.get(&node).unwrap().remove(receiver);
                    train_formations.insert(node, new_formation);
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
                vehicles: self.vehicles.clone(),
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
}
