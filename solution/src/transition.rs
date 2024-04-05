use std::collections::HashSet;

use im::HashMap;
use model::base_types::{MaintenanceCounter, VehicleId};

use crate::tour::Tour;

#[derive(Clone)]
pub struct Transition {
    cycles: Vec<TransitionCycle>,
    total_maintenance_violation: MaintenanceCounter,
}

impl Transition {
    pub fn one_cycle_per_vehicle(tours: &HashMap<VehicleId, Tour>) -> Transition {
        let mut total_maintenance_violation = 0;
        let cycles = tours
            .iter()
            .map(|(&vehicle_id, tour)| {
                let maintenance_violation = tour.maintenance_counter().max(0);
                total_maintenance_violation += maintenance_violation;
                TransitionCycle::new(vec![vehicle_id], maintenance_violation)
            })
            .collect();

        Transition {
            cycles,
            total_maintenance_violation,
        }
    }

    // TODO implement greedy

    pub fn maintenance_violation(&self) -> MaintenanceCounter {
        self.total_maintenance_violation
    }

    pub fn verify_consistency(&self, tours: &HashMap<VehicleId, Tour>) {
        // each vehicle is present in exactly one cycle
        let cycles: Vec<VehicleId> = self
            .cycles
            .iter()
            .flat_map(|transition_cycle| transition_cycle.cycle.iter().cloned())
            .collect();
        assert_eq!(cycles.len(), tours.len());
        let vehicles_from_tours: HashSet<VehicleId> = tours.keys().cloned().collect();
        let vehicles_from_cycles: HashSet<VehicleId> = cycles.iter().cloned().collect();
        assert_eq!(vehicles_from_tours, vehicles_from_cycles);

        // verify maintenance violations
        let mut computed_total_maintenance_violation = 0;
        for transition_cycle in self.cycles.iter() {
            let maintenance_counter: MaintenanceCounter = transition_cycle
                .cycle
                .iter()
                .map(|&vehicle_id| tours.get(&vehicle_id).unwrap().maintenance_counter())
                .sum();
            let computed_maintenance_violation = maintenance_counter.max(0);
            assert_eq!(
                computed_maintenance_violation,
                transition_cycle.maintenance_violation
            );
            computed_total_maintenance_violation += computed_maintenance_violation;
        }
        assert_eq!(
            computed_total_maintenance_violation,
            self.total_maintenance_violation
        );
    }
}

#[derive(Debug, Clone)]
pub struct TransitionCycle {
    cycle: Vec<VehicleId>,
    maintenance_violation: MaintenanceCounter,
}

impl TransitionCycle {
    pub fn new(
        cycle: Vec<VehicleId>,
        maintenance_violation: MaintenanceCounter,
    ) -> TransitionCycle {
        TransitionCycle {
            cycle,
            maintenance_violation,
        }
    }
}
