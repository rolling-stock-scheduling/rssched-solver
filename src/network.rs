mod nodes;
use nodes::Node;

use crate::time::Time;
use std::fmt;

pub(crate) struct Network {
    nodes: Vec<Node>,
}

impl Network{
    pub(crate) fn initialize() -> Network {

        let mut nodes: Vec<nodes::Node> = Vec::new();
        nodes.push(Node::create_service_node(0,1,Time::new("2021-12-23 21:56"),Time::new("2021-12-23 22:56"),200));
        nodes.push(Node::create_maintenance_node(0, Time::new("2021-02-23 21:56"), Time::new("2021-12-23 21:56") ));
        let (start, end) = Node::create_vehicle_nodes(1, 0, Time::new("2021-12-10 08:00"), 2, Time::new("2021-12-26 00:00"));
        nodes.push(start);
        nodes.push(end);

        println!("{}", Time::new("2021-12-23 21:56")<Time::new("2021-12-23 22:56"));
        Network{nodes}

    }
}

impl fmt::Display for Network {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"** Network with {} nodes:\n", self.nodes.len())?;
        for (i,v) in self.nodes.iter().enumerate() {
            write!(f,"\t{}: {}\n", i, v)?;
        }
        Ok(())
    }
}
