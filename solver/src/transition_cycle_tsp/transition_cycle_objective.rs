// © 2023-2024 ETH Zurich
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// any later version.
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

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
