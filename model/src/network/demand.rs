use crate::vehicles::VehicleType;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Demand {
    amount: u8,
}

// methods
impl Demand {
    pub fn get_valid_types(&self) -> Vec<VehicleType> {
        vec![VehicleType::Standard; self.amount as usize]
    }

    pub fn get_missing_types(&self, vehicle_types: &Vec<VehicleType>) -> Vec<VehicleType> {
        vec![VehicleType::Standard; self.amount as usize - vehicle_types.len()]
    }

    pub fn number_of_vehicles(&self) -> u8 {
        self.amount
    }
}

// static functions
impl Demand {
    pub fn new(amount: u8) -> Demand {
        Demand { amount }
    }
}
