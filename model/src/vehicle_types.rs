use std::{collections::HashMap, sync::Arc};

use crate::base_types::{PassengerCount, VehicleCount, VehicleTypeId};

pub struct VehicleTypes {
    vehicle_types: HashMap<VehicleTypeId, Arc<VehicleType>>,
    ids_sorted: Vec<VehicleTypeId>, // sorted by seat count, then capacity, then length, then id
}

impl VehicleTypes {
    pub fn new(vehicle_types_vec: Vec<VehicleType>) -> VehicleTypes {
        let vehicle_types: HashMap<_, _> = vehicle_types_vec
            .into_iter()
            .map(|vt| (vt.id, Arc::new(vt)))
            .collect();

        /* let mut ids_sorted_by_seat_count: Vec<_> = vehicle_types.keys().cloned().collect();
        ids_sorted_by_seat_count.sort_by_key(|&id| {
            let vt = vehicle_types.get(&id).unwrap();
            (vt.seats(), vt.capacity(), vt.maximal_formation_count(), id)
        }); */
        let mut ids_sorted_by_id: Vec<_> = vehicle_types.keys().cloned().collect();
        ids_sorted_by_id.sort();

        VehicleTypes {
            vehicle_types,
            ids_sorted: ids_sorted_by_id,
        }
    }

    pub fn get(&self, id: VehicleTypeId) -> Option<Arc<VehicleType>> {
        self.vehicle_types.get(&id).cloned()
    }

    /// Returns an iterator over all vehicle types, sorted by seat count.
    pub fn iter(&self) -> impl Iterator<Item = VehicleTypeId> + '_ {
        self.ids_sorted.iter().cloned()
    }

    /// Returns best vehicle_type for demand.
    /// Take vehicle_type with the least number of seats such that all passengers are covered.
    /// if no vehicle_type can cover the demand take biggest vehicle (last in sorted list).
    pub fn best_for(&self, demand: PassengerCount) -> VehicleTypeId {
        *self
            .ids_sorted
            .iter()
            .find(|vt| self.vehicle_types[vt].seats() >= demand)
            .unwrap_or(self.ids_sorted.last().unwrap())
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct VehicleType {
    id: VehicleTypeId,
    name: String,
    seats: PassengerCount,
    capacity: PassengerCount,
    maximal_formation_count: Option<VehicleCount>,
}

impl VehicleType {
    pub fn new(
        id: VehicleTypeId,
        name: String,
        capacity_of_passengers: PassengerCount,
        number_of_seats: PassengerCount,
        maximal_formation_count: Option<VehicleCount>,
    ) -> VehicleType {
        VehicleType {
            id,
            name,
            seats: number_of_seats,
            capacity: capacity_of_passengers,
            maximal_formation_count,
        }
    }

    pub fn id(&self) -> VehicleTypeId {
        self.id
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn seats(&self) -> PassengerCount {
        self.seats
    }

    pub fn capacity(&self) -> PassengerCount {
        self.capacity
    }

    pub fn maximal_formation_count(&self) -> Option<VehicleCount> {
        self.maximal_formation_count
    }
}
