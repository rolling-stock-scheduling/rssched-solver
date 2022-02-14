use crate::base_types::UnitId;

pub(super) struct TrainFormation {
    units: Vec<UnitId>,
    flipped: bool, // if flipped = false, then units[0] is first, units[1] second, ...; if flipped = true then units[0] is last, units[1] next-to-last ...
}

// static functions
impl TrainFormation {
    pub(super) fn new (units: Vec<UnitId>) -> TrainFormation {
        TrainFormation{units, flipped:false}
    }
}

// methods
impl TrainFormation {
    pub(super) fn add (&mut self, unit: UnitId) {
        self.units.push(unit);
    }
    
    pub(super) fn remove (&mut self, unit: UnitId) {
        // only delete the first occuarnce of the unit (could appear twice as we first add and then
        // remove
        match self.units.iter().position(|&u| u == unit) {
            None => {panic!("Unit {} was not part of the TrainFormation",unit);}
            Some(pos) => {self.units.remove(pos);}
        }
    }
}
