use ::flat_ast::*;
use std::collections::{BTreeMap, HashSet};
use heck::CamelCase;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
struct NodeId(usize);

#[derive(Debug, Clone)]
struct Edge {
    to: NodeId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NodeType {
    TySeq,
    TyChoice,
    TySimple,
    TyEnum,
    TyEmpty
}

#[derive(Debug, PartialEq)]
enum Color {
    White,
    Black,
    Grey
}

#[derive(Debug)]
struct Node {
    id: NodeId,
    name: String,
    edges: BTreeMap<u32, Edge>,
    color: Color,
    prune: bool,
    type_: NodeType,
    type_name: String,
    depth: u32
}

#[derive(Debug)]
struct Graph {
    nodes: Vec<Node>,
    start_nodes: HashSet<NodeId>
}

impl Graph {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            start_nodes: HashSet::new()
        }
    }

    fn add_start_node(&mut self, name: &str) {
        self.find_node(name).map(|start_node| self.start_nodes.insert(start_node));
    }

    fn find_node(&self, name: &str) -> Option<NodeId> {
        for node in &self.nodes {
            if node.name == *name {
                return Some(node.id);
            }
        }
        None
    }

    fn get_node(&self, name: &str) -> Result<NodeId, ::failure::Error> {
        let node = self.find_node(name);
        let node = match node {
            None => {
                return Err(::failure::err_msg(format!("SimpleType/ComplexType not found {}", name)));
            },
            Some(node) => node
        };
        Ok(node)
    }

    fn add_node(&mut self, name: &str, type_: NodeType, type_name: &str) {
        let id = NodeId(self.nodes.len());
        let mut node = Node {
            id,
            name: name.to_owned(),
            type_,
            type_name: type_name.to_owned(),
            edges: BTreeMap::new(),
            color: Color::White,
            prune: true,
            depth: 0,
        };
        if type_ == NodeType::TyEnum {
            node.prune = false;
        }
        self.nodes.push(node);
    }

    fn add_edges(&mut self, from_node: NodeId, elements: &[Element]) {
        for elem in elements {
            let node = self.find_node(elem.type_());
            if let Some(to) = node {
                let edge = Edge { to };
                let from_node = &mut self.nodes[from_node.0];
                from_node.edges.insert(elem.id(), edge);
            }
        }
    }

    fn add_edge(&mut self, from_node: NodeId, to: NodeId) {
        let edge = Edge { to };
        let from_node = &mut self.nodes[from_node.0];
        let highest = from_node.edges.keys().fold(0, |id, edge| if edge > &id { *edge } else { id }) + 1;
        from_node.edges.insert(highest, edge);
    }

    fn run(&mut self) {
        for start_node in self.start_nodes.clone() {
            self.run_passes(start_node);
        }
    }

    fn run_passes(&mut self, node_id: NodeId) {
        use self::Color::*;

        let mut adjacent_nodes = HashSet::new();
        let mut cycles = HashSet::new();
        let node_id = node_id.0;
        {
            let node = &mut self.nodes[node_id];
            if node.color != White {
                return;
            }
            node.color = Grey;
            node.prune = false;
        }

        {
            let node = &self.nodes[node_id];
            for (elem_id, edge) in &node.edges {
                let to = &self.nodes[edge.to.0];
                match to.color {
                    White => { adjacent_nodes.insert(to.id); },
                    Grey => { cycles.insert(*elem_id); },
                    _ => {}
                }
            }
        }

        {
            let node = &mut self.nodes[node_id];
            for (elem_id, _edge) in node.edges.iter_mut() {
                if cycles.contains(&elem_id) {
                    // TODO: do something here?
                }
            }
        }

        for adjacent in adjacent_nodes {
            self.run_passes(adjacent);
        }

        let node = &mut self.nodes[node_id];
        node.color = Black;
    }
}

fn visit(node: NodeId, graph: &mut Graph, depth: u32) {
    for edge in graph.nodes[node.0].edges.clone().values() {
        visit(edge.to, graph, depth + 1);
    }
    if graph.nodes[node.0].depth < depth {
        graph.nodes[node.0].depth = depth;
    }
}

fn remove_duplicates(mut packet: Packet) -> Result<Packet, ::failure::Error> {
    let mut names = std::collections::HashSet::new();
    packet.contents_mut().retain(|e|
        if let Some(ref name) = PacketContent::type_from_name(e) {
                names.insert(name.clone())
        } else {
            true
        });
    Ok(packet)
}

pub fn run(mut packet: Packet) -> Result<Packet, ::failure::Error> {
    use self::NodeType::*;

    packet = remove_duplicates(packet)?;

    let mut graph = Graph::new();

    for content in packet.contents() {
        use self::PacketContent::*;
        match content {
            Simple(ref s) => {
                graph.add_node(s.name(), TySimple, s.name());
            },
            Complex(ref c) => {
                use self::ComplexTypeContent::*;
                match c.content() {
                    Seq(_) => graph.add_node(c.name(), TySeq, c.name()),
                    Choice(_) => graph.add_node(c.name(), TyChoice, c.name()),
                    Empty => graph.add_node(c.name(), TyEmpty, c.name())
                }
            },
            _ => {}
        }
    }

    for content in packet.contents() {
        match content {
            PacketContent::Complex(ref c) => {
                use self::ComplexTypeContent::*;
                let node = graph.get_node(c.name())?;
                match c.content() {
                    Seq(seq) => graph.add_edges(node, seq.elements()),
                    Choice(choice) => graph.add_edges(node, choice.elements()),
                    _ => {}
                }
            },
            PacketContent::Simple(ref s) => {
                use self::SimpleTypeContent::*;
                match s.content() {
                    Restriction(ref r) => {
                        if let Ok(node) = graph.get_node(&r.base().to_owned().to_camel_case()) {
                            let to = graph.get_node(s.name()).unwrap();
                            graph.add_edge(to, node);
                        }
                    }
                }
            },
            PacketContent::Element(ref e) => {
                trace!("adding start node {}", e.type_());
                graph.add_start_node(&e.type_().to_owned().to_camel_case());
            }
        }
    }

    graph.run();

    // depth-first, post-traversal dependencies check
    for node in graph.start_nodes.clone().iter() {
        visit(*node, &mut graph, 0);
    }
    debug!("Graph {:?}", graph);

    let mut sequences = ::std::collections::HashMap::<String, Vec<Sequence>>::new();

    for content in packet.contents() {
        match content {
            PacketContent::Complex(c) => {
                match c.content() {
                    ComplexTypeContent::Choice(_) => {
                        for content in packet.contents().iter() {
                            match content {
                                PacketContent::Complex(c) => match c.content() {
                                    ComplexTypeContent::Seq(ccc) => {
                                        if sequences.contains_key(c.name()) {
                                            sequences.get_mut(c.name()).unwrap().push(ccc.clone());
                                        }
                                        sequences.insert(c.name().to_string(), vec![ccc.clone()]);
                                    },
                                    _ => {}
                                },
                                _ => {}
                            }
                        }
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }

    // pruning
    for node in &graph.nodes {
        if node.prune {
            trace!("pruning {}", node.name);
            packet.contents_mut().retain(|e| if let Some(ref name) = PacketContent::type_from_name(e) { *name != node.name } else { true });
        }
    }

    packet.contents_mut().sort_by(|a, b| {
        let node_a = match a {
            PacketContent::Complex(ref c) => graph.find_node(c.name()),
            PacketContent::Simple(ref s) => graph.find_node(s.name()),
            _ => None
        };
        let node_b = match b {
            PacketContent::Complex(ref c) => graph.find_node(c.name()),
            PacketContent::Simple(ref s) => graph.find_node(s.name()),
            _ => None
        };
        match (node_a, node_b) {
            (None, None) | (None, _) | (_, None) => ::std::cmp::Ordering::Equal,
            (Some(NodeId(a)), Some(NodeId(b))) => graph.nodes[b].depth.cmp(&graph.nodes[a].depth)
        }
    });

    Ok(packet)
}
