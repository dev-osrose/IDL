use ::flat_ast::*;
use std::collections::{BTreeMap, HashSet};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
struct NodeId(usize);

#[derive(Debug)]
struct Edge {
    to: NodeId,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
    type_name: String
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
        let node = Node {
            id,
            name: name.to_owned(),
            type_,
            type_name: type_name.to_owned(),
            edges: BTreeMap::new(),
            color: Color::White,
            prune: true
        };
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

pub fn run(mut packet: Packet) -> Result<Packet, ::failure::Error> {
    use self::NodeType::*;

    let mut graph = Graph::new();

    for content in packet.contents() {
        use self::PacketContent::*;
        match content {
            Simple(ref s) => {
                use self::SimpleTypeContent::*;
                let mut enum_type = None;
                for content in s.contents() {
                    match content {
                        Restriction(ref r) => {
                            for content in r.contents() {
                                use self::RestrictionContent::*;
                                match content {
                                    Enumeration(_) => enum_type = Some(r.base()),
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                if let Some(enum_type) = enum_type {
                    graph.add_node(s.name(), TyEnum, enum_type);
                } else {
                    graph.add_node(s.name(), TySimple, s.name());
                }
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
            PacketContent::Element(ref e) => {
                graph.add_start_node(e.type_());
            },
            _ => {}
        }
    }

    graph.run();

    for content in packet.contents_mut() {
        match content {
            PacketContent::Complex(ref mut c) => {
            },
            PacketContent::Simple(ref mut s) => {
            },
            PacketContent::Element(ref mut e) => {
                let node = graph.get_node(e.type_());
                if let Ok(node) = node {
                    if graph.nodes[node.0].type_ == TyEnum {
                        e.set_enum_type(graph.nodes[node.0].type_name.clone());
                    }
                }
            },
            _ => {}
        }
    }

    Ok(packet)
}