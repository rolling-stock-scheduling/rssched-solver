use std::fmt;

use sbb_model::base_types::{PassengerCount, TrainLength, VehicleId};

use super::vehicle::Vehicle;
use std::iter::Iterator;

#[derive(Clone)]
pub(crate) struct TrainFormation {
    formation: Vec<Vehicle>, // index 0 is at tail, index len()-1 is front
}

// static functions
impl TrainFormation {
    pub(crate) fn new(formation: Vec<Vehicle>) -> TrainFormation {
        TrainFormation { formation }
    }
}

// methods
impl TrainFormation {
    pub(crate) fn replace(&self, old: VehicleId, new: Vehicle) -> TrainFormation {
        let mut new_formation = self.formation.clone();
        let pos = new_formation
            .iter()
            .position(|u| u.id() == old)
            .expect("vehicle was not part of the TrainFormation and cannot be replaced");

        // replace old by new:
        new_formation.push(new);
        new_formation.swap_remove(pos);

        TrainFormation {
            formation: new_formation,
        }
    }

    pub(crate) fn remove(&self, vehicle: VehicleId) -> TrainFormation {
        let mut new_formation = self.formation.clone();
        let pos = new_formation
            .iter()
            .position(|u| u.id() == vehicle)
            .expect("vehicle was not part of the TrainFormation and cannot be removed");

        // remove vehicle:
        new_formation.remove(pos);

        TrainFormation {
            formation: new_formation,
        }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &Vehicle> + '_ {
        self.formation.iter()
    }

    pub(crate) fn seats(&self) -> PassengerCount {
        self.formation.iter().map(|v| v.seats()).sum()
    }

    pub(crate) fn capacity(&self) -> PassengerCount {
        self.formation.iter().map(|v| v.capacity()).sum()
    }

    pub(crate) fn length(&self) -> TrainLength {
        self.formation.iter().map(|v| v.length()).sum()
    }
}

impl fmt::Display for TrainFormation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.formation.is_empty() {
            for vehicle in self.formation.iter() {
                write!(f, "[{}]->", vehicle)?;
            }
        } else {
            write!(f, "---")?;
        }
        Ok(())
    }
}
