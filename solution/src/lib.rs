pub mod json_serialisation;
mod path;
mod tour;
mod train_formation;
mod vehicle;

use sbb_model::base_types::Distance;
use sbb_model::base_types::NodeId;
use sbb_model::base_types::VehicleId;
use sbb_model::config::Config;
use sbb_model::network::Network;
use sbb_model::vehicle_types::VehicleTypes;

use path::Path;
use path::Segment;
use tour::Tour;
use train_formation::TrainFormation;
use vehicle::Vehicle;

use im::HashMap;
use std::sync::Arc;

// TODO: try to use im::Vector instead of Vec and compare performance.

// this represents a solution to the rolling stock problem.
// It should be an immutable object. So whenever a modification is applied a copy of the
// schedule is create.
#[derive(Clone)]
pub struct Schedule {
    // all vehicles (non-dummy) that are used in the schedule
    vehicles: HashMap<VehicleId, Vehicle>,

    // the tours assigned to vehicles
    tours: HashMap<VehicleId, Tour>,

    // not fully covered nodes can be organized to tours, so they can be assigned to vehicles as
    // segments; dummies are never part of a train_formation, they don't have a type and they never
    // include service trips that are fully covered.
    dummy_tours: HashMap<VehicleId, Tour>,

    // for each node (except for depots) we store the train formation that covers it.
    covered_by: HashMap<NodeId, TrainFormation>,

    // redundant information for faster access
    vehicle_ids_sorted: Vec<VehicleId>,
    dummy_ids_sorted: Vec<VehicleId>,
    dummy_counter: usize,

    config: Arc<Config>,
    vehicle_types: Arc<VehicleTypes>,
    network: Arc<Network>,
}

// basic methods
impl Schedule {
    fn tour_of(&self, vehicle: VehicleId) -> Result<&Tour, String> {
        match self.tours.get(&vehicle) {
            Some(tour) => Ok(tour),
            None => self.dummy_tours.get(&vehicle).ok_or(format!(
                "{} is neither vehicle nor a dummy. So there is no tour.",
                vehicle
            )),
        }
    }

    pub fn is_dummy(&self, vehicle: VehicleId) -> bool {
        self.dummy_tours.contains_key(&vehicle)
    }

    fn is_vehicle(&self, vehicle: VehicleId) -> bool {
        self.vehicles.contains_key(&vehicle)
    }

    fn covered_by(&self, node: NodeId) -> &TrainFormation {
        self.covered_by.get(&node).unwrap()
    }

    pub fn get_vehicle(&self, vehicle: VehicleId) -> Result<&Vehicle, String> {
        self.vehicles
            .get(&vehicle)
            .ok_or_else(|| format!("{} is not an vehicle.", vehicle))
    }

    pub fn number_of_dummy_tours(&self) -> usize {
        self.dummy_tours.len()
    }

    // pub(crate) fn objective_value(&self) -> ObjectiveValue {
    // self.objective_value
    // }

    fn dummy_iter(&self) -> impl Iterator<Item = VehicleId> + '_ {
        self.dummy_ids_sorted.iter().copied()
    }

    fn vehicles_iter(&self) -> impl Iterator<Item = VehicleId> + '_ {
        self.vehicle_ids_sorted.iter().copied()
    }

    pub fn print_long(&self) {
        println!(
            "** schedule with {} tours and {} dummy-tours:",
            self.tours.len(),
            self.dummy_tours.len()
        );
        for vehicle in self.vehicles_iter() {
            print!("     {}: ", self.get_vehicle(vehicle).unwrap());
            self.tours.get(&vehicle).unwrap().print();
        }
        for dummy in self.dummy_iter() {
            print!("     {}: ", dummy);
            self.dummy_tours.get(&dummy).unwrap().print();
        }
    }

    pub fn print(&self) {
        for vehicle in self.vehicles_iter() {
            println!("{}: {}", vehicle, self.tours.get(&vehicle).unwrap());
        }
        for dummy in self.dummy_iter() {
            println!("{}: {}", dummy, self.dummy_tours.get(&dummy).unwrap());
        }
    }

    pub fn total_dead_head_distance(&self) -> Distance {
        self.tours
            .values()
            .map(|tour| tour.dead_head_distance())
            .sum()
    }

    pub fn get_network(&self) -> &Network {
        &self.network
    }
}

// modification methods
impl Schedule {
    /// Simulates inserting the node_sequence into the tour of vehicle. Return all nodes (as a Path) that would
    /// have been removed from the tour (None if there are no non-depot nodes in conflict)).
    fn conflict(&self, segment: Segment, receiver: VehicleId) -> Option<Path> {
        self.tour_of(receiver).unwrap().conflict(segment)
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
        let mut covered_by = self.covered_by.clone();
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
                match new_tour_receiver.earliest_not_reaching_node(sub_segment_start) {
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

        // update covered_by:
        for node in moved_nodes.iter() {
            let new_formation = covered_by
                .get(node)
                .unwrap()
                .replace(provider, self.vehicles.get(&receiver).cloned().unwrap());
            covered_by.insert(*node, new_formation);
        }

        Ok(Schedule {
            vehicles: self.vehicles.clone(),
            tours,
            dummy_tours,
            covered_by,
            vehicle_ids_sorted,
            dummy_ids_sorted,
            dummy_counter: self.dummy_counter,
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
        let mut covered_by = self.covered_by.clone();
        let mut vehicle_ids_sorted = self.vehicle_ids_sorted.clone();
        let mut dummy_ids_sorted = self.dummy_ids_sorted.clone();
        let mut dummy_counter = self.dummy_counter;
        // let mut vehicle_objective_info = self.vehicle_objective_info.clone();
        // let mut dummy_objective_info = self.dummy_objective_info.clone();

        let tour_provider = self.tour_of(provider)?;
        let tour_receiver = self.tour_of(receiver)?;

        // remove segment for provider
        let (shrinked_tour_provider, path) = tour_provider.remove(segment)?;

        // update covered_by:
        for node in path.iter() {
            let new_formation = covered_by
                .get(&node)
                .unwrap()
                .replace(provider, self.vehicles.get(&receiver).cloned().unwrap());
            covered_by.insert(node, new_formation);
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
                    let new_formation = covered_by.get(&node).unwrap().remove(receiver);
                    covered_by.insert(node, new_formation);
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
                dummy_tours,
                covered_by,
                vehicle_ids_sorted,
                dummy_ids_sorted,
                dummy_counter,
                config: self.config.clone(),
                vehicle_types: self.vehicle_types.clone(),
                network: self.network.clone(),
            },
            new_dummy_opt,
        ))
    }

    // TODO depots of different vehicle types as one depot
    // TODO change depot of a single tour
    // TODO modular objective
    // TODO check visibility of different objects and methods
}
