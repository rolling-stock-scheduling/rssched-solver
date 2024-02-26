use objective_framework::{BaseValue, Coefficient, Indicator, Level, Objective};
use solution::Schedule;

/// Sum over all service trips max{0, passengers - seats}
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
    let unserved_passengers = Level::new(vec![(
        Coefficient::Integer(1),
        Box::new(UnservedPassengersIndicator),
    )]);

    let vehicle_count = Level::new(vec![(
        Coefficient::Integer(1),
        Box::new(VehicleCountIndicator),
    )]);

    let overhead_seat_distance = Level::new(vec![(
        Coefficient::Integer(1),
        Box::new(OverheadSeatDistanceIndicator),
    )]);

    Objective::new(vec![
        unserved_passengers,
        vehicle_count,
        overhead_seat_distance,
    ])
}
