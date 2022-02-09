use std::fmt;
use crate::distance::Distance;
use crate::time::Duration;
use crate::locations::Locations;
use crate::network::Network;
use crate::base_types::{NodeId,UnitId};

use itertools::Itertools;

pub(crate) struct Tour {
    unit: UnitId,
    nodes: Vec<NodeId>
}

impl Tour {
    pub(crate) fn length(&self, loc: &Locations, nw: &Network) -> Distance {
        let service_length: Distance = self.nodes.iter().map(|&n| nw.node(n).length()).sum();

        let dead_head_length = self.nodes.iter().tuple_windows().map(
            |(&a,&b)| loc.distance(nw.node(a).end_location(),nw.node(b).start_location())).sum();
        service_length + dead_head_length
    }

    pub(crate) fn travel_time(&self, loc: &Locations, nw: &Network) -> Duration {
        let service_tt: Duration = self.nodes.iter().map(|&n| nw.node(n).travel_time()).sum();
        let dead_head_tt = self.nodes.iter().tuple_windows().map(
            |(&a,&b)| loc.travel_time(nw.node(a).end_location(), nw.node(b).start_location())).sum();
        service_tt + dead_head_tt
    }

    pub(crate) fn print(&self, loc: &Locations, nw: &Network) {
        println!("tour with {} of length {} and travel time {}:", self.nodes.len(), self.length(loc, nw), self.travel_time(loc, nw));
        for node in self.nodes.iter() {
            println!("\t\t* {}", node);
        }
    }
}

impl Tour {
    pub(super) fn new(unit: UnitId, nodes: Vec<NodeId>) -> Tour {
        Tour{unit, nodes}
    }
}


impl fmt::Display for Tour {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "tour of {} with {} nodes", self.unit, self.nodes.len())
    }
}
