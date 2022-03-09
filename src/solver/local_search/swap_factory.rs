use super::swaps::{Swap, PathExchange};
use crate::schedule::Schedule;
use crate::schedule::path::Segment;

/// Computes for a given schedule all Swaps in the neighborhood.
pub(crate) trait SwapFactory {
    fn create_swap_iterator<'a>(&'a self, schedule: &'a Schedule) -> Box<dyn Iterator<Item = Box<dyn Swap>>+ 'a>;
}



///////////////////////////////////////////////////////////
///////////////////// AllExchanges ////////////////////////
///////////////////////////////////////////////////////////

pub(crate) struct AllExchanges {
}

impl AllExchanges {
    pub(crate) fn new() -> AllExchanges {
        AllExchanges{}
    }
}

impl SwapFactory for AllExchanges {
    fn create_swap_iterator<'a> (&'a self, schedule : &'a Schedule) -> Box<dyn Iterator<Item = Box<dyn Swap>> + 'a> {
        Box::new(
            schedule.dummy_iter().chain(schedule.real_units_iter())
            .flat_map(move |provider|
                schedule.tour_of(provider).nodes_iter().enumerate()
                .flat_map(move |(i,seg_start)| schedule.tour_of(provider).nodes_iter().skip(i).map(move |seg_end| Segment::new(*seg_start, *seg_end)))
                .filter(move |seg| schedule.tour_of(provider).removable(*seg))
                .flat_map(move |seg|
                    schedule.real_units_iter().chain(schedule.dummy_iter())
                    .filter(move |&u| u != provider && schedule.conflict(seg, u).is_ok())
                    .map(move |receiver| -> Box<dyn Swap> {
                        Box::new(PathExchange::new(seg, provider, receiver))
                    })
                )
            )
        )
    }
}

