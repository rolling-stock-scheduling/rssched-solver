mod modifications;
#[cfg(test)]
mod tests;

use itertools::Itertools;
use model::base_types::Cost;
use model::base_types::DepotIdx;
use model::base_types::Distance;
use model::base_types::MaintenanceCounter;
use model::base_types::NodeIdx;
use model::base_types::PassengerCount;
use model::base_types::VehicleCount;
use model::base_types::VehicleIdx;
use model::base_types::VehicleTypeIdx;
use model::network::nodes::Node;
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

type DepotUsage = HashMap<(DepotIdx, VehicleTypeIdx), (HashSet<VehicleIdx>, HashSet<VehicleIdx>)>;

// this represents a solution to the rolling stock problem.
// It should be an immutable object. So whenever a modification is applied a copy of the
// schedule is create.
#[derive(Clone)]
pub struct Schedule {
    // all vehicles (non-dummy) that are used in the schedule
    vehicles: HashMap<VehicleIdx, Vehicle>,

    // the tours assigned to vehicles
    tours: HashMap<VehicleIdx, Tour>,

    // A schedule is thought to be repeated each period but vehicles may take a different tour on
    // the next period. The information which vehicles of the first execution of the schedule
    // becomes which vehicle in the next period is stored in the next_period_transition (grouped by
    // type).
    next_period_transitions: HashMap<VehicleTypeIdx, Transition>,

    // for each node (except for depots) we store the train formation that covers it.
    // If a node is not covered yet, there is still an entry with an empty train formation.
    // So each non-depot node is covered by exactly one train formation.
    train_formations: HashMap<NodeIdx, TrainFormation>,

    // for each depot-vehicle_type-pair we store the vehicles of that type that are spawned at this depot and the vehicles that
    // despawn at this depot.
    // First hashset are the spawned vehicles, second hashset are the despawned vehicles.
    depot_usage: DepotUsage,

    // not fully covered nodes can be organized to tours, so they can be assigned to vehicles as
    // segments; dummies are never part of a train_formation, they don't have a type and they never
    // include service trips that are fully covered.
    dummy_tours: HashMap<VehicleIdx, Tour>,

    // counter for vehicle or dummy ids
    vehicle_counter: usize,

    // redundant information for faster access
    vehicle_ids_grouped_and_sorted: HashMap<VehicleTypeIdx, Vec<VehicleIdx>>,
    dummy_ids_sorted: Vec<VehicleIdx>,
    unserved_passengers: (PassengerCount, PassengerCount),
    maintenance_violation: MaintenanceCounter,
    costs: Cost,

    network: Arc<Network>,
}

// basic methods
impl Schedule {
    pub fn number_of_vehicles(&self) -> usize {
        self.vehicles.len()
    }

    pub fn vehicles_iter(
        &self,
        vehicle_type: VehicleTypeIdx,
    ) -> impl Iterator<Item = VehicleIdx> + '_ {
        self.vehicle_ids_grouped_and_sorted[&vehicle_type]
            .iter()
            .copied()
    }

    pub fn vehicles_iter_all(&self) -> impl Iterator<Item = VehicleIdx> + '_ {
        let vehicle_types: Vec<_> = self.network.vehicle_types().iter().collect();
        vehicle_types
            .into_iter()
            .flat_map(|vt| self.vehicles_iter(vt))
    }

    pub fn is_vehicle(&self, vehicle: VehicleIdx) -> bool {
        self.vehicles.contains_key(&vehicle)
    }

    pub fn get_vehicle(&self, vehicle: VehicleIdx) -> Result<&Vehicle, String> {
        self.vehicles
            .get(&vehicle)
            .ok_or_else(|| format!("{} is not an vehicle.", vehicle))
    }

    pub fn vehicle_type_of(&self, vehicle: VehicleIdx) -> Result<VehicleTypeIdx, String> {
        Ok(self.get_vehicle(vehicle)?.type_idx())
    }

    pub fn is_dummy(&self, vehicle: VehicleIdx) -> bool {
        self.dummy_tours.contains_key(&vehicle)
    }

    pub fn is_vehicle_or_dummy(&self, vehicle: VehicleIdx) -> bool {
        self.is_vehicle(vehicle) || self.is_dummy(vehicle)
    }

    pub fn number_of_dummy_tours(&self) -> usize {
        self.dummy_tours.len()
    }

    pub fn dummy_iter(&self) -> impl Iterator<Item = VehicleIdx> + '_ {
        self.dummy_ids_sorted.iter().copied()
    }

    pub fn tour_of(&self, vehicle: VehicleIdx) -> Result<&Tour, String> {
        match self.tours.get(&vehicle) {
            Some(tour) => Ok(tour),
            None => self.dummy_tours.get(&vehicle).ok_or(format!(
                "{} is neither vehicle nor a dummy. So there is no tour.",
                vehicle
            )),
        }
    }

    pub fn get_tours(&self) -> &HashMap<VehicleIdx, Tour> {
        &self.tours
    }

    pub fn maintenance_violation(&self) -> MaintenanceCounter {
        self.maintenance_violation
    }

    pub fn next_day_transition_of(&self, vehicle_type: VehicleTypeIdx) -> &Transition {
        self.next_period_transitions.get(&vehicle_type).unwrap()
    }

    pub fn set_next_day_transitions(
        &self,
        transitions: HashMap<VehicleTypeIdx, Transition>,
    ) -> Self {
        let mut new_schedule = self.clone();
        new_schedule.next_period_transitions = transitions;
        new_schedule
    }

    pub fn train_formation_of(&self, node: NodeIdx) -> &TrainFormation {
        self.train_formations.get(&node).unwrap()
    }

    /// Returns the number of vehicles of the given type that are spawned at the given depot
    pub fn number_of_vehicles_of_same_type_spawned_at(
        &self,
        depot: DepotIdx,
        vehicle_type: VehicleTypeIdx,
    ) -> VehicleCount {
        self.number_of_vehicles_of_same_type_spawned_at_custom_usage(
            depot,
            vehicle_type,
            &self.depot_usage,
        )
    }

    pub fn number_of_vehicles_spawned_at(&self, depot: DepotIdx) -> VehicleCount {
        self.number_of_vehicles_spawned_at_custom_usage(depot, &self.depot_usage)
    }

    /// Returns the number of vehicles of the given type that are spawned at the given depot - the
    /// number of vehicles of the given type that despawn at the given depot.
    /// Hence, negative values mean that there are more vehicles despawning than spawning.
    pub fn depot_balance(&self, depot: DepotIdx, vehicle_type: VehicleTypeIdx) -> i32 {
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
        start_depot: NodeIdx,
        vehicle_type: VehicleTypeIdx,
    ) -> bool {
        self.can_depot_spawn_vehicle_custom_usage(start_depot, vehicle_type, &self.depot_usage)
    }

    pub fn reduces_spawning_at_depot_violation(
        &self,
        vehicle_type: VehicleTypeIdx,
        depot: DepotIdx,
    ) -> bool {
        self.depot_balance(depot, vehicle_type) < 0
    }

    pub fn reduces_despawning_at_depot_violation(
        &self,
        vehicle_type: VehicleTypeIdx,
        depot: DepotIdx,
    ) -> bool {
        self.depot_balance(depot, vehicle_type) > 0
    }

    /// Returns the number of passengers that do not fit (first entry) or seated passenger that
    /// cannot sit (second entry) at the given node.
    pub fn unserved_passengers_at(&self, node: NodeIdx) -> (PassengerCount, PassengerCount) {
        Schedule::compute_unserved_passengers_at_node(
            &self.network,
            node,
            self.train_formations.get(&node).unwrap(),
        )
    }

    /// Returns the number of passengers that do not fit (first entry) or seated passenger that
    /// cannot sit (second entry).
    pub fn unserved_passengers(&self) -> (PassengerCount, PassengerCount) {
        self.unserved_passengers
    }

    pub fn is_fully_covered(&self, service_trip: NodeIdx) -> bool {
        self.unserved_passengers_at(service_trip) == (0, 0)
    }

    pub fn costs(&self) -> Cost {
        self.costs
    }

    pub fn print_tours_long(&self) {
        println!(
            "** schedule with {} tours and {} dummy-tours:",
            self.tours.len(),
            self.dummy_tours.len()
        );
        for vehicle in self.vehicles_iter_all() {
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
        for vehicle in self.vehicles_iter_all() {
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

    pub fn print_next_day_transitions(&self) {
        for (vehicle_type, transition) in self.next_period_transitions.iter() {
            println!(
                "\nNextDayTransitions for {}:",
                self.network.vehicle_types().get(*vehicle_type).unwrap()
            );
            transition.print();
        }
    }

    pub fn get_network(&self) -> Arc<Network> {
        self.network.clone()
    }

    pub fn get_vehicle_types(&self) -> Arc<VehicleTypes> {
        self.network.vehicle_types()
    }

    /// As hitch_hikers we main vehicles on a service trip that are not needed.
    /// In other words one of the vehicle just uses this service trip to avoid a dead_head_trip
    /// (normally service trips are cheapter than dead_head_trips as staff must only be payed once).
    /* pub fn count_hitch_hikers(&self) -> VehicleCount {
        self.train_formations
            .iter()
            .filter(|(&node, _)| self.network.node(node).is_service())
            .map(|(&service_node, train_formation)| {
                let vehicle_type = self.network.vehicle_type_for(service_node);
                let vehicle_count = train_formation.vehicle_count();
                let required_vehicle = self
                    .network
                    .number_of_vehicles_required_to_serve(vehicle_type, service_node);

                if vehicle_count < required_vehicle {
                    println!(
                        "Node {} requires {} vehicles but only {} are available",
                        service_node, required_vehicle, vehicle_count
                    );
                    println!("  Vehicles: {:?}", train_formation.ids());
                }
                vehicle_count - required_vehicle
            })
            .sum()
    } */

    pub fn verify_consistency(&self) {
        // check vehicles
        for (id, vehicle) in self.vehicles.iter() {
            assert_eq!(*id, vehicle.idx());
            assert_eq!(self.vehicle_type_of(*id).unwrap(), vehicle.type_idx());
        }

        // check vehicles id sets are equal
        let vehicles: HashSet<VehicleIdx> = self.vehicles.keys().cloned().collect();
        let vehicles_from_tours: HashSet<VehicleIdx> = self.tours.keys().cloned().collect();
        let vehicles_from_train_formations: HashSet<VehicleIdx> = self
            .train_formations
            .values()
            .flat_map(|train_formation| train_formation.ids())
            .collect();
        let vehicles_from_depot_usage: HashSet<VehicleIdx> = self
            .depot_usage
            .values()
            .flat_map(|(spawned, despawned)| spawned.iter().chain(despawned.iter()))
            .cloned()
            .collect();
        let vehicles_from_sorted: HashSet<VehicleIdx> = self.vehicles_iter_all().collect();

        assert_eq!(vehicles, vehicles_from_tours);
        assert_eq!(vehicles, vehicles_from_sorted);
        assert_eq!(vehicles, vehicles_from_train_formations); // we do not allow tours to not cover
                                                              // anything, so each vehicle must be
                                                              // in at least one formation
        assert_eq!(vehicles, vehicles_from_depot_usage);

        // check if vehicles are sorted
        for vehicle_type in self.network.vehicle_types().iter() {
            for (vehicle1, vehicle2) in self.vehicle_ids_grouped_and_sorted[&vehicle_type]
                .iter()
                .tuple_windows()
            {
                assert!(vehicle1 < vehicle2);
            }
        }

        // check dummy tours
        let dummy_vehicles: HashSet<VehicleIdx> = self.dummy_tours.keys().cloned().collect();
        let dummy_vehicles_from_sorted: HashSet<VehicleIdx> =
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

            // check that vehicle type of service trips matches vehicle type of vehicle
            for node in tour.all_non_depot_nodes_iter() {
                if self.network.node(node).is_service() {
                    assert_eq!(
                        self.vehicle_type_of(*vehicle).unwrap(),
                        self.network.vehicle_type_for(node)
                    );
                }
            }

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

            // check that train_formations do not violate maximal formation count
            for node in tour.all_non_depot_nodes_iter() {
                let maximal_formation_count_opt = match self.network.node(node) {
                    Node::Service(_) => self.network.maximal_formation_count_for(node),
                    Node::Maintenance(_) => {
                        Some(self.network.track_count_of_maintenance_slot(node))
                    }
                    _ => None,
                };
                let train_formation = self.train_formations.get(&node).unwrap();
                if let Some(maximal_formation_count) = maximal_formation_count_opt {
                    assert!(train_formation.vehicle_count() <= maximal_formation_count,);
                }
            }

            // check depots usage
            let vehicle_type = self.vehicle_type_of(*vehicle).unwrap();

            let start_depot = self.network.get_depot_id(tour.start_depot().unwrap());
            let (spawned, _) = self.depot_usage.get(&(start_depot, vehicle_type)).unwrap();
            assert!(spawned.contains(vehicle));

            let end_depot = self.network.get_depot_id(tour.end_depot().unwrap());
            let (_, despawned) = self.depot_usage.get(&(end_depot, vehicle_type)).unwrap();
            assert!(despawned.contains(vehicle));

            tour.verify_consistency();
        }

        // check next_period_transition
        self.next_period_transitions
            .iter()
            .for_each(|(vt, transition)| {
                let tours_of_type = self
                    .vehicles_iter(*vt)
                    .map(|vehicle| (vehicle, self.tour_of(vehicle).unwrap().clone()))
                    .collect();
                transition.verify_consistency(&tours_of_type, &self.network);
            });

        // check unserved passengers
        assert_eq!(
            self.unserved_passengers,
            self.network
                .all_service_nodes()
                .map(|node| self.unserved_passengers_at(node))
                .fold(
                    (0, 0),
                    |(unserved, seated), (unserved_node, seated_node)| {
                        (unserved + unserved_node, seated + seated_node)
                    }
                )
        );

        // check maintenance violation
        assert_eq!(
            self.maintenance_violation,
            self.next_period_transitions
                .values()
                .map(|transition| transition.maintenance_violation())
                .sum::<MaintenanceCounter>()
        );

        // check costs
        assert_eq!(
            self.costs,
            self.tours.values().map(|tour| tour.costs()).sum::<Cost>()
                + self.network.number_of_service_nodes() as Cost
                    * self.network.config().costs.staff
        );

        // check that all tours are in the depot_usage
        for (depot, vehicle_type) in self.depot_usage.keys() {
            let (spawned, despawned) = self.depot_usage.get(&(*depot, *vehicle_type)).unwrap();
            for vehicle in spawned.iter() {
                assert!(self.vehicles.contains_key(vehicle));
                assert_eq!(self.vehicle_type_of(*vehicle).unwrap(), *vehicle_type);
                assert_eq!(
                    self.network
                        .get_depot_id(self.tour_of(*vehicle).unwrap().start_depot().unwrap()),
                    *depot
                );
            }
            for vehicle in despawned.iter() {
                assert!(self.vehicles.contains_key(vehicle));
                assert_eq!(self.vehicle_type_of(*vehicle).unwrap(), *vehicle_type);
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
                let vehicle_id = vehicle.idx();
                assert_eq!(
                    self.vehicles.get(&vehicle_id).unwrap().type_idx(),
                    vehicle.type_idx()
                );
                assert!(self
                    .tour_of(vehicle_id)
                    .unwrap()
                    .all_nodes_iter()
                    .contains(&node));
            }

            // check that no vehicle appears twice in a train_formation
            let vehicle_ids_as_set: HashSet<VehicleIdx> = HashSet::from(train_formation.ids());
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
            assert!(total_spawned <= total_capacity,);
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
                    .vehicles_iter_all()
                    .zip(other.vehicles_iter_all())
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

        let next_period_transitions = network
            .vehicle_types()
            .iter()
            .map(|vt| (vt, Transition::new_fast(&[], &HashMap::new(), &network)))
            .collect();

        let vehicle_ids_grouped_and_sorted = network
            .vehicle_types()
            .iter()
            .map(|vt| (vt, Vec::new()))
            .collect();

        let costs = network.number_of_service_nodes() as Cost * network.config().costs.staff;

        let unserved_passengers =
            Schedule::compute_unserved_passengers(&network, &train_formations);

        Schedule::new(
            HashMap::new(),
            HashMap::new(),
            next_period_transitions,
            train_formations,
            HashMap::new(),
            HashMap::new(),
            0,
            vehicle_ids_grouped_and_sorted,
            Vec::new(),
            unserved_passengers,
            0,
            costs,
            network,
        )
    }

    /// initializing a schedule with a given list of tours (in Vec<NodeId> form) for each vehicle
    /// type.
    /// If a tour violates the depot constraints an message is printed and another depot is used
    /// instead.
    pub fn from_tours(
        tours: StdHashMap<VehicleTypeIdx, Vec<Vec<NodeIdx>>>,
        network: Arc<Network>,
    ) -> Result<Schedule, String> {
        let mut schedule = Schedule::empty(network);

        for (vehicle_type, tours) in tours {
            for tour in tours {
                let result = schedule.spawn_vehicle_for_path(vehicle_type, tour);

                schedule = result.unwrap().0;
            }
        }

        Ok(schedule)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        vehicles: HashMap<VehicleIdx, Vehicle>,
        tours: HashMap<VehicleIdx, Tour>,
        next_period_transitions: HashMap<VehicleTypeIdx, Transition>,
        train_formations: HashMap<NodeIdx, TrainFormation>,
        depot_usage: DepotUsage,
        dummy_tours: HashMap<VehicleIdx, Tour>,
        vehicle_counter: usize,
        vehicle_ids_grouped_and_sorted: HashMap<VehicleTypeIdx, Vec<VehicleIdx>>,
        dummy_ids_sorted: Vec<VehicleIdx>,
        unserved_passengers: (PassengerCount, PassengerCount),
        maintenance_violation: MaintenanceCounter,
        costs: Cost,
        network: Arc<Network>,
    ) -> Schedule {
        let schedule = Schedule {
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
            network,
        };

        // schedule.verify_consistency(); // For bug testing

        schedule
    }
}

// private methods
impl Schedule {
    fn can_depot_spawn_vehicle_custom_usage(
        &self,
        start_depot: NodeIdx,
        vehicle_type: VehicleTypeIdx,
        depot_usage: &DepotUsage,
    ) -> bool {
        let depot = self.network.get_depot_id(start_depot);
        let capacity_for_type = self.network.capacity_of(depot, vehicle_type);

        if capacity_for_type == 0 {
            return false;
        }

        if self.number_of_vehicles_of_same_type_spawned_at_custom_usage(
            depot,
            vehicle_type,
            depot_usage,
        ) >= capacity_for_type
        {
            return false;
        }

        if self.number_of_vehicles_spawned_at_custom_usage(depot, depot_usage)
            >= self.network.total_capacity_of(depot)
        {
            return false;
        }

        true
    }

    fn number_of_vehicles_of_same_type_spawned_at_custom_usage(
        &self,
        depot: DepotIdx,
        vehicle_type: VehicleTypeIdx,
        depot_usage: &DepotUsage,
    ) -> VehicleCount {
        depot_usage
            .get(&(depot, vehicle_type))
            .map(|(spawned, _)| spawned.len())
            .unwrap_or(0) as VehicleCount
    }

    fn number_of_vehicles_spawned_at_custom_usage(
        &self,
        depot: DepotIdx,
        depot_usage: &DepotUsage,
    ) -> VehicleCount {
        self.network
            .vehicle_types()
            .iter()
            .map(|vt| {
                depot_usage
                    .get(&(depot, vt))
                    .map(|(spawned, _)| spawned.len() as VehicleCount)
                    .unwrap_or(0)
            })
            .sum()
    }

    fn compute_unserved_passengers_at_node(
        network: &Network,
        node: NodeIdx,
        train_formation: &TrainFormation,
    ) -> (PassengerCount, PassengerCount) {
        let demand = network.passengers_of(node);
        let capacity = train_formation.capacity();

        let seat_demand = network.seated_passengers_of(node);
        let seat_capacity = train_formation.seats();
        (
            demand.saturating_sub(capacity), // return 0 if demand < capacity
            seat_demand.saturating_sub(seat_capacity), // return 0 if seat_demand < seat_capacity
        )
    }

    fn compute_unserved_passengers(
        network: &Network,
        train_formations: &HashMap<NodeIdx, TrainFormation>,
    ) -> (PassengerCount, PassengerCount) {
        network
            .all_service_nodes()
            .map(|node| {
                Schedule::compute_unserved_passengers_at_node(
                    network,
                    node,
                    train_formations.get(&node).unwrap(),
                )
            })
            .fold(
                (0, 0),
                |(unserved, seated), (unserved_node, seated_node)| {
                    (unserved + unserved_node, seated + seated_node)
                },
            )
    }
}
// modifying methods are located in schedule_modifications.rs
