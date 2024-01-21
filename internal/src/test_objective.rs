use model::base_types::Distance;
use objective_framework::{BaseValue, Coefficient, Indicator, Level, Objective};
use solution::Schedule;
use time::Duration;

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

/// Number of Dummies
struct NumberOfDummiesIndicator;

impl Indicator<Schedule> for NumberOfDummiesIndicator {
    fn evaluate(&self, schedule: &Schedule) -> BaseValue {
        BaseValue::Integer(schedule.number_of_dummy_tours() as i64)
    }

    fn name(&self) -> String {
        String::from("numberOfDummies")
    }
}

/// Dummy distance in m
struct DummyDistanceIndicator;

impl Indicator<Schedule> for DummyDistanceIndicator {
    fn evaluate(&self, schedule: &Schedule) -> BaseValue {
        BaseValue::Integer(
            schedule
                .dummy_iter()
                .map(|d| schedule.tour_of(d).unwrap().total_distance())
                .sum::<Distance>()
                .in_meter() as i64,
        )
    }

    fn name(&self) -> String {
        String::from("dummyDistanceTraveled")
    }
}

/// Dummy duration
struct DummyDurationIndicator;

impl Indicator<Schedule> for DummyDurationIndicator {
    fn evaluate(&self, schedule: &Schedule) -> BaseValue {
        BaseValue::Duration(
            schedule
                .dummy_iter()
                .map(|d| schedule.tour_of(d).unwrap().useful_duration())
                .sum::<Duration>(),
        )
    }

    fn name(&self) -> String {
        String::from("dummyDurationTraveled")
    }
}

/// Earliest Dummy Start, time between earliest dummy start and latest end time of a service node
struct EarliestDummyStartIndicator;

impl Indicator<Schedule> for EarliestDummyStartIndicator {
    fn evaluate(&self, schedule: &Schedule) -> BaseValue {
        let latest_end_time = schedule.get_network().latest_end_time();
        BaseValue::Integer(
            (latest_end_time
                - schedule
                    .dummy_iter()
                    .map(|d| schedule.tour_of(d).unwrap().start_time())
                    .min()
                    .unwrap_or(latest_end_time))
            .in_sec() as i64,
        )
    }

    fn name(&self) -> String {
        String::from("earliestDummyStart")
    }
}

/// Sum over all vehicles: overhead duration
struct OverheadDurationIndicator;

impl Indicator<Schedule> for OverheadDurationIndicator {
    fn evaluate(&self, schedule: &Schedule) -> BaseValue {
        BaseValue::Duration(
            schedule
                .vehicles_iter()
                .chain(schedule.dummy_iter())
                .map(|v| schedule.tour_of(v).unwrap().total_overhead_duration())
                .sum(),
        )
    }

    fn name(&self) -> String {
        String::from("overheadDuration")
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
    let _usefull_service_time = Level::new(vec![(
        Coefficient::Integer(-1),
        Box::new(ServiceTimeSquaredIndicator),
    )]);

    let _dummy_level = Level::new(vec![(
        Coefficient::Integer(1),
        Box::new(NumberOfDummiesIndicator),
    )]);

    let _dummy_distance_level = Level::new(vec![(
        Coefficient::Integer(1),
        Box::new(DummyDistanceIndicator),
    )]);

    let _dummy_duration_level = Level::new(vec![(
        Coefficient::Integer(1),
        Box::new(DummyDurationIndicator),
    )]);

    let _earliest_dummy_start_level = Level::new(vec![(
        Coefficient::Integer(1),
        Box::new(EarliestDummyStartIndicator),
    )]);

    let _overhead_duration_level = Level::new(vec![(
        Coefficient::Integer(1),
        Box::new(OverheadDurationIndicator),
    )]);

    let _unserved_passengers_level = Level::new(vec![(
        Coefficient::Integer(1),
        Box::new(NumberOfUnservedPassengersIndicator),
    )]);

    let _vehicle_count_level = Level::new(vec![(
        Coefficient::Integer(1),
        Box::new(NumberOfVehiclesIndicator),
    )]);

    let _overhead_seat_distance_level = Level::new(vec![(
        Coefficient::Integer(1),
        Box::new(OverheadSeatDistanceIndicator),
    )]);

    Objective::new(vec![
        _earliest_dummy_start_level,
        // _dummy_level,
        _vehicle_count_level,
        // _overhead_duration_level,
        // _overhead_seat_distance_level,
    ])
}
