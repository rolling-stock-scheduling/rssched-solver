pub mod swaps;
use model::base_types::{NodeIdx, VehicleIdx};
use model::network::Network;
use rapid_solve::heuristics::common::ParallelNeighborhood;
use rapid_time::Duration;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use solution::{segment::Segment, Schedule};
use std::sync::Arc;

use std::iter;

use self::swaps::{PathExchange, SpawnVehicleForMaintenance, Swap, SwapInfo};

use super::ScheduleWithInfo;

#[derive(Clone)]
pub struct RSSchedParallelNeighborhood {
    segment_length_limit: Option<Duration>,
    overhead_threshold: Option<Duration>,
    network: Arc<Network>,
}

impl RSSchedParallelNeighborhood {
    pub fn new(
        segment_length_limit: Option<Duration>,
        overhead_threshold: Option<Duration>,
        network: Arc<Network>,
    ) -> RSSchedParallelNeighborhood {
        RSSchedParallelNeighborhood {
            segment_length_limit,
            overhead_threshold,
            network,
        }
    }
}

impl ParallelNeighborhood<ScheduleWithInfo> for RSSchedParallelNeighborhood {
    fn neighbors_of<'a>(
        &'a self,
        schedule_with_info: &'a ScheduleWithInfo,
    ) -> impl ParallelIterator<Item = ScheduleWithInfo> + 'a {
        let spawning_iterator = self.spawn_vehicle_for_maintenance_iterator(schedule_with_info);
        let segment_exchange_iterator = self.segment_exchange_iterator(schedule_with_info);
        let hitch_hiking_iterator = self.hitch_hiking_iterator(schedule_with_info);
        let remove_single_node_iterator = self.remove_single_node_iterator(schedule_with_info);
        spawning_iterator
            .chain(segment_exchange_iterator)
            .chain(hitch_hiking_iterator)
            .chain(remove_single_node_iterator)
    }
}

impl RSSchedParallelNeighborhood {
    pub fn spawn_vehicle_for_maintenance_iterator<'a>(
        &'a self,
        schedule_with_info: &'a ScheduleWithInfo,
    ) -> impl ParallelIterator<Item = ScheduleWithInfo> + 'a {
        let schedule = schedule_with_info.get_schedule();

        let mut maintenance_nodes: Vec<NodeIdx> = self
            .network
            .maintenance_nodes()
            .filter(|&m| {
                schedule.train_formation_of(m).vehicle_count()
                    < self.network.track_count_of_maintenance_slot(m)
            })
            .collect();

        // sort maintenance nodes by workload
        maintenance_nodes.sort_by_key(|&m| {
            (schedule.train_formation_of(m).vehicle_count() * 10000)
                / self.network.track_count_of_maintenance_slot(m)
        });

        maintenance_nodes
            .into_par_iter()
            .flat_map(move |maintenance| {
                let receivers: Vec<_> = schedule.vehicles_iter_all().collect();
                receivers.into_par_iter().filter_map(move |receiver| {
                    let swap = SpawnVehicleForMaintenance::new(maintenance, receiver);
                    match swap.apply(schedule) {
                        Ok(new_schedule) => Some(ScheduleWithInfo::new(
                            new_schedule,
                            SwapInfo::SpawnVehicleForMaintenance(receiver),
                            format!(
                                "{} ({})",
                                swap,
                                self.network
                                    .vehicle_types()
                                    .get(schedule.vehicle_type_of(receiver).unwrap())
                                    .unwrap(),
                            ),
                        )),
                        Err(_) => None,
                    }
                })
            })
    }

    /// Creates all PathExchanges where every vehicle is receiver and every other vehicle is provider.
    /// All segments starting and ending at a non-depot node are considered.
    /// The segment time length is smaller than the threshold. (None means unlimite.)
    /// The length of a segment is measured from the start_time of the first node to the end_time of
    /// the last node.
    pub fn segment_exchange_iterator<'a>(
        &'a self,
        schedule_with_info: &'a ScheduleWithInfo,
    ) -> impl ParallelIterator<Item = ScheduleWithInfo> + 'a {
        let schedule = schedule_with_info.get_schedule();
        let mut providers: Vec<VehicleIdx> = self.dummy_and_real_vehicles(schedule).collect();

        // rotate providers such that start_provider is the first provider
        // e.g. start_provider = v5
        // so v0, v1, v2, v3, v4, v5, v6, v7, v8, v9
        // becomes v5, v6, v7, v8, v9, v0, v1, v2, v3, v4
        if let SwapInfo::PathExchange(last_provider) = schedule_with_info.get_last_swap_info() {
            if let Some(position) = providers.iter().position(|&v| v == last_provider) {
                providers.rotate_left(position);
            }
        }

        // as provider first take dummies then real Vehicles:
        providers.into_par_iter().flat_map(move |provider|
            // create segment of provider's tour
            self.segments(provider, schedule)
            .flat_map(move |seg|
                // as receiver first take the real Vehicles then the dummies
                self.real_and_dummy_vehicles(schedule)
                // skip provider as receiver
                .filter(move |&u| u != provider)
                // create the swap
                .filter_map(move |receiver|{
                    let swap = PathExchange::new(seg, provider, receiver);
                    match swap.apply(schedule) {
                        Ok(new_schedule) => {
                            Some(ScheduleWithInfo::new(
                                new_schedule,
                                SwapInfo::PathExchange(provider),
                                format!(
                                    "PathExchange {} from {}{} to {}{}",
                                    seg,
                                    provider,
                                    schedule.vehicle_type_of(provider).map(|vt| format!(" ({})", self.network.vehicle_types().get(vt).unwrap())).unwrap_or("".to_string()),
                                    receiver,
                                    schedule.vehicle_type_of(receiver).map(|vt| format!(" ({})", self.network.vehicle_types().get(vt).unwrap())).unwrap_or("".to_string()),
                                )
                            ))
                        }
                        Err(_) => None,
                    }
                })
            ))
    }

    pub fn hitch_hiking_iterator<'a>(
        &'a self,
        schedule_with_info: &'a ScheduleWithInfo,
    ) -> impl ParallelIterator<Item = ScheduleWithInfo> + Send + Sync + 'a {
        let schedule = schedule_with_info.get_schedule();

        let vehicles: Vec<_> = schedule.vehicles_iter_all().collect();

        vehicles.into_par_iter().flat_map(move |vehicle| {
            let vehicle_type = schedule.vehicle_type_of(vehicle).unwrap();
            let service_nodes: Vec<_> = self.network.service_nodes(vehicle_type).collect();
            service_nodes.into_par_iter().filter_map(move |node| {
                let swap = swaps::AddTripForHitchHiking::new(node, vehicle);
                match swap.apply(schedule) {
                    Ok(new_schedule) => Some(ScheduleWithInfo::new(
                        new_schedule,
                        SwapInfo::AddTripForHitchHiking(vehicle),
                        format!("{}", swap),
                    )),
                    Err(_) => None,
                }
            })
        })
    }

    pub fn remove_single_node_iterator<'a>(
        &'a self,
        schedule_with_info: &'a ScheduleWithInfo,
    ) -> impl ParallelIterator<Item = ScheduleWithInfo> + Send + Sync + 'a {
        let schedule = schedule_with_info.get_schedule();
        let vehicles: Vec<_> = schedule.vehicles_iter_all().collect();

        vehicles.into_par_iter().flat_map(move |vehicle| {
            let tour = schedule.tour_of(vehicle).unwrap();
            let non_depot_nodes: Vec<_> = tour.all_non_depot_nodes_iter().collect();
            non_depot_nodes.into_par_iter().filter_map(move |node| {
                let swap = swaps::RemoveSingleNode::new(node, vehicle);
                match swap.apply(schedule) {
                    Ok(new_schedule) => Some(ScheduleWithInfo::new(
                        new_schedule,
                        SwapInfo::RemoveSingleNode(vehicle),
                        format!("{}", swap),
                    )),
                    Err(_) => None,
                }
            })
        })
    }

    fn segments<'a>(
        &'a self,
        provider: VehicleIdx,
        schedule: &'a Schedule,
    ) -> impl ParallelIterator<Item = Segment> + 'a {
        let threshold = match self.overhead_threshold {
            None => Duration::ZERO,
            Some(d) => d,
        };
        let tour = schedule
            .tour_of(provider)
            .expect("provider not in schedule");
        // all non-depot nodes of provider's tour might be the start of a segment
        let start_nodes: Vec<_> = tour.all_non_depot_nodes_iter().enumerate().collect();
        start_nodes
            .into_par_iter()
            // only take nodes with enough preceding overhead:
            .filter(move |(_, n)| {
                schedule.is_dummy(provider) || tour.preceding_overhead(*n).unwrap() >= threshold
            })
            .flat_map(move |(i, seg_start)| {
                // all non-depot nodes (after the start) could be the end of the segment
                let end_nodes: Vec<_> = tour
                    .all_non_depot_nodes_iter()
                    .skip(i)
                    // only take nodes with enough subsequent overhead:
                    .filter(move |n| {
                        schedule.is_dummy(provider)
                            || tour.subsequent_overhead(*n).unwrap() >= threshold
                    })
                    // only take the nodes such that the segment is not longer than the threshold
                    .take_while(move |seg_end| {
                        self.segment_length_limit.is_none()
                            || self.network.node(*seg_end).end_time()
                                - self.network.node(seg_start).start_time()
                                <= self.segment_length_limit.unwrap()
                    })
                    // if provider is a real vehicle, add the last_node as segment end. (note that is not taken twice as EndNodes
                    // end at time Infinity
                    .chain(
                        iter::once(schedule.tour_of(provider).unwrap().last_node())
                            .filter(move |_n| self.segment_length_limit.is_some()),
                    )
                    .collect();
                // create the segment
                end_nodes
                    .into_par_iter()
                    .map(move |seg_end| Segment::new(seg_start, seg_end))
            })
            // test whether the segment can be removed
            .filter(move |seg| {
                schedule
                    .tour_of(provider)
                    .unwrap()
                    .check_removable(*seg)
                    .is_ok()
            })
    }

    fn dummy_and_real_vehicles<'a>(
        &'a self,
        schedule: &'a Schedule,
    ) -> impl ParallelIterator<Item = VehicleIdx> + 'a {
        let vehicles: Vec<_> = schedule
            .dummy_iter()
            .chain(schedule.vehicles_iter_all())
            .collect();
        vehicles.into_par_iter()
    }

    fn real_and_dummy_vehicles<'a>(
        &'a self,
        schedule: &'a Schedule,
    ) -> impl ParallelIterator<Item = VehicleIdx> + 'a {
        let vehicles: Vec<_> = schedule
            .vehicles_iter_all()
            .chain(schedule.dummy_iter())
            .collect();
        vehicles.into_par_iter()
    }
}
