use sbb_model::base_types::VehicleId;
use sbb_model::vehicles::Vehicles;
// use crate::Vehicles::vehicle;
use std::fmt;

use std::iter::Iterator;

use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct TrainFormation {
    formation: Vec<VehicleId>, // index 0 is at tail, index len()-1 is front
    vehicles: Arc<Vehicles>,
}

// static functions
impl TrainFormation {
    pub(crate) fn new(formation: Vec<VehicleId>, vehicles: Arc<Vehicles>) -> TrainFormation {
        TrainFormation {
            formation,
            vehicles,
        }
    }
}

// methods
impl TrainFormation {
    pub(crate) fn replace(&self, old: VehicleId, new: VehicleId) -> TrainFormation {
        let mut new_formation = self.formation.clone();
        let pos = new_formation
            .iter()
            .position(|&u| u == old)
            .expect("vehicle was not part of the TrainFormation");

        // replace old by new:
        new_formation.push(new);
        new_formation.swap_remove(pos);

        TrainFormation {
            formation: new_formation,
            vehicles: self.vehicles.clone(),
        }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = VehicleId> + '_ {
        self.formation.iter().copied()
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
