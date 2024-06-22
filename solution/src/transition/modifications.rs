use im::HashMap;
use model::{
    base_types::{MaintenanceCounter, NodeIdx, VehicleIdx, INF_DISTANCE},
    network::Network,
};

use crate::tour::Tour;

use super::{CycleIdx, Transition, TransitionCycle};

impl Transition {
    // TEST this function (with multiple vehicles being updated in a raw)
    // old_tours are the tours of the old schedule, updated_tours are the tours of vehicle that
    // have already been updated in the current transition. Hence, for determining the tours of the
    // predecessor and the successor first take the updated_tours and if they are not present, then
    // take the old_tours.
    pub fn update_vehicle(
        &self,
        vehicle: VehicleIdx,
        new_tour: &Tour,
        updated_tours: &HashMap<VehicleIdx, &Tour>,
        old_tours: &HashMap<VehicleIdx, Tour>,
        network: &Network,
    ) -> Transition {
        let old_tour = old_tours.get(&vehicle).unwrap();
        let mut cycles = self.cycles.clone();
        let cycle_idx = self.cycle_lookup.get(&vehicle).unwrap();
        let old_cycle = &mut cycles.get(*cycle_idx).unwrap();

        let new_maintenance_counter = if old_cycle.len() == 1 {
            new_tour.maintenance_counter()
                + network
                    .dead_head_distance_between(
                        new_tour.end_depot().unwrap(),
                        new_tour.start_depot().unwrap(),
                    )
                    .in_meter()
                    .unwrap_or(INF_DISTANCE) as MaintenanceCounter
        } else {
            let (end_depot_of_predecessor, start_depot_of_successor) = self
                .end_depot_of_predecessor_and_start_depot_of_successor(
                    vehicle,
                    updated_tours,
                    old_tours,
                );

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

            old_cycle.maintenance_counter() - maintenance_counter_for_removal
                + maintenance_counter_for_addition
        };

        let new_cycle = TransitionCycle::new(old_cycle.get_vec().clone(), new_maintenance_counter);

        let total_maintenance_violation = (self.total_maintenance_violation
            + new_maintenance_counter.max(0))
            - old_cycle.maintenance_counter().max(0);

        let total_maintenance_counter = (self.total_maintenance_counter + new_maintenance_counter)
            - old_cycle.maintenance_counter();

        cycles[*cycle_idx] = new_cycle;

        Transition {
            cycles,
            total_maintenance_violation,
            total_maintenance_counter,
            cycle_lookup: self.cycle_lookup.clone(),
            empty_cycles: self.empty_cycles.clone(),
        }
    }

    // TEST this function
    pub fn add_vehicle_to_own_cycle(
        &self,
        vehicle: VehicleIdx,
        new_tour: &Tour,
        network: &Network,
    ) -> Transition {
        let mut cycles = self.cycles.clone();
        let mut cycle_lookup = self.cycle_lookup.clone();
        let mut empty_cycles = self.empty_cycles.clone();

        let maintenance_counter_of_tour = new_tour.maintenance_counter()
            + network
                .dead_head_distance_between(
                    new_tour.end_depot().unwrap(),
                    new_tour.start_depot().unwrap(),
                )
                .in_meter()
                .unwrap_or(INF_DISTANCE) as MaintenanceCounter;

        let new_cycle = TransitionCycle::new(vec![vehicle], maintenance_counter_of_tour);
        let total_maintenance_violation =
            self.total_maintenance_violation + maintenance_counter_of_tour.max(0);
        let total_maintenance_counter =
            self.total_maintenance_counter + maintenance_counter_of_tour;

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
            total_maintenance_counter,
            cycle_lookup,
            empty_cycles,
        }
    }

    // TEST this function (with multiple vehicles being removed in a raw) and with a
    // maintenance vehicle removed
    pub fn remove_vehicle(
        &self,
        vehicle: VehicleIdx,
        updated_tours: &HashMap<VehicleIdx, &Tour>,
        old_tours: &HashMap<VehicleIdx, Tour>,
        network: &Network,
    ) -> Transition {
        let mut cycles = self.cycles.clone();
        let mut cycle_lookup = self.cycle_lookup.clone();
        let mut empty_cycles = self.empty_cycles.clone();

        let cycle_idx = cycle_lookup.get(&vehicle).unwrap();
        let old_cycle = cycles.get(*cycle_idx).unwrap();
        let new_cycle_vec: Vec<_> = old_cycle.iter().filter(|&v| v != vehicle).collect();
        let new_maintenance_counter = if new_cycle_vec.is_empty() {
            empty_cycles.push(*cycle_idx);
            0
        } else {
            let (end_depot_of_predecessor, start_depot_of_successor) = self
                .end_depot_of_predecessor_and_start_depot_of_successor(
                    vehicle,
                    updated_tours,
                    old_tours,
                );

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
                .unwrap_or(INF_DISTANCE)
                as MaintenanceCounter;

            old_cycle.maintenance_counter() - maintenance_counter_for_removal
                + maintenance_counter_for_addition
        };
        let new_cycle = TransitionCycle::new(new_cycle_vec, new_maintenance_counter);

        let total_maintenance_violation = (self.total_maintenance_violation
            + new_maintenance_counter.max(0))
            - old_cycle.maintenance_counter().max(0);

        let total_maintenance_counter = (self.total_maintenance_counter + new_maintenance_counter)
            - old_cycle.maintenance_counter();

        cycles[*cycle_idx] = new_cycle;

        cycle_lookup.remove(&vehicle);

        Transition {
            cycles,
            total_maintenance_violation,
            total_maintenance_counter,
            cycle_lookup,
            empty_cycles,
        }
    }

    // TEST this function with non-maintenance vehicle and maintenance vehicle
    pub fn add_vehicle_at_the_end(
        &self,
        vehicle: VehicleIdx,
        new_cycle_idx: CycleIdx,
        updated_tours: &HashMap<VehicleIdx, &Tour>,
        old_tours: &HashMap<VehicleIdx, Tour>,
        network: &Network,
    ) -> Transition {
        let mut cycles = self.cycles.clone();
        let mut cycle_lookup = self.cycle_lookup.clone();
        let mut empty_cycles = self.empty_cycles.clone();

        let old_cycle = cycles.get(new_cycle_idx).unwrap();

        let new_cycle_vec: Vec<_> = old_cycle.iter().chain(std::iter::once(vehicle)).collect();

        let tour_of_vehicle = updated_tours
            .get(&vehicle)
            .copied()
            .unwrap_or_else(|| old_tours.get(&vehicle).unwrap());

        let new_maintenance_counter = if new_cycle_vec.len() == 1 {
            empty_cycles.retain(|&idx| idx != new_cycle_idx);
            tour_of_vehicle.maintenance_counter()
                + network
                    .dead_head_distance_between(
                        tour_of_vehicle.end_depot().unwrap(),
                        tour_of_vehicle.start_depot().unwrap(),
                    )
                    .in_meter()
                    .unwrap_or(INF_DISTANCE) as MaintenanceCounter
        } else {
            let end_depot_of_predecessor = new_cycle_vec
                .get(new_cycle_vec.len() - 2)
                .map(|&v| {
                    updated_tours
                        .get(&v)
                        .copied()
                        .unwrap_or_else(|| old_tours.get(&v).unwrap())
                        .end_depot()
                        .unwrap()
                })
                .unwrap();

            let start_depot_of_successor = new_cycle_vec
                .first()
                .map(|&v| {
                    updated_tours
                        .get(&v)
                        .copied()
                        .unwrap_or_else(|| old_tours.get(&v).unwrap())
                        .start_depot()
                        .unwrap()
                })
                .unwrap();

            let maintenance_counter_for_removal = network
                .dead_head_distance_between(end_depot_of_predecessor, start_depot_of_successor)
                .in_meter()
                .unwrap_or(INF_DISTANCE)
                as MaintenanceCounter;

            let maintenance_counter_for_addtion = self
                .maintenance_counter_of_tour_plus_dead_head_trips_before_and_after(
                    tour_of_vehicle,
                    end_depot_of_predecessor,
                    start_depot_of_successor,
                    network,
                );

            old_cycle.maintenance_counter() - maintenance_counter_for_removal
                + maintenance_counter_for_addtion
        };

        let new_cycle = TransitionCycle::new(new_cycle_vec, new_maintenance_counter);

        let total_maintenance_violation = (self.total_maintenance_violation
            + new_maintenance_counter.max(0))
            - old_cycle.maintenance_counter().max(0);

        let total_maintenance_counter = (self.total_maintenance_counter + new_maintenance_counter)
            - old_cycle.maintenance_counter();

        cycles[new_cycle_idx] = new_cycle;

        cycle_lookup.insert(vehicle, new_cycle_idx);

        Transition {
            cycles,
            total_maintenance_violation,
            total_maintenance_counter,
            cycle_lookup,
            empty_cycles: self.empty_cycles.clone(),
        }
    }

    pub fn move_vehicle(
        &self,
        vehicle: VehicleIdx,
        new_cycle_idx: CycleIdx,
        tours: &HashMap<VehicleIdx, Tour>,
        network: &Network,
    ) -> Transition {
        let transition = self.remove_vehicle(vehicle, &HashMap::new(), tours, network);
        transition.add_vehicle_at_the_end(vehicle, new_cycle_idx, &HashMap::new(), tours, network)
    }

    pub fn replace_cycle(&self, cycle_idx: CycleIdx, new_cycle: TransitionCycle) -> Transition {
        let mut cycles = self.cycles.clone();

        let old_cycle = self.cycles.get(cycle_idx).unwrap();

        let total_maintenance_violation = self.total_maintenance_violation
            + new_cycle.maintenance_counter().max(0)
            - old_cycle.maintenance_counter().max(0);

        let total_maintenance_counter = self.total_maintenance_counter
            + new_cycle.maintenance_counter()
            - old_cycle.maintenance_counter();

        cycles[cycle_idx] = new_cycle;

        Transition {
            cycles,
            total_maintenance_violation,
            total_maintenance_counter,
            cycle_lookup: self.cycle_lookup.clone(),
            empty_cycles: self.empty_cycles.clone(),
        }
    }
}

impl Transition {
    // as we do multiple transition updates for the same old_schedule, the tours must be updated
    // one-by-one. To speed up the process, have the old_tours of the old_schedule and the tour
    // updates are contained in a separate hashmap. So as tours consider first the tours in
    // updated_tours and if they are not present, then consider the tours in old_tours.
    fn end_depot_of_predecessor_and_start_depot_of_successor(
        &self,
        vehicle: VehicleIdx,
        updated_tours: &HashMap<VehicleIdx, &Tour>,
        old_tours: &HashMap<VehicleIdx, Tour>,
    ) -> (NodeIdx, NodeIdx) {
        let cycle_idx = self.cycle_lookup.get(&vehicle).unwrap();

        let vehicle_idx = self.cycles[*cycle_idx]
            .iter()
            .position(|v| v == vehicle)
            .unwrap();
        let predecessor = if vehicle_idx == 0 {
            self.cycles[*cycle_idx].last().unwrap()
        } else {
            self.cycles[*cycle_idx].get(vehicle_idx - 1).unwrap()
        };

        let successor = if vehicle_idx == self.cycles[*cycle_idx].len() - 1 {
            self.cycles[*cycle_idx].first().unwrap()
        } else {
            self.cycles[*cycle_idx].get(vehicle_idx + 1).unwrap()
        };

        (
            updated_tours
                .get(&predecessor)
                .copied()
                .unwrap_or_else(|| old_tours.get(&predecessor).unwrap())
                .end_depot()
                .unwrap(),
            updated_tours
                .get(&successor)
                .copied()
                .unwrap_or_else(|| old_tours.get(&successor).unwrap())
                .start_depot()
                .unwrap(),
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
                .unwrap_or(INF_DISTANCE) as MaintenanceCounter
            + network
                .dead_head_distance_between(tour.end_depot().unwrap(), start_depot_of_successor)
                .in_meter()
                .unwrap_or(INF_DISTANCE) as MaintenanceCounter
    }
}
