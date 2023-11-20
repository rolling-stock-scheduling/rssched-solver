use crate::schedule::Schedule;

use objective_framework::{Indicator, Level, ObjBaseValue, ObjCoefficient, Objective};

struct NumberOfDummiesIndicator;

impl Indicator<Schedule> for NumberOfDummiesIndicator {
    fn evaluate(&self, schedule: &Schedule) -> ObjBaseValue {
        ObjBaseValue::Count(schedule.number_of_dummy_tours() as i32)
    }

    fn name(&self) -> String {
        String::from("number of dummies")
    }
}

struct DeadheadDistanceIndicator;

impl Indicator<Schedule> for DeadheadDistanceIndicator {
    fn evaluate(&self, schedule: &Schedule) -> ObjBaseValue {
        ObjBaseValue::count(schedule.total_dead_head_distance() as i32)
    }

    fn name(&self) -> String {
        String::from("deadhead distance")
    }
}

pub fn build_simple_objective() -> Objective<Schedule> {
    let first_level = Level::new(vec![(
        ObjCoefficient::Integer(1),
        Box::new(NumberOfDummiesIndicator),
    )]);

    let second_level = Level::new(vec![(
        ObjCoefficient::Integer(1),
        Box::new(DeadheadDistanceIndicator),
    )]);

    Objective::new(vec![first_level, second_level])
}
