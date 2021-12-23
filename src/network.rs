mod nodes;
use nodes::Node;

pub(crate) struct Network {
    nodes: Vec<Node>,
}

impl Network{
    pub(crate) fn initialize() -> Network {

        let mut nodes: Vec<nodes::Node> = Vec::new();
        nodes.push(Node::create_service_node(0,1,8,200));

        Network{nodes}
    }
}
