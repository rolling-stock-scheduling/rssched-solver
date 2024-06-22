use objective_framework::{BaseValue, Coefficient, Indicator, Level, Objective};

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
    let maintenance_counter = Level::new(vec![(
        Coefficient::Integer(1),
        Box::new(MaintenanceCounterIndicator),
    )]);
    Objective::new(vec![maintenance_counter])
}
