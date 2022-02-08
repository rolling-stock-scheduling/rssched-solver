use std::fmt;
use crate::distance::Distance;
use crate::time::Duration;
use crate::placeholder::UnitId;

pub(crate) struct Unit {
    id: UnitId,
    unit_type: UnitType,
    initial_dist_counter: Distance, // distance since last maintenance (at start_node)
    initial_tt_counter: Duration // travel time since last maintenance (at start_node)
}

// static functions
impl Unit {
    pub(crate) fn new(id: UnitId, unit_type: UnitType, initial_dist_counter: Distance, initial_tt_counter: Duration) -> Unit {
        Unit{
            id,
            unit_type,
            initial_dist_counter,
            initial_tt_counter
        }
    }
}

// methods
impl Unit {
    pub(crate) fn get_type(&self) -> UnitType {
        self.unit_type
    }
}



impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unit {} ({:?}; {}; {})", self.id, self.unit_type, self.initial_dist_counter, self.initial_tt_counter)
    }
}

#[derive(Debug,Clone,Copy)]
pub(crate) enum UnitType {
    Giruno,
    FVDosto,
    Astoro
}

