use rapid_solve::objective::{BaseValue, Coefficient, Indicator, LinearCombination, Objective};

use super::TransitionWithInfo;

struct MaintenanceViolationIndicator;

impl Indicator<TransitionWithInfo> for MaintenanceViolationIndicator {
    fn evaluate(&self, transition_with_info: &TransitionWithInfo) -> BaseValue {
        BaseValue::Integer(
            transition_with_info
                .get_transition()
                .maintenance_violation(),
        )
    }

    fn name(&self) -> String {
        String::from("maintenanceViolation")
    }
}

struct MaintenanceCounterIndicator;

impl Indicator<TransitionWithInfo> for MaintenanceCounterIndicator {
    fn evaluate(&self, transition_with_info: &TransitionWithInfo) -> BaseValue {
        BaseValue::Integer(transition_with_info.get_transition().maintenance_counter())
    }

    fn name(&self) -> String {
        String::from("maintenanceCounter")
    }
}

pub fn build() -> Objective<TransitionWithInfo> {
    let maintenance_violation = LinearCombination::new(vec![(
        Coefficient::Integer(1),
        Box::new(MaintenanceViolationIndicator),
    )]);

    let maintenance_counter = LinearCombination::new(vec![(
        Coefficient::Integer(1),
        Box::new(MaintenanceCounterIndicator),
    )]);
    Objective::new(vec![maintenance_violation, maintenance_counter])
}
