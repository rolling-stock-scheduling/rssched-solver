mod modifications;
#[cfg(test)]
mod tests;

use itertools::Itertools;
use model::base_types::Cost;
use model::base_types::DepotId;
use model::base_types::Distance;
use model::base_types::MaintenanceCounter;
use model::base_types::NodeId;
use model::base_types::PassengerCount;
use model::base_types::VehicleCount;
use model::base_types::VehicleId;
use model::base_types::VehicleTypeId;
use model::network::Network;
use model::vehicle_types::VehicleTypes;

use crate::tour::Tour;
use crate::train_formation::TrainFormation;
use crate::transition::Transition;
use crate::vehicle::Vehicle;

use im::HashMap;
use im::HashSet;
use std::cmp::Ordering;
use std::collections::HashMap as StdHashMap;
use std::sync::Arc;

type DepotUsage = HashMap<(DepotId, VehicleTypeId), (HashSet<VehicleId>, HashSet<VehicleId>)>;

// this represents a solution to the rolling stock problem.
// It should be an immutable object. So whenever a modification is applied a copy of the
// schedule is create.
#[derive(Clone)]
pub struct Schedule {
    // all vehicles (non-dummy) that are used in the schedule
    vehicles: HashMap<VehicleId, Vehicle>,

    // the tours assigned to vehicles
    tours: HashMap<VehicleId, Tour>,

    // A schedule is thought to be repeated each period but vehicles may take a different tour on
    // the next period. The information which vehicles of the first execution of the schedule
    // becomes which vehicle in the next period is stored in the next_period_transition.
    next_period_transition: Transition,

    // for each node (except for depots) we store the train formation that covers it.
    // If a node is not covered yet, there is still an entry with an empty train formation.
    // So each non-depot node is covered by exactly one train formation.
    train_formations: HashMap<NodeId, TrainFormation>,

    // for each depot-vehicle_type-pair we store the vehicles of that type that are spawned at this depot and the vehicles that
    // despawn at this depot.
    // First hashset are the spawned vehicles, second hashset are the despawned vehicles.
    depot_usage: DepotUsage,

    // not fully covered nodes can be organized to tours, so they can be assigned to vehicles as
    // segments; dummies are never part of a train_formation, they don't have a type and they never
    // include service trips that are fully covered.
    dummy_tours: HashMap<VehicleId, Tour>,

    // counter for vehicle or dummy ids
    vehicle_counter: usize,

    // redundant information for faster access
    vehicle_ids_sorted: Vec<VehicleId>,
    dummy_ids_sorted: Vec<VehicleId>,

    network: Arc<Network>,
}

// basic methods
impl Schedule {
    pub fn number_of_vehicles(&self) -> usize {
        self.vehicles.len()
    }

    pub fn vehicles_iter(&self) -> impl Iterator<Item = VehicleId> + '_ {
        self.vehicle_ids_sorted.iter().copied()
    }

    pub fn is_vehicle(&self, vehicle: VehicleId) -> bool {
        self.vehicles.contains_key(&vehicle)
    }

    pub fn get_vehicle(&self, vehicle: VehicleId) -> Result<&Vehicle, String> {
        self.vehicles
            .get(&vehicle)
            .ok_or_else(|| format!("{} is not an vehicle.", vehicle))
    }

    pub fn vehicle_type_of(&self, vehicle: VehicleId) -> VehicleTypeId {
        self.get_vehicle(vehicle).unwrap().type_id()
    }

    pub fn is_dummy(&self, vehicle: VehicleId) -> bool {
        self.dummy_tours.contains_key(&vehicle)
    }

    pub fn is_vehicle_or_dummy(&self, vehicle: VehicleId) -> bool {
        self.is_vehicle(vehicle) || self.is_dummy(vehicle)
    }

    pub fn number_of_dummy_tours(&self) -> usize {
        self.dummy_tours.len()
    }

    pub fn dummy_iter(&self) -> impl Iterator<Item = VehicleId> + '_ {
        self.dummy_ids_sorted.iter().copied()
    }

    pub fn tour_of(&self, vehicle: VehicleId) -> Result<&Tour, String> {
        match self.tours.get(&vehicle) {
            Some(tour) => Ok(tour),
            None => self.dummy_tours.get(&vehicle).ok_or(format!(
                "{} is neither vehicle nor a dummy. So there is no tour.",
                vehicle
            )),
        }
    }

    pub fn maintenance_violation(&self) -> MaintenanceCounter {
        self.next_period_transition.maintenance_violation()
    }

    pub fn train_formation_of(&self, node: NodeId) -> &TrainFormation {
        self.train_formations.get(&node).unwrap()
    }

    /// Returns the number of vehicles of the given type that are spawned at the given depot
    pub fn number_of_vehicles_of_same_type_spawned_at(
        &self,
        depot: DepotId,
        vehicle_type: VehicleTypeId,
    ) -> VehicleCount {
        self.depot_usage
            .get(&(depot, vehicle_type))
            .map(|(spawned, _)| spawned.len())
            .unwrap_or(0) as VehicleCount
    }

    pub fn number_of_vehicles_spawned_at(&self, depot: DepotId) -> VehicleCount {
        self.network
            .vehicle_types()
            .iter()
            .map(|vt| {
                self.depot_usage
                    .get(&(depot, vt))
                    .map(|(spawned, _)| spawned.len() as VehicleCount)
                    .unwrap_or(0)
            })
            .sum()
    }

    /// Returns the number of vehicles of the given type that are spawned at the given depot - the
    /// number of vehicles of the given type that despawn at the given depot.
    /// Hence, negative values mean that there are more vehicles despawning than spawning.
    pub fn depot_balance(&self, depot: DepotId, vehicle_type: VehicleTypeId) -> i32 {
        self.depot_usage
            .get(&(depot, vehicle_type))
            .map(|(spawned, despawned)| (spawned.len() as i32 - despawned.len() as i32))
            .unwrap_or(0)
    }

    pub fn total_depot_balance_violation(&self) -> VehicleCount {
        self.depot_usage
            .keys()
            .map(|(depot, vehicle_type)| {
                self.depot_balance(*depot, *vehicle_type).unsigned_abs() as VehicleCount
            })
            .sum()
    }

    pub fn can_depot_spawn_vehicle(
        &self,
        start_depot: NodeId,
        vehicle_type: VehicleTypeId,
    ) -> bool {
        let depot = self.network.get_depot_id(start_depot);
        let capacity_for_type = self.network.capacity_of(depot, vehicle_type);

        if capacity_for_type == 0 {
            return false;
        }

        if self.number_of_vehicles_of_same_type_spawned_at(depot, vehicle_type) >= capacity_for_type
        {
            return false;
        }

        if self.number_of_vehicles_spawned_at(depot) >= self.network.total_capacity_of(depot) {
            return false;
        }

        true
    }

    pub fn reduces_spawning_at_depot_violation(
        &self,
        vehicle_type: VehicleTypeId,
        depot: DepotId,
    ) -> bool {
        self.depot_balance(depot, vehicle_type) < 0
    }

    pub fn reduces_despawning_at_depot_violation(
        &self,
        vehicle_type: VehicleTypeId,
        depot: DepotId,
    ) -> bool {
        self.depot_balance(depot, vehicle_type) > 0
    }

    /// Returns the number of passengers that do not fit (first entry) or seated passenger that
    /// cannot sit (second entry) at the given node.
    pub fn unserved_passengers_at(&self, node: NodeId) -> (PassengerCount, PassengerCount) {
        let demand = self.network.passengers_of(node);
        let capacity = self.train_formation_of(node).capacity();

        let seat_demand = self.network.seated_passengers_of(node);
        let seat_capacity = self.train_formation_of(node).seats();
        (
            demand.saturating_sub(capacity), // return 0 if demand < capacity
            seat_demand.saturating_sub(seat_capacity), // return 0 if seat_demand < seat_capacity
        )
    }

    /// Returns the number of passengers that do not fit (first entry) or seated passenger that
    /// cannot sit (second entry).
    pub fn unserved_passengers(&self) -> (PassengerCount, PassengerCount) {
        self.network
            .all_service_nodes()
            .map(|node| self.unserved_passengers_at(node))
            .fold(
                (0, 0),
                |(unserved_demand, unserved_seated),
                 (unserved_demand_at_node, unserved_seated_at_node)| {
                    (
                        unserved_demand + unserved_demand_at_node,
                        unserved_seated + unserved_seated_at_node,
                    )
                },
            )
    }

    pub fn is_fully_covered(&self, service_trip: NodeId) -> bool {
        self.unserved_passengers_at(service_trip) == (0, 0)
    }

    pub fn costs(&self) -> Cost {
        self.tours.values().map(|tour| tour.costs()).sum::<Cost>()
            + self.network.number_of_service_nodes() as Cost * self.network.config().costs.staff
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

    pub fn total_dead_head_distance(&self) -> Distance {
        self.tours
            .values()
            .map(|tour| tour.dead_head_distance())
            .sum()
    }

    pub fn print_tours(&self) {
        for vehicle in self.vehicles_iter() {
            println!(
                "{}: {}",
                self.get_vehicle(vehicle).unwrap(),
                self.tours.get(&vehicle).unwrap()
            );
        }
        for dummy in self.dummy_iter() {
            println!("{}: {}", dummy, self.dummy_tours.get(&dummy).unwrap());
        }
    }

    pub fn print_depot_balances(&self) {
        for depot in self.network.depots_iter() {
            for vehicle_type in self.network.vehicle_types().iter() {
                println!(
                    "  depot {}, vehicle type {}: {}",
                    depot,
                    vehicle_type,
                    self.depot_balance(depot, vehicle_type)
                );
            }
        }
        println!(
            "  total depot balance violation: {}",
            self.total_depot_balance_violation()
        );
    }

    pub fn print_train_formations(&self) {
        for node in self.network.coverable_nodes() {
            println!("{}: {}", node, self.train_formations.get(&node).unwrap());
        }
    }

    pub fn get_network(&self) -> &Network {
        &self.network
    }

    pub fn get_vehicle_types(&self) -> Arc<VehicleTypes> {
        self.network.vehicle_types()
    }

    pub fn verify_consistency(&self) {
        // check vehicles
        for (id, vehicle) in self.vehicles.iter() {
            assert_eq!(*id, vehicle.id());
            assert_eq!(self.vehicle_type_of(*id), vehicle.type_id());
        }

        // check vehicles id sets are equal
        let vehicles: HashSet<VehicleId> = self.vehicles.keys().cloned().collect();
        let vehicles_from_tours: HashSet<VehicleId> = self.tours.keys().cloned().collect();
        let vehicles_from_train_formations: HashSet<VehicleId> = self
            .train_formations
            .values()
            .flat_map(|train_formation| train_formation.ids())
            .collect();
        let vehicles_from_depot_usage: HashSet<VehicleId> = self
            .depot_usage
            .values()
            .flat_map(|(spawned, despawned)| spawned.iter().chain(despawned.iter()))
            .cloned()
            .collect();
        let vehicles_from_sorted: HashSet<VehicleId> =
            self.vehicle_ids_sorted.iter().cloned().collect();

        assert_eq!(vehicles, vehicles_from_tours);
        assert_eq!(vehicles, vehicles_from_sorted);
        assert_eq!(vehicles, vehicles_from_train_formations); // we do not allow tours to not cover
                                                              // anything, so each vehicle must be
                                                              // in at least one formation
        assert_eq!(vehicles, vehicles_from_depot_usage);

        // check if vehicles are sorted
        for (vehicle1, vehicle2) in self.vehicle_ids_sorted.iter().tuple_windows() {
            assert!(vehicle1 < vehicle2);
        }

        // check dummy tours
        let dummy_vehicles: HashSet<VehicleId> = self.dummy_tours.keys().cloned().collect();
        let dummy_vehicles_from_sorted: HashSet<VehicleId> =
            self.dummy_ids_sorted.iter().cloned().collect();

        assert_eq!(dummy_vehicles, dummy_vehicles_from_sorted);
        // check if dummy tours are sorted
        for (dummy1, dummy2) in self.dummy_ids_sorted.iter().tuple_windows() {
            assert!(dummy1 < dummy2);
        }

        // check tours
        for vehicle in self.vehicles.keys() {
            let tour = self.tours.get(vehicle).unwrap();

            assert!(!tour.is_dummy());

            // check that all nodes are covered by a train_formation
            for node in tour.all_non_depot_nodes_iter() {
                assert!(self.train_formations.contains_key(&node));
                assert!(self
                    .train_formations
                    .get(&node)
                    .unwrap()
                    .ids()
                    .contains(vehicle));
            }

            // check depots usage
            let vehicle_type = self.vehicle_type_of(*vehicle);

            let start_depot = self.network.get_depot_id(tour.start_depot().unwrap());
            let (spawned, _) = self.depot_usage.get(&(start_depot, vehicle_type)).unwrap();
            assert!(spawned.contains(vehicle));

            let end_depot = self.network.get_depot_id(tour.end_depot().unwrap());
            let (_, despawned) = self.depot_usage.get(&(end_depot, vehicle_type)).unwrap();
            assert!(despawned.contains(vehicle));

            tour.verify_consistency();
        }

        // check next_period_transition
        self.next_period_transition.verify_consistency(
            &self.tours,
            self.network.config().maintenance.maximal_distance,
        );

        // check that all tours are in the depot_usage
        for (depot, vehicle_type) in self.depot_usage.keys() {
            let (spawned, despawned) = self.depot_usage.get(&(*depot, *vehicle_type)).unwrap();
            for vehicle in spawned.iter() {
                assert!(self.vehicles.contains_key(vehicle));
                assert_eq!(self.vehicle_type_of(*vehicle), *vehicle_type);
                assert_eq!(
                    self.network
                        .get_depot_id(self.tour_of(*vehicle).unwrap().start_depot().unwrap()),
                    *depot
                );
            }
            for vehicle in despawned.iter() {
                assert!(self.vehicles.contains_key(vehicle));
                assert_eq!(self.vehicle_type_of(*vehicle), *vehicle_type);
                assert_eq!(
                    self.network
                        .get_depot_id(self.tour_of(*vehicle).unwrap().end_depot().unwrap()),
                    *depot
                );
            }
        }

        // check train_formations
        for node in self.network.coverable_nodes() {
            let train_formation = self.train_formations.get(&node).unwrap();
            for vehicle in train_formation.iter() {
                let vehicle_id = vehicle.id();
                assert_eq!(
                    self.vehicles.get(&vehicle_id).unwrap().type_id(),
                    vehicle.type_id()
                );
                assert!(self
                    .tour_of(vehicle_id)
                    .unwrap()
                    .all_nodes_iter()
                    .contains(&node));
            }

            // check that no vehicle appears twice in a train_formation
            let vehicle_ids_as_set: HashSet<VehicleId> = HashSet::from(train_formation.ids());
            assert_eq!(vehicle_ids_as_set.len(), train_formation.ids().len());
        }

        // check if depot spawning limits are respected
        for (depot, vehicle_type) in self.depot_usage.keys().cloned() {
            let number_of_spawned_vehicles =
                self.number_of_vehicles_of_same_type_spawned_at(depot, vehicle_type);
            let capacity = self.network.capacity_of(depot, vehicle_type);
            assert!(number_of_spawned_vehicles <= capacity);
        }

        for depot in self.network.depots_iter() {
            let total_capacity = self.network.total_capacity_of(depot);
            let total_spawned = self.number_of_vehicles_spawned_at(depot);
            assert!(total_spawned <= total_capacity);
        }

        println!("Debug only: Schedule is consistent");
    }
}

impl Ord for Schedule {
    // First compare the number of vehicles.
    // Then compare the tours of the vehicles. (By the order given by the vehicle ids).
    // If all tours are equal, compare the number of dummy tours.
    // Finally, compare the dummy tours. (From small to long).
    //
    // I.e. two schedules are different if they have the same tours (real and dummy) but the
    // vehicle_ids are ordered differently.
    // However, two schedules are equal if they have the same tours (real and dummy) and only the
    // dummy_tours differ in the order.

    fn cmp(&self, other: &Self) -> Ordering {
        self.number_of_vehicles()
            .cmp(&other.number_of_vehicles())
            .then(
                match self
                    .vehicles_iter()
                    .zip(other.vehicles_iter())
                    .map(|(vehicle, other_vehicle)| {
                        self.tour_of(vehicle)
                            .unwrap()
                            .cmp(other.tour_of(other_vehicle).unwrap())
                    })
                    .find(|ord| *ord != Ordering::Equal)
                {
                    Some(other) => other,
                    None => {
                        // finally compare dummy_tours. For this first sort the dummy tours and
                        // then compare from small to long.
                        let mut dummy_tours: Vec<_> = self.dummy_tours.values().collect();
                        dummy_tours.sort();
                        let mut other_dummy_tours: Vec<_> = other.dummy_tours.values().collect();
                        other_dummy_tours.sort();
                        match dummy_tours
                            .iter()
                            .zip(other_dummy_tours.iter())
                            .map(|(&tour, &other_tour)| tour.cmp(other_tour))
                            .find(|ord| *ord != Ordering::Equal)
                        {
                            Some(other) => other,
                            None => Ordering::Equal,
                        }
                    }
                },
            )
    }
}

impl PartialOrd for Schedule {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Schedule {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Eq for Schedule {}

// static methods
impl Schedule {
    /// initializing an empty schedule
    pub fn empty(network: Arc<Network>) -> Schedule {
        let mut train_formations = HashMap::new();
        for node in network.coverable_nodes() {
            train_formations.insert(node, TrainFormation::empty());
        }

        let next_period_transition = Transition::one_cycle_per_vehicle(
            &HashMap::new(),
            network.config().maintenance.maximal_distance,
        );

        Schedule {
            vehicles: HashMap::new(),
            tours: HashMap::new(),
            next_period_transition,
            train_formations,
            depot_usage: HashMap::new(),
            dummy_tours: HashMap::new(),
            vehicle_ids_sorted: Vec::new(),
            dummy_ids_sorted: Vec::new(),
            vehicle_counter: 0,
            network,
        }
    }

    /// initializing a schedule with a given list of tours (in Vec<NodeId> form) for each vehicle
    /// type
    pub fn from_tours(
        tours: StdHashMap<VehicleTypeId, Vec<Vec<NodeId>>>,
        network: Arc<Network>,
    ) -> Result<Schedule, String> {
        let mut schedule = Schedule::empty(network);

        for (vehicle_type, tours) in tours {
            for tour in tours {
                let result = schedule.spawn_vehicle_for_path(vehicle_type, tour)?;
                schedule = result.0;
            }
        }

        Ok(schedule)
    }
}

// modifying methods are located in schedule_modifications.rs
