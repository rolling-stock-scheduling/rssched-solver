use crate::base_types::Penalty;
use crate::train_formation::TrainFormation;
pub(super) struct Demand {
    amount: u8
}

// methods
impl Demand {
    pub(crate) fn compute_penalty(&self, train: &TrainFormation) -> Penalty {
        self.amount as Penalty - train.len() as Penalty
    }
}

// static functions
impl Demand {
    pub(crate) fn new(amount: u8) -> Demand {
        Demand{amount}
    }
}
