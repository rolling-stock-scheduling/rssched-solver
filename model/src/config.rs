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

use rapid_time::Duration;

use crate::base_types::{Cost, Distance};

pub struct Config {
    pub forbid_dead_head_trip: bool,
    pub day_limit_threshold: Duration,
    pub shunting: ShuntingConfig,
    pub maintenance: MaintenanceConfig,
    pub costs: CostsConfig,
}

pub struct ShuntingConfig {
    pub minimal: Duration,
    pub dead_head_trip: Duration,
}

pub struct MaintenanceConfig {
    pub maximal_distance: Distance,
}

pub struct CostsConfig {
    pub staff: Cost,
    pub service_trip: Cost,
    pub maintenance: Cost,
    pub dead_head_trip: Cost,
    pub idle: Cost,
}

impl Config {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        forbid_dead_head_trip: bool,
        day_limit_threshold: Duration,
        shunting_minimal: Duration,
        shunting_dead_head_trip: Duration,
        maintenance_maximal_distance: Distance,
        costs_staff: Cost,
        costs_service_trip: Cost,
        costs_maintenance: Cost,
        costs_dead_head_trip: Cost,
        costs_idle: Cost,
    ) -> Config {
        Config {
            forbid_dead_head_trip,
            day_limit_threshold,
            shunting: ShuntingConfig {
                minimal: shunting_minimal,
                dead_head_trip: shunting_dead_head_trip,
            },
            maintenance: MaintenanceConfig {
                maximal_distance: maintenance_maximal_distance,
            },
            costs: CostsConfig {
                staff: costs_staff,
                service_trip: costs_service_trip,
                maintenance: costs_maintenance,
                dead_head_trip: costs_dead_head_trip,
                idle: costs_idle,
            },
        }
    }
}
