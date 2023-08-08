mod vehicle;

mod tour;
use sbb_model::base_types::DummyId;
use sbb_model::base_types::VehicleId;
use sbb_model::vehicle_types::VehicleTypes;
use tour::Tour;

pub(crate) mod path;

pub(crate) mod objective;
use objective::ObjectiveValue;

pub(crate) mod train_formation;
use train_formation::TrainFormation;

use sbb_model::base_types::NodeId;
use sbb_model::config::Config;
use sbb_model::network::Network;

use im::HashMap;

use std::cmp::Ordering;
use std::sync::Arc;

use self::vehicle::Vehicle;

// TODO: try to use im::Vector instead of Vec and compare performance.

// this represents a solution to the rolling stock problem.
// It should be an immutable object. So whenever a modification is applied a copy of the
// schedule is create.
#[derive(Clone)]
pub(crate) struct Schedule {
    // all vehicles that are used in the schedule
    vehicles: HashMap<VehicleId, Vehicle>,

    // the tours assigned to vehicles
    tours: HashMap<VehicleId, Tour>,

    // not fully covered nodes can be organized to tours, so they can be assigned to vehicles as
    // segments; dummies are never part of a train_formation, they don't have a type and they never
    // include service trips that are fully covered.
    dummy_tours: HashMap<DummyId, Tour>,

    // for each node (except for depots) we store the train formation that covers it.
    covered_by: HashMap<NodeId, TrainFormation>,

    // redundant information for faster access
    vehicle_ids_sorted: Vec<VehicleId>,
    dummy_ids_sorted: Vec<DummyId>,
    dummy_counter: usize,

    /*
    vehicle_objective_info: HashMap<VehicleId, ObjectiveInfo>, // for each vehicle we store the overhead_time and the dead_head_distance
    dummy_objective_info: HashMap<VehicleId, Duration>, // for each dummy we store the overhead_time
    */
    objective_value: ObjectiveValue,
    config: Arc<Config>,
    vehicle_types: Arc<VehicleTypes>,
    network: Arc<Network>,
}

// methods
impl Schedule {
    pub(crate) fn tour_of(&self, vehicle: VehicleId) -> &Tour {
        &self.tours.get(&vehicle).unwrap_or_else(|| {
            panic!(
                "{} is neither no valid vehicle. So there is no tour.",
                vehicle
            )
        })
    }

    pub(crate) fn covered_by(&self, node: NodeId) -> &TrainFormation {
        self.covered_by.get(&node).unwrap()
    }

    pub(crate) fn get_vehicle(&self, vehicle: VehicleId) -> &Vehicle {
        self.vehicles
            .get(&vehicle)
            .expect(&format!("{} is not an vehicle.", vehicle))
    }

    pub(crate) fn number_of_dummy_tours(&self) -> usize {
        self.dummy_tours.len()
    }

    pub(crate) fn objective_value(&self) -> ObjectiveValue {
        self.objective_value
    }

    pub(crate) fn dummy_iter(&self) -> impl Iterator<Item = DummyId> + '_ {
        self.dummy_ids_sorted.iter().copied()
    }

    pub(crate) fn vehicles_iter(&self) -> impl Iterator<Item = VehicleId> + '_ {
        self.vehicle_ids_sorted.iter().copied()
    }

    pub(crate) fn print_long(&self) {
        println!(
            "** schedule with {} tours and {} dummy-tours:",
            self.tours.len(),
            self.dummy_tours.len()
        );
        for vehicle in self.vehicles_iter() {
            print!("     {}: ", self.get_vehicle(vehicle));
            self.tours.get(&vehicle).unwrap().print();
        }
        for dummy in self.dummy_iter() {
            print!("     {}: ", dummy);
            self.dummy_tours.get(&dummy).unwrap().print();
        }
    }

    pub(crate) fn print(&self) {
        for vehicle in self.vehicles_iter() {
            println!("{}: {}", vehicle, self.tours.get(&vehicle).unwrap());
        }
        for dummy in self.dummy_iter() {
            println!("{}: {}", dummy, self.dummy_tours.get(&dummy).unwrap());
        }
    }
}
