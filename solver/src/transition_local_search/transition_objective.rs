use objective_framework::{BaseValue, Coefficient, Indicator, Level, Objective};
use solution::transition::Transition;

struct MaintenanceViolationIndicator;

impl Indicator<Transition> for MaintenanceViolationIndicator {
    fn evaluate(&self, transition: &Transition) -> BaseValue {
        BaseValue::Integer(transition.maintenance_violation())
    }

    fn name(&self) -> String {
        String::from("maintenanceViolation")
    }
}

struct MaintenanceCounterIndicator;

impl Indicator<Transition> for MaintenanceCounterIndicator {
    fn evaluate(&self, transition: &Transition) -> BaseValue {
        BaseValue::Integer(transition.maintenance_counter())
    }

    fn name(&self) -> String {
        String::from("maintenanceCounter")
    }
}

pub fn build() -> Objective<Transition> {
    let maintenance_violation = Level::new(vec![(
        Coefficient::Integer(1),
        Box::new(MaintenanceViolationIndicator),
    )]);

    let maintenance_counter = Level::new(vec![(
        Coefficient::Integer(1),
        Box::new(MaintenanceCounterIndicator),
    )]);
    Objective::new(vec![maintenance_violation, maintenance_counter])
}
