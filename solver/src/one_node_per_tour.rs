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

use model::network::Network;
use rapid_solve::objective::{EvaluatedSolution, Objective};
use solution::Schedule;
use std::sync::Arc;

pub struct OneNodePerTour {
    network: Arc<Network>,
    objective: Arc<Objective<Schedule>>,
}
impl OneNodePerTour {
    pub fn initialize(network: Arc<Network>, objective: Arc<Objective<Schedule>>) -> Self {
        Self { network, objective }
    }

    pub fn solve(&self) -> EvaluatedSolution<Schedule> {
        let mut schedule = Schedule::empty(self.network.clone());

        for service_trip in self.network.all_service_nodes() {
            while !schedule.is_fully_covered(service_trip) {
                let vehicle_type = self.network.vehicle_type_for(service_trip);

                schedule = schedule
                    .spawn_vehicle_for_path(vehicle_type, vec![service_trip])
                    .unwrap()
                    .0;
            }
        }

        self.objective.evaluate(schedule)
    }
}
