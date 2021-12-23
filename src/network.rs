mod nodes;
use nodes::Node;

use std::fmt;

pub(crate) struct Network {
    nodes: Vec<Node>,
}

impl Network{
    pub(crate) fn initialize() -> Network {

        let mut nodes: Vec<nodes::Node> = Vec::new();
        nodes.push(Node::create_service_node(0,1,8,14,200));
        nodes.push(Node::create_maintenance_node(0, 1400, 16));
        let (start, end) = Node::create_vehicle_nodes(1, 0, 6, 1, 23);
        nodes.push(start);
        nodes.push(end);

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
