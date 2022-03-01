use crate::base_types::UnitId;
use crate::units::{Unit, Units};
use std::fmt;

use std::iter::Iterator;

use std::rc::Rc;

#[derive(Clone)]
pub(crate) struct TrainFormation {
    formation: Vec<UnitId>, // index 0 is at tail, index len()-1 is front
    units: Rc<Units>
}

// static functions
impl TrainFormation {
    pub(crate) fn new (formation: Vec<UnitId>, units: Rc<Units>) -> TrainFormation {
        TrainFormation{formation, units}
    }
}

// methods
impl TrainFormation {
    pub(crate) fn replace(&self, old: UnitId, new: UnitId) -> TrainFormation {
        let mut new_formation = self.formation.clone();
        let pos = new_formation.iter().position(|&u| u == old).expect("Unit was not part of the TrainFormation");

        // replace old by new:
        new_formation.push(new);
        new_formation.swap_remove(pos);

        TrainFormation{formation: new_formation, units: self.units.clone()}


    }

    pub(crate) fn iter(&self) -> impl Iterator<Item=UnitId> + '_ {
        self.formation.iter().copied()
    }

    pub(crate) fn get_as_units(&self) -> Vec<&Unit> {
        self.formation.iter().map(|&id| self.units.get_unit(id)).collect()
    }

    pub(crate) fn len(&self) -> usize {
        self.formation.len()
    }
}

impl fmt::Display for TrainFormation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.formation.len() > 0 {
            for unit in self.formation.iter() {
                write!(f, "[{}]->", unit)?;
            }
        } else {
            write!(f, "---")?;
        }
        Ok(())
    }
}
