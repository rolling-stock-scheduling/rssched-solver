use crate::base_types::UnitId;
use crate::units::{Unit, Units};
use std::fmt;


pub(super) struct TrainFormation<'a> {
    formation: Vec<UnitId>,
    flipped: bool, // if flipped = false, then formation[0] is first, formation[1] second, ...; if flipped = true then formation[0] is last, formation[1] next-to-last ...
    units: &'a Units
}

// static functions
impl<'a> TrainFormation<'a> {
    pub(crate) fn new (formation: Vec<UnitId>, units: &'a Units) -> TrainFormation<'a> {
        TrainFormation{formation, flipped:false, units}
    }
}

// methods
impl<'a> TrainFormation<'a> {
    pub(crate) fn add (&mut self, unit: UnitId) {
        self.formation.push(unit);
    }

    pub(crate) fn remove (&mut self, unit: UnitId) {
        // only delete the first occuarnce of the unit (could appear twice as we first add and then
        // remove
        match self.formation.iter().position(|&u| u == unit) {
            None => {panic!("Unit {} was not part of the TrainFormation",unit);}
            Some(pos) => {self.formation.remove(pos);}
        }
    }

    pub(crate) fn get(&self) -> Vec<UnitId> {
        if self.flipped {
            self.formation.iter().rev().copied().collect()
        } else {
            self.formation.iter().copied().collect()
        }
    }

    pub(crate) fn get_as_units(&self) -> Vec<&Unit> {
        if self.flipped {
            self.formation.iter().rev().map(|&id| self.units.get_unit(id)).collect()
        } else {
            self.formation.iter().map(|&id| self.units.get_unit(id)).collect()
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.formation.len()
    }
}

impl<'a> fmt::Display for TrainFormation<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.formation.len() > 0 {
            for unit in self.formation.iter().rev() {
                write!(f, "[{}]->", unit)?;
            }
        } else {
            write!(f, "---")?;
        }
        Ok(())
    }
}
