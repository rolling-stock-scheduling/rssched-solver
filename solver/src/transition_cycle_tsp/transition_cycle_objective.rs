use rapid_solve::objective::{BaseValue, Coefficient, Indicator, LinearCombination, Objective};

use super::TransitionCycleWithInfo;

struct MaintenanceCounterIndicator;

impl Indicator<TransitionCycleWithInfo> for MaintenanceCounterIndicator {
    fn evaluate(&self, transition_cycle_with_info: &TransitionCycleWithInfo) -> BaseValue {
        BaseValue::Integer(transition_cycle_with_info.get_cycle().maintenance_counter())
    }

    fn name(&self) -> String {
        String::from("maintenanceCounter")
    }
}

pub fn build() -> Objective<TransitionCycleWithInfo> {
    let maintenance_counter = LinearCombination::new(vec![(
        Coefficient::Integer(1),
        Box::new(MaintenanceCounterIndicator),
    )]);
    Objective::new(vec![maintenance_counter])
}
