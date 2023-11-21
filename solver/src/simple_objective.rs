use objective_framework::{BaseValue, Coefficient, Indicator, Level, Objective};
use sbb_solution::Schedule;

struct NumberOfDummiesIndicator;

impl Indicator<Schedule> for NumberOfDummiesIndicator {
    fn evaluate(&self, schedule: &Schedule) -> BaseValue {
        BaseValue::Integer(schedule.number_of_dummy_tours() as i64)
    }

    fn name(&self) -> String {
        String::from("number of dummies")
    }
}

struct DeadheadDistanceIndicator;

impl Indicator<Schedule> for DeadheadDistanceIndicator {
    fn evaluate(&self, schedule: &Schedule) -> BaseValue {
        BaseValue::Integer(schedule.total_dead_head_distance().in_meter() as i64)
    }

    fn name(&self) -> String {
        String::from("deadhead distance")
    }
}

pub fn build_simple_objective() -> Objective<Schedule> {
    let first_level = Level::new(vec![(
        Coefficient::Integer(1),
        Box::new(NumberOfDummiesIndicator),
    )]);

    let second_level = Level::new(vec![(
        Coefficient::Integer(1),
        Box::new(DeadheadDistanceIndicator),
    )]);

    Objective::new(vec![first_level, second_level])
}
