use im::HashMap;
use model::{
    base_types::{MaintenanceCounter, VehicleIdx, INF_DISTANCE},
    network::Network,
};

use crate::tour::Tour;

#[derive(Debug, Clone)]
pub struct TransitionCycle {
    cycle: Vec<VehicleIdx>,
    maintenance_counter: MaintenanceCounter,
}

impl TransitionCycle {
    pub fn new(cycle: Vec<VehicleIdx>, maintenance_counter: MaintenanceCounter) -> TransitionCycle {
        TransitionCycle {
            cycle,
            maintenance_counter,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = VehicleIdx> + '_ {
        self.cycle.iter().copied()
    }

    pub fn get_vec(&self) -> &Vec<VehicleIdx> {
        &self.cycle
    }

    pub fn len(&self) -> usize {
        self.cycle.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cycle.is_empty()
    }

    pub fn first(&self) -> Option<VehicleIdx> {
        self.cycle.first().copied()
    }

    pub fn last(&self) -> Option<VehicleIdx> {
        self.cycle.last().copied()
    }

    pub fn get(&self, idx: usize) -> Option<VehicleIdx> {
        self.cycle.get(idx).copied()
    }

    pub fn maintenance_counter(&self) -> MaintenanceCounter {
        self.maintenance_counter
    }

    pub fn three_opt(
        &self,
        i: usize,
        j: usize,
        k: usize,
        tours: &HashMap<VehicleIdx, Tour>,
        network: &Network,
    ) -> TransitionCycle {
        let mut maintenance_counter = self.maintenance_counter;
        let n = self.cycle.len();

        let end_depot_i = tours.get(&self.cycle[i]).unwrap().end_depot().unwrap();
        let start_depot_i_plus_1 = tours
            .get(&self.cycle[(i + 1) % n])
            .unwrap()
            .start_depot()
            .unwrap();

        let end_depot_j = tours.get(&self.cycle[j]).unwrap().end_depot().unwrap();
        let start_depot_j_plus_1 = tours
            .get(&self.cycle[(j + 1) % n])
            .unwrap()
            .start_depot()
            .unwrap();
        let end_depot_k = tours.get(&self.cycle[k]).unwrap().end_depot().unwrap();
        let start_depot_k_plus_1 = tours
            .get(&self.cycle[(k + 1) % n])
            .unwrap()
            .start_depot()
            .unwrap();

        // Remove counter of depot trips between (i, i+1), (j, j+1), and (k, k+1)
        maintenance_counter -= network
            .dead_head_distance_between(end_depot_i, start_depot_i_plus_1)
            .in_meter()
            .unwrap_or(INF_DISTANCE) as MaintenanceCounter;
        maintenance_counter -= network
            .dead_head_distance_between(end_depot_j, start_depot_j_plus_1)
            .in_meter()
            .unwrap_or(INF_DISTANCE) as MaintenanceCounter;
        maintenance_counter -= network
            .dead_head_distance_between(end_depot_k, start_depot_k_plus_1)
            .in_meter()
            .unwrap_or(INF_DISTANCE) as MaintenanceCounter;

        // Add counter of depot trips between (i, j+1), (j, k+1), and (k, i+1)
        maintenance_counter += network
            .dead_head_distance_between(end_depot_i, start_depot_j_plus_1)
            .in_meter()
            .unwrap_or(INF_DISTANCE) as MaintenanceCounter;
        maintenance_counter += network
            .dead_head_distance_between(end_depot_j, start_depot_k_plus_1)
            .in_meter()
            .unwrap_or(INF_DISTANCE) as MaintenanceCounter;
        maintenance_counter += network
            .dead_head_distance_between(end_depot_k, start_depot_i_plus_1)
            .in_meter()
            .unwrap_or(INF_DISTANCE) as MaintenanceCounter;

        // Perform the swap
        let mut new_cycle = Vec::with_capacity(n);
        new_cycle.extend(&self.cycle[..i + 1]);
        new_cycle.extend(&self.cycle[j + 1..k + 1]);
        new_cycle.extend(&self.cycle[i + 1..j + 1]);
        new_cycle.extend(&self.cycle[k + 1..]);

        TransitionCycle::new(new_cycle, maintenance_counter)
    }
}

impl std::fmt::Display for TransitionCycle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Cycle: ({}), counter: {}",
            self.cycle
                .iter()
                .map(|&idx| format!("{}", idx.idx()))
                .collect::<Vec<String>>()
                .join(", "),
            self.maintenance_counter
        )
    }
}
