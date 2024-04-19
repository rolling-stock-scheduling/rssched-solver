pub mod swaps;
use heuristic_framework::local_search::Neighborhood;
use model::base_types::VehicleIdx;
use model::network::Network;
use solution::{segment::Segment, Schedule};
use std::sync::Arc;
use time::Duration;

use std::iter;

use self::swaps::{PathExchange, SpawnVehicleForMaintenance, Swap};

use super::ScheduleWithInfo;

///////////////////////////////////////////////////////////
////////////////// LimitedExchanges ///////////////////////
///////////////////////////////////////////////////////////

/// Creates all PathExchanges where every vehicle is receiver and every other vehicle is provider.
/// All segments starting and ending at a non-depot node are considered.
/// The segment time length is smaller than the threshold. (None means unlimite.)
/// The length of a segment is measured from the start_time of the first node to the end_time of
/// the last node.
#[derive(Clone)]
pub struct SpawnForMaintenanceAndPathExchange {
    segment_length_limit: Option<Duration>,
    overhead_threshold: Option<Duration>,
    only_dummy_provider: bool,
    network: Arc<Network>,
}

impl SpawnForMaintenanceAndPathExchange {
    pub(crate) fn new(
        segment_length_limit: Option<Duration>,
        overhead_threshold: Option<Duration>,
        only_dummy_provider: bool,
        network: Arc<Network>,
    ) -> SpawnForMaintenanceAndPathExchange {
        SpawnForMaintenanceAndPathExchange {
            segment_length_limit,
            overhead_threshold,
            only_dummy_provider,
            network,
        }
    }
}

impl Neighborhood<ScheduleWithInfo> for SpawnForMaintenanceAndPathExchange {
    fn neighbors_of<'a>(
        &'a self,
        schedule_with_info: &'a ScheduleWithInfo,
        // start_provider: Option<VehicleId>,
    ) -> Box<dyn Iterator<Item = ScheduleWithInfo> + Send + Sync + 'a> {
        ///////////////////////////////////////////
        // first: spawn vehicles for maintenance //
        ///////////////////////////////////////////

        let schedule = schedule_with_info.get_schedule();

        // TODO: next neighborhood start with a different maintenance node (for speedup)
        let spawning_iterator = self
            .network
            .maintenance_nodes()
            .flat_map(move |maintenance| {
                let vehicle_types: Vec<_> = self.network.vehicle_types().iter().collect();
                vehicle_types.into_iter().filter_map(move |vehicle_type| {
                    let swap = SpawnVehicleForMaintenance::new(maintenance, vehicle_type);
                    match swap.apply(schedule) {
                        Ok(schedule) => Some(ScheduleWithInfo::new(
                            schedule,
                            None,
                            format!(
                                "Spawned vehicle of type {} for maintenance slot {}",
                                self.network.vehicle_types().get(vehicle_type).unwrap(), maintenance
                            ),
                        )),
                        Err(_) => None,
                    }
                })
            });

        ///////////////////////////////
        // second: exchange segments //
        ///////////////////////////////

        let providers: Vec<VehicleIdx> = if self.only_dummy_provider {
            schedule.dummy_iter().collect()
        } else {
            self.dummy_and_real_vehicles(schedule).collect()
        };

        // TODO: next neighborhood start with different provider (for speedup)
        // rotate providers such that start_provider is the first provider
        // e.g. start_provider = v5
        // so v0, v1, v2, v3, v4, v5, v6, v7, v8, v9
        // becomes v5, v6, v7, v8, v9, v0, v1, v2, v3, v4
        // if let Some(position) = providers.iter().position(|&v| Some(v) == start_provider) {
        // providers.rotate_left(position);
        // }

        let segment_exchange_iterator =
            // as provider first take dummies then real Vehicles:
            providers.into_iter().flat_map(move |provider|
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
                                    Some(provider), 
                                    format!(
                                        "PathExchange from {}{} to {}{} of segment {}", 
                                        provider, 
                                        schedule.vehicle_type_of(provider).map(|vt| format!(" ({})", self.network.vehicle_types().get(vt).unwrap())).unwrap_or("".to_string()),
                                        receiver, 
                                        schedule.vehicle_type_of(receiver).map(|vt| format!(" ({})", self.network.vehicle_types().get(vt).unwrap())).unwrap_or("".to_string()),
                                        seg 
                                        )
                                    )
                                )
                            }
                            Err(_) => None,
                        }
                    })
                ));

        Box::new(spawning_iterator.chain(segment_exchange_iterator))
    }
}

impl SpawnForMaintenanceAndPathExchange {
    fn segments<'a>(
        &'a self,
        provider: VehicleIdx,
        schedule: &'a Schedule,
    ) -> impl Iterator<Item = Segment> + 'a {
        let threshold = match self.overhead_threshold {
            None => Duration::ZERO,
            Some(d) => d,
        };
        let tour = schedule
            .tour_of(provider)
            .expect("provider not in schedule");
        // all non-depot nodes of provider's tour might be the start of a segment
        tour.all_non_depot_nodes_iter().enumerate()
        // only take nodes with enough preceding overhead:
        .filter(move |(_,n)| schedule.is_dummy(provider) || tour.preceding_overhead(*n).unwrap() >= threshold)
        .flat_map(move |(i,seg_start)|
            // all non-depot nodes (after the start) could be the end of the segment
            tour.all_non_depot_nodes_iter().skip(i)
            // only take nodes with enough subsequent overhead:
            .filter(move |n| schedule.is_dummy(provider) || tour.subsequent_overhead(*n).unwrap() >= threshold)
            // only take the nodes such that the segment is not longer than the threshold
            .take_while(move |seg_end| self.segment_length_limit.is_none()
                        || self.network.node(*seg_end).end_time() - self.network.node(seg_start).start_time() <= self.segment_length_limit.unwrap())
            // if provider is a real vehicle, add the last_node as segment end. (note that is not taken twice as EndNodes
            // end at time Infinity
            .chain(iter::once(schedule.tour_of(provider).unwrap().last_node())
                   .filter(move |_n| self.segment_length_limit.is_some()))
            // create the segment
            .map(move |seg_end| Segment::new(seg_start, seg_end))
        )
        // test whether the segment can be removed
        .filter(move |seg| schedule.tour_of(provider).unwrap().check_removable(*seg).is_ok())
    }

    fn dummy_and_real_vehicles<'a>(
        &'a self,
        schedule: &'a Schedule,
    ) -> impl Iterator<Item = VehicleIdx> + 'a {
        schedule.dummy_iter().chain(schedule.vehicles_iter_all())
    }

    fn real_and_dummy_vehicles<'a>(
        &'a self,
        schedule: &'a Schedule,
    ) -> impl Iterator<Item = VehicleIdx> + 'a {
        schedule.vehicles_iter_all().chain(schedule.dummy_iter())
    }
}
