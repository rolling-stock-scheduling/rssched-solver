use std::fmt;
use crate::vehicle::Vehicle;
use crate::network::nodes::Node;
use crate::distance::Distance;
use crate::time::Duration;
use crate::locations::Locations;

use itertools::Itertools;

pub(crate) struct Tour<'a> {
    vehicle: &'a Vehicle,
    nodes: Vec<&'a Node<'a>>
}

impl<'a> Tour<'a> {
    pub(crate) fn length(&self, locations: &Locations) -> Distance {
        let service_length: Distance = self.nodes.iter().map(|&n| n.length()).sum();

        let dead_head_length = self.nodes.iter().tuple_windows().map(|(a,b)| locations.distance(a.end_location(),b.start_location())).sum();
        service_length + dead_head_length
    }

    pub(crate) fn travel_time(&self, locations: &Locations) -> Duration {
        let service_tt: Duration = self.nodes.iter().map(|&n| n.travel_time()).sum();
        let dead_head_tt = self.nodes.iter().tuple_windows().map(|(a,b)| locations.travel_time(a.end_location(), b.start_location())).sum();
        service_tt + dead_head_tt
    }

    pub(crate) fn print(&self, locations: &Locations) {
        println!("tour with {} of length {} and travel time {}:", self.nodes.len(), self.length(locations), self.travel_time(locations));
        for node in self.nodes.iter() {
            println!("\t\t* {}", node);
        }
    }
}

impl<'a> Tour<'a> {
    pub(super) fn new(vehicle: &'a Vehicle, nodes: Vec<&'a Node<'a>>) -> Tour<'a> {
        Tour{vehicle, nodes}
    }
}


impl<'a> fmt::Display for Tour<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "tour of {} with {} nodes", self.vehicle, self.nodes.len())
    }
}
