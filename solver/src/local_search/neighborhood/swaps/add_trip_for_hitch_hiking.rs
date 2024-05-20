use std::fmt;

use model::base_types::{NodeIdx, VehicleIdx};
use solution::{path::Path, Schedule};

use super::{improve_depot_and_recompute_transitions, Swap};

/// Forces a maintenance slot to a given vehicle and spawns a new vehicle for the conflict path.
/// If the maintenance slot is already fully occupied, the last occupant is removed.
#[derive(Clone)]
pub struct AddTripForHitchHiking {
    node: NodeIdx,
    vehicle: VehicleIdx,
}

impl AddTripForHitchHiking {
    pub(crate) fn new(node: NodeIdx, vehicle: VehicleIdx) -> AddTripForHitchHiking {
        AddTripForHitchHiking { node, vehicle }
    }
}

impl Swap for AddTripForHitchHiking {
    fn apply(&self, schedule: &Schedule) -> Result<Schedule, String> {
        let (sched, conflict) = schedule.add_path_to_vehicle_tour(
            self.vehicle,
            Path::new_from_single_node(self.node, schedule.get_network()),
        )?;
        match conflict {
            Some(_) => Err("node causes conflict".to_string()),
            None => Ok(improve_depot_and_recompute_transitions(
                sched,
                vec![self.vehicle],
            )),
        }
    }
}

impl fmt::Display for AddTripForHitchHiking {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AddTripForHitchHiking {} to {}", self.node, self.vehicle)
    }
}
