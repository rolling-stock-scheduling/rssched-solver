use super::swaps::{PathExchange, Swap};
use sbb_model::base_types::VehicleId;
use sbb_model::network::Network;
use sbb_solution::path::Segment;
use sbb_solution::Schedule;
use std::sync::Arc;
use time::Duration;

use std::iter;

/// Computes for a given schedule all Swaps in the neighborhood.
pub(crate) trait SwapFactory: Clone + Send {
    fn create_swap_iterator<'a>(
        &'a self,
        schedule: &'a Schedule,
    ) -> Box<dyn Iterator<Item = Box<dyn Swap + Send>> + Send + 'a>;
}

///////////////////////////////////////////////////////////
////////////////// LimitedExchanges ///////////////////////
///////////////////////////////////////////////////////////

/// Takes all PathExchanges where the segment time length is smaller than the threshold.
/// The length of a segment is measured from the start_time of the first node to the end_time of
/// the last node.
#[derive(Clone)]
pub(crate) struct LimitedExchanges {
    segment_length_limit: Option<Duration>,
    overhead_threshold: Option<Duration>,
    only_dummy_provider: bool,
    network: Arc<Network>,
}

impl LimitedExchanges {
    pub(crate) fn new(
        segment_length_limit: Option<Duration>,
        overhead_threshold: Option<Duration>,
        only_dummy_provider: bool,
        network: Arc<Network>,
    ) -> LimitedExchanges {
        LimitedExchanges {
            segment_length_limit,
            overhead_threshold,
            only_dummy_provider,
            network,
        }
    }
}

impl SwapFactory for LimitedExchanges {
    fn create_swap_iterator<'a>(
        &'a self,
        schedule: &'a Schedule,
    ) -> Box<dyn Iterator<Item = Box<dyn Swap + Send>> + Send + 'a> {
        let providers: Vec<_> = if self.only_dummy_provider {
            schedule.dummy_iter().collect()
        } else {
            self.dummy_and_real_vehicles(schedule).collect()
        };
        Box::new(
            // as provider first take dummies then real Vehicles:
            providers.into_iter().flat_map(move |provider|
                // create segment of provider's tour
                self.segments(provider, schedule)
                .flat_map(move |seg|
                    // as receiver first take the real Vehicles then the dummies
                    self.real_and_dummy_vehicles(schedule)
                    // skip provider as receiver
                    .filter(move |&u| u != provider)
                    // crate the swap
                    .map(move |receiver| -> Box<dyn Swap + Send> {
                        Box::new(PathExchange::new(seg, provider, receiver))
                    })
                )),
        )
    }
}

impl LimitedExchanges {
    fn segments<'a>(
        &'a self,
        provider: VehicleId,
        schedule: &'a Schedule,
    ) -> impl Iterator<Item = Segment> + 'a {
        let threshold = match self.overhead_threshold {
            None => Duration::zero(),
            Some(d) => d,
        };
        let tour = schedule
            .tour_of(provider)
            .expect("provider not in schedule");
        // all nodes of provider's tour might be the start of a segment
        tour.all_nodes_iter().enumerate()
        // only take nodes with enough preceding overhead:
        .filter(move |(_,n)| schedule.is_dummy(provider) || tour.preceding_overhead(*n).unwrap() >= threshold)
        .flat_map(move |(i,seg_start)|
            // all nodes (after the start) could be the end of the segment
            tour.all_nodes_iter().skip(i)
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
        .filter(move |seg| schedule.tour_of(provider).unwrap().removable(*seg))
    }

    fn dummy_and_real_vehicles<'a>(
        &'a self,
        schedule: &'a Schedule,
    ) -> impl Iterator<Item = VehicleId> + 'a {
        schedule.dummy_iter().chain(schedule.vehicles_iter())
    }

    fn real_and_dummy_vehicles<'a>(
        &'a self,
        schedule: &'a Schedule,
    ) -> impl Iterator<Item = VehicleId> + 'a {
        schedule.vehicles_iter().chain(schedule.dummy_iter())
    }
}
