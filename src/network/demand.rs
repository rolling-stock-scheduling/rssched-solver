use crate::base_types::Penalty;
use crate::units::UnitType;
use crate::schedule::train_formation::TrainFormation;

#[derive(Clone,Copy)]
pub(crate) struct Demand {
    amount: u8
}

// methods
impl Demand {
    pub(crate) fn compute_penalty(&self, train: &TrainFormation) -> Penalty {
        if (self.amount as Penalty) < (train.len() as Penalty) {
            panic!("The Demand is smaller than the train-length");
        }
        self.amount as Penalty - train.len() as Penalty
    }

    pub(crate) fn get_valid_types(&self) -> Vec<UnitType> {
        vec!(UnitType::Standard; self.amount as usize)
    }
    
    pub(crate) fn get_missing_types(&self, unit_types: &Vec<UnitType>) -> Vec<UnitType> {
        vec!(UnitType::Standard; self.amount as usize - unit_types.len())
    }

    pub(crate) fn number_of_units(&self) -> u8 {
        self.amount
    }
}

// static functions
impl Demand {
    pub(crate) fn new(amount: u8) -> Demand {
        Demand{amount}
    }
}
