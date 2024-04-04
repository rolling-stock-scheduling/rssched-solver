use objective_framework::{BaseValue, Coefficient, Indicator, Level, Objective};
use solution::Schedule;

struct MaintenanceViolationIndicator;
impl Indicator<Schedule> for MaintenanceViolationIndicator {
    fn evaluate(&self, schedule: &Schedule) -> BaseValue {
        BaseValue::Integer(schedule.maintenance_violation())
    }

    fn name(&self) -> String {
        String::from("maintenanceViolation")
    }
}

/// Sum over all service trips: max{0, passengers - capacity} + max{0, seated_passengers - seats}
struct UnservedPassengersIndicator;

impl Indicator<Schedule> for UnservedPassengersIndicator {
    fn evaluate(&self, schedule: &Schedule) -> BaseValue {
        let unserved_passengers = schedule.unserved_passengers();
        BaseValue::Integer((unserved_passengers.0 + unserved_passengers.1) as i64)
    }

    fn name(&self) -> String {
        String::from("unservedPassengers")
    }
}

/// Number of vehicles (each type count as 1)
struct VehicleCountIndicator;

impl Indicator<Schedule> for VehicleCountIndicator {
    fn evaluate(&self, schedule: &Schedule) -> BaseValue {
        BaseValue::Integer(schedule.number_of_vehicles() as i64)
    }

    fn name(&self) -> String {
        String::from("vehicleCount")
    }
}

struct CostsIndicator;

impl Indicator<Schedule> for CostsIndicator {
    fn evaluate(&self, schedule: &Schedule) -> BaseValue {
        BaseValue::Integer(schedule.costs() as i64)
    }

    fn name(&self) -> String {
        String::from("costs")
    }
}

pub fn build() -> Objective<Schedule> {
    let maintenance_violation = Level::new(vec![(
        Coefficient::Integer(1),
        Box::new(MaintenanceViolationIndicator),
    )]);

    let unserved_passengers = Level::new(vec![(
        Coefficient::Integer(1),
        Box::new(UnservedPassengersIndicator),
    )]);

    let vehicle_count = Level::new(vec![(
        Coefficient::Integer(1),
        Box::new(VehicleCountIndicator),
    )]);

    let costs = Level::new(vec![(Coefficient::Integer(1), Box::new(CostsIndicator))]);

    Objective::new(vec![
        maintenance_violation,
        unserved_passengers,
        vehicle_count,
        costs,
    ])
}
