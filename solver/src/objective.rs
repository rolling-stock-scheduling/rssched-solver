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

use crate::local_search::ScheduleWithInfo;
use rapid_solve::objective::{BaseValue, Coefficient, Indicator, LinearCombination, Objective};

/// Sum over all service trips: max{0, passengers - capacity} + max{0, seated_passengers - seats}
struct UnservedPassengersIndicator;

impl Indicator<ScheduleWithInfo> for UnservedPassengersIndicator {
    fn evaluate(&self, schedule_with_info: &ScheduleWithInfo) -> BaseValue {
        let unserved_passengers = schedule_with_info.get_schedule().unserved_passengers();
        BaseValue::Integer((unserved_passengers.0 + unserved_passengers.1) as i64)
    }

    fn name(&self) -> String {
        String::from("unservedPassengers")
    }
}

/// Each fleet is partitioned into rotation cycles, if total length exceeds the maintenance limit,
/// the excess is counted as violation
struct MaintenanceViolationIndicator;

impl Indicator<ScheduleWithInfo> for MaintenanceViolationIndicator {
    fn evaluate(&self, schedule_with_info: &ScheduleWithInfo) -> BaseValue {
        BaseValue::Integer(schedule_with_info.get_schedule().maintenance_violation())
    }

    fn name(&self) -> String {
        String::from("maintenanceViolation")
    }
}

/// Number of vehicles (each type count as 1)
struct VehicleCountIndicator;

impl Indicator<ScheduleWithInfo> for VehicleCountIndicator {
    fn evaluate(&self, schedule_with_info: &ScheduleWithInfo) -> BaseValue {
        BaseValue::Integer(schedule_with_info.get_schedule().number_of_vehicles() as i64)
    }

    fn name(&self) -> String {
        String::from("vehicleCount")
    }
}

struct CostsIndicator;

impl Indicator<ScheduleWithInfo> for CostsIndicator {
    fn evaluate(&self, schedule_with_info: &ScheduleWithInfo) -> BaseValue {
        BaseValue::Integer(schedule_with_info.get_schedule().costs() as i64)
    }

    fn name(&self) -> String {
        String::from("costs")
    }
}

pub fn build() -> Objective<ScheduleWithInfo> {
    let maintenance_violation = LinearCombination::new(vec![(
        Coefficient::Integer(1),
        Box::new(MaintenanceViolationIndicator),
    )]);

    let unserved_passengers = LinearCombination::new(vec![(
        Coefficient::Integer(1),
        Box::new(UnservedPassengersIndicator),
    )]);

    let vehicle_count = LinearCombination::new(vec![(
        Coefficient::Integer(1),
        Box::new(VehicleCountIndicator),
    )]);

    let costs = LinearCombination::new(vec![(Coefficient::Integer(1), Box::new(CostsIndicator))]);

    Objective::new(vec![
        unserved_passengers,
        maintenance_violation,
        vehicle_count,
        costs,
    ])
}
