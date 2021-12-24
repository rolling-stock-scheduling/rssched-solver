mod nodes;
use nodes::Node;

use crate::time::Time;
use crate::location::Location;
use std::fmt;

pub(crate) struct Network<'a> {
    nodes: Vec<Node<'a>>,
}

impl<'a> Network<'a> {
    pub(crate) fn initialize(station: Vec<&'a Location>) -> Network {

        let mut nodes: Vec<nodes::Node> = Vec::new();
        nodes.push(Node::create_service_node(
                station[0],
                station[1],
                Time::new("2021-12-23 21:56"),
                Time::new("2021-12-23 22:56"),
                200));

        nodes.push(Node::create_maintenance_node(
                station[2],
                Time::new("2021-02-23 21:56"),
                Time::new("2021-12-23 21:56") ));

        let (start, end) = Node::create_vehicle_nodes(
            1,
            station[0],
            Time::new("2021-12-10 08:00"),
            station[1],
            Time::new("2021-12-26 00:00"));
        nodes.push(start);
        nodes.push(end);

        for n in nodes.iter() {
            println!("start_time: {}, end_time: {}, start_location: {}, end_location: {}", n.start_time(), n.end_time(), n.start_location(), n.end_location());
        }
        Network{nodes}



    }
}

impl<'a> fmt::Display for Network<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"** Network with {} nodes:\n", self.nodes.len())?;
        for (i,v) in self.nodes.iter().enumerate() {
            write!(f,"\t{}: {}\n", i, v)?;
        }
        Ok(())
    }
}
