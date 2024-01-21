use objective_framework::{BaseValue, Coefficient, Indicator, Level, Objective};
use solution::Schedule;

struct NumberOfUnservedPassengersIndicator;

/// Sum over all service trips max{0, passengers - seats}
impl Indicator<Schedule> for NumberOfUnservedPassengersIndicator {
    fn evaluate(&self, schedule: &Schedule) -> BaseValue {
        BaseValue::Integer(schedule.unserved_passengers() as i64)
    }

    fn name(&self) -> String {
        String::from("unservedPassengers")
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

struct TotalDeadHeadDistanceIndicator;

impl Indicator<Schedule> for TotalDeadHeadDistanceIndicator {
    fn evaluate(&self, schedule: &Schedule) -> BaseValue {
        BaseValue::Integer(schedule.total_dead_head_distance().in_meter() as i64)
    }

    fn name(&self) -> String {
        String::from("deadHeadDistanceTraveled")
    }
}

struct ServiceTimeSquaredIndicator;

impl Indicator<Schedule> for ServiceTimeSquaredIndicator {
    fn evaluate(&self, schedule: &Schedule) -> BaseValue {
        BaseValue::Integer(
            schedule
                .vehicles_iter()
                .map(|v| {
                    schedule
                        .tour_of(v)
                        .unwrap()
                        .useful_duration()
                        .in_min()
                        .pow(2) as i64
                })
                .sum(),
        )
    }

    fn name(&self) -> String {
        String::from("serviceTimeSquared")
    }
}

pub fn _build() -> Objective<Schedule> {
    let usefull_service_time = Level::new(vec![(
        Coefficient::Integer(-1),
        Box::new(ServiceTimeSquaredIndicator),
    )]);

    let first_level = Level::new(vec![(
        Coefficient::Integer(1),
        Box::new(NumberOfUnservedPassengersIndicator),
    )]);

    let second_level = Level::new(vec![(
        Coefficient::Integer(1),
        Box::new(NumberOfVehiclesIndicator),
    )]);

    let third_level = Level::new(vec![(
        Coefficient::Integer(1),
        Box::new(TotalDeadHeadDistanceIndicator),
    )]);

    Objective::new(vec![
        usefull_service_time,
        first_level,
        second_level,
        third_level,
    ])
}
