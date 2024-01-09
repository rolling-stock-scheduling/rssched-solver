use std::fmt;

use sbb_model::base_types::{PassengerCount, VehicleId};

use crate::vehicle::Vehicle;
use std::iter::Iterator;

#[derive(Clone)]
pub struct TrainFormation {
    formation: Vec<Vehicle>, // index 0 is at front, index len()-1 is tail
}

// static functions
impl TrainFormation {
    pub(crate) fn empty() -> TrainFormation {
        TrainFormation {
            formation: Vec::new(),
        }
    }
}

// methods
impl TrainFormation {
    pub(crate) fn replace(&self, old: VehicleId, new: Vehicle) -> Result<TrainFormation, String> {
        let mut new_formation = self.formation.clone();
        let pos = new_formation
            .iter()
            .position(|u| u.id() == old)
            .ok_or_else(|| {
                format!(
                    "vehicle {} was not part of the TrainFormation and cannot be replaced",
                    old
                )
            })?;

        // replace old by new:
        new_formation.push(new);
        new_formation.swap_remove(pos);

        Ok(TrainFormation {
            formation: new_formation,
        })
    }

    pub(crate) fn remove(&self, vehicle: VehicleId) -> Result<TrainFormation, String> {
        let mut new_formation = self.formation.clone();
        let pos = new_formation
            .iter()
            .position(|u| u.id() == vehicle)
            .ok_or_else(|| {
                format!(
                    "vehicle {} was not part of the TrainFormation and cannot be removed",
                    vehicle
                )
            })?;

        // remove vehicle:
        new_formation.remove(pos);

        Ok(TrainFormation {
            formation: new_formation,
        })
    }

    pub(crate) fn add_at_tail(&self, vehicle: Vehicle) -> TrainFormation {
        let mut new_formation = self.formation.clone();
        new_formation.push(vehicle);

        TrainFormation {
            formation: new_formation,
        }
    }

    pub(crate) fn seats(&self) -> PassengerCount {
        self.formation.iter().map(|v| v.seats()).sum()
    }

    // pub(crate) fn capacity(&self) -> PassengerCount {
    // self.formation.iter().map(|v| v.capacity()).sum()
    // }

    // pub(crate) fn length(&self) -> TrainLength {
    // self.formation.iter().map(|v| v.length()).sum()
    // }
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
