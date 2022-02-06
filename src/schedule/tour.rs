use std::fmt;
use crate::vehicle::Vehicle;
use crate::network::nodes::Node;
use crate::distance::Distance;
use crate::location::DeadHeadMetrics;

use itertools::Itertools;

pub(crate) struct Tour<'a> {
    vehicle: &'a Vehicle,
    nodes: Vec<&'a Node<'a>>
}

impl<'a> Tour<'a> {
    pub(crate) fn length(&self, dhd: &DeadHeadMetrics) -> Distance {
        let service_length: Distance = self.nodes.iter().map(|&n| n.length()).sum();

        let dead_head_length = self.nodes.iter().tuple_windows().map(|(a,b)| dhd.dist(a.end_location(),b.start_location())).sum();
        service_length + dead_head_length

    }
}

impl<'a> Tour<'a> {
    pub(super) fn new(vehicle: &'a Vehicle, nodes: Vec<&'a Node<'a>>) -> Tour<'a> {
        Tour{vehicle, nodes}
    }
}


impl<'a> fmt::Display for Tour<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Tour of {} with {} nodes.", self.vehicle, self.nodes.len())
    }
}
