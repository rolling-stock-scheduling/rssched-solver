mod modifications;
use sbb_model::base_types::Distance;
use sbb_model::base_types::NodeId;
use sbb_model::base_types::PassengerCount;
use sbb_model::base_types::SeatDistance;
use sbb_model::base_types::VehicleId;
use sbb_model::base_types::VehicleTypeId;
use sbb_model::config::Config;
use sbb_model::network::Network;
use sbb_model::vehicle_types::VehicleTypes;

use crate::path::Path;
use crate::path::Segment;
use crate::tour::Tour;
use crate::train_formation::TrainFormation;
use crate::vehicle::Vehicle;

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

    // for each node (except for depots) we store the train formation that covers it.
    // If a node is not covered yet, there is still an entry with an empty train formation.
    // So each non-depot node is covered by exactly one train formation.
    train_formations: HashMap<NodeId, TrainFormation>,

    // not fully covered nodes can be organized to tours, so they can be assigned to vehicles as
    // segments; dummies are never part of a train_formation, they don't have a type and they never
    // include service trips that are fully covered.
    dummy_tours: HashMap<VehicleId, Tour>,

    // counter for vehicle or dummy ids
    vehicle_counter: usize,

    // redundant information for faster access
    vehicle_ids_sorted: Vec<VehicleId>,
    dummy_ids_sorted: Vec<VehicleId>,

    config: Arc<Config>,
    vehicle_types: Arc<VehicleTypes>,
    network: Arc<Network>,
}

// basic methods
impl Schedule {
    pub fn tour_of(&self, vehicle: VehicleId) -> Result<&Tour, String> {
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

    pub fn is_vehicle(&self, vehicle: VehicleId) -> bool {
        self.vehicles.contains_key(&vehicle)
    }

    pub fn train_formation_of(&self, node: NodeId) -> &TrainFormation {
        self.train_formations.get(&node).unwrap()
    }

    pub fn get_vehicle(&self, vehicle: VehicleId) -> Result<&Vehicle, String> {
        self.vehicles
            .get(&vehicle)
            .ok_or_else(|| format!("{} is not an vehicle.", vehicle))
    }

    pub fn number_of_vehicles(&self) -> usize {
        self.vehicles.len()
    }

    pub fn number_of_dummy_tours(&self) -> usize {
        self.dummy_tours.len()
    }

    pub fn dummy_iter(&self) -> impl Iterator<Item = VehicleId> + '_ {
        self.dummy_ids_sorted.iter().copied()
    }

    pub fn vehicles_iter(&self) -> impl Iterator<Item = VehicleId> + '_ {
        self.vehicle_ids_sorted.iter().copied()
    }

    pub fn can_depot_spawn_vehicle(
        &self,
        start_depot: NodeId,
        vehicle_type_id: VehicleTypeId,
    ) -> bool {
        let capacity = self.network.capacity_for(start_depot, vehicle_type_id);

        if capacity == Some(0) {
            return false;
        }

        if capacity == None {
            return true;
        }

        let number_of_spawned_vehicles = self
            .vehicles_iter()
            .filter(|vehicle| {
                self.tour_of(*vehicle)
                    .unwrap()
                    .first_node()
                    .eq(&start_depot)
            })
            .filter(|vehicle| {
                self.get_vehicle(*vehicle)
                    .unwrap()
                    .type_id()
                    .eq(&vehicle_type_id)
            })
            .count() as PassengerCount;

        if number_of_spawned_vehicles < capacity.unwrap() {
            return true;
        }
        false
    }

    // Objective indicators
    pub fn number_of_unserved_passengers(&self) -> PassengerCount {
        self.network
            .service_nodes()
            .map(|node| {
                let demand = self.network.node(node).as_service_trip().demand();
                let served = self.train_formation_of(node).seats();
                if served > demand {
                    0
                } else {
                    demand - served
                }
            })
            .sum()
    }

    pub fn is_fully_covered(&self, service_trip: NodeId) -> bool {
        self.train_formation_of(service_trip).seats()
            >= self.network.node(service_trip).as_service_trip().demand()
    }

    /// sum over all vehicles: number of seats * distance
    pub fn seat_distance_traveled(&self) -> SeatDistance {
        self.tours
            .iter()
            .map(|(vehicle, tour)| {
                tour.service_distance().in_meter() as SeatDistance
                    * self.get_vehicle(*vehicle).unwrap().seats() as SeatDistance
            })
            .sum::<SeatDistance>()
    }

    pub fn print_tours_long(&self) {
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

    pub fn print_tours(&self) {
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

    pub fn print_train_formations(&self) {
        for node in self.network.coverable_nodes() {
            println!("{}: {}", node, self.train_formations.get(&node).unwrap());
        }
    }

    pub fn get_network(&self) -> &Network {
        &self.network
    }

    /// Simulates inserting the node_sequence into the tour of the receiver. Returns all nodes (as a Path) that would
    /// have been removed from the tour. (None if there are no non-depot nodes in conflict)).
    fn conflict(&self, segment: Segment, receiver: VehicleId) -> Option<Path> {
        self.tour_of(receiver).unwrap().conflict(segment)
    }

    // TODO depots of different vehicle types as one depot
    // TODO change depot of a single tour
    // TODO check visibility of different objects and methods
}

// static methods
impl Schedule {
    // initializing an empty schedule
    pub fn empty(
        vehicle_types: Arc<VehicleTypes>,
        network: Arc<Network>,
        config: Arc<Config>,
    ) -> Schedule {
        let vehicles = HashMap::new();
        let tours = HashMap::new();
        let dummy_tours = HashMap::new();
        let mut train_formations = HashMap::new();
        let vehicle_ids_sorted = Vec::new();
        let dummy_ids_sorted = Vec::new();
        let dummy_counter = 0;

        for node in network.coverable_nodes() {
            train_formations.insert(node, TrainFormation::empty());
        }

        Schedule {
            vehicles,
            tours,
            train_formations,
            dummy_tours,
            vehicle_ids_sorted,
            dummy_ids_sorted,
            vehicle_counter: dummy_counter,
            config,
            vehicle_types,
            network,
        }
    }
}

// modifying methods are located in schedule_modifications.rs
