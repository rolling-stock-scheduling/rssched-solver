use objective_framework::{BaseValue, Coefficient, Indicator, Level, Objective};
use sbb_solution::Schedule;

struct NumberOfUnservedPassengersIndicator;

/// Sum over all service trips max{0, passengers - seats}
impl Indicator<Schedule> for NumberOfUnservedPassengersIndicator {
    fn evaluate(&self, schedule: &Schedule) -> BaseValue {
        BaseValue::Integer(schedule.number_of_unserved_passengers() as i64)
    }

    fn name(&self) -> String {
        String::from("numberOfUnservedPassengers")
    }
}

/// Number of vehicles (each type count as 1)
struct NumberOfVehiclesIndicator;

impl Indicator<Schedule> for NumberOfVehiclesIndicator {
    fn evaluate(&self, schedule: &Schedule) -> BaseValue {
        BaseValue::Integer(schedule.number_of_vehicles() as i64)
    }

    fn name(&self) -> String {
        String::from("numberOfVehicles")
    }
}

/// Sum over all vehicles: distance in m * number of seats
/// - sum over all service trips: distance in km * number of passengers
struct OverheadSeatDistanceIndicator;

impl Indicator<Schedule> for OverheadSeatDistanceIndicator {
    fn evaluate(&self, schedule: &Schedule) -> BaseValue {
        BaseValue::Integer(schedule.seat_distance_traveled() as i64)
    }

    fn name(&self) -> String {
        String::from("seatDistanceTraveled")
    }
}

pub fn build() -> Objective<Schedule> {
    let first_level = Level::new(vec![(
        Coefficient::Integer(-1),
        Box::new(NumberOfUnservedPassengersIndicator),
    )]);

    let third_level = Level::new(vec![(
        Coefficient::Integer(-1),
        Box::new(OverheadSeatDistanceIndicator),
    )]);

    Objective::new(vec![first_level, third_level])
}
