// © 2023-2024 ETH Zurich
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// any later version.
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use std::fmt;

use model::base_types::{PassengerCount, VehicleCount, VehicleIdx};

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
    pub(crate) fn replace(&self, old: VehicleIdx, new: Vehicle) -> Result<TrainFormation, String> {
        let mut new_formation = self.formation.clone();
        let pos = new_formation
            .iter()
            .position(|u| u.idx() == old)
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

    pub(crate) fn remove(&self, vehicle: VehicleIdx) -> Result<TrainFormation, String> {
        let mut new_formation = self.formation.clone();
        let pos = new_formation
            .iter()
            .position(|u| u.idx() == vehicle)
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

    pub fn ids(&self) -> Vec<VehicleIdx> {
        self.formation.iter().map(|v| v.idx()).collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Vehicle> {
        self.formation.iter()
    }

    pub fn capacity(&self) -> PassengerCount {
        self.formation.iter().map(|v| v.capacity()).sum()
    }

    pub fn seats(&self) -> PassengerCount {
        self.formation.iter().map(|v| v.seats()).sum()
    }

    pub fn vehicle_count(&self) -> VehicleCount {
        self.formation.len() as VehicleCount
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
