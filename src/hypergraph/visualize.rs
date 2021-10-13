use core::fmt::Display;
use std::rc::Rc;

use crate::{traits::HypergraphClass, Hypergraph};

pub struct DotFormatter<N, E, H, L> {
    pub edge: Rc<dyn Fn(&Vec<usize>, &E) -> String>,
    pub node: Rc<dyn Fn(&Vec<usize>, &N) -> String>,
    pub hypergraph: Rc<dyn Fn(&Vec<usize>, &Option<H>) -> String>,
    pub link: Rc<dyn Fn(&Vec<usize>, &Option<L>) -> String>,
}

/// # Visualize
///
/// Visualize hypergraphs.
impl<N, E, H, L, Ty: HypergraphClass> Hypergraph<N, E, H, L, Ty> {
    /// Transforms into a [dot language](https://graphviz.org/doc/info/lang.html) representation, from Graphviz.
    ///
    /// Hyperedges are represented as nodes without borders.
    pub fn as_dot(&self, formatter: impl Into<Option<DotFormatter<N, E, H, L>>>) -> String {
        self.as_dot_impl(vec![], &formatter.into())
    }
    fn as_dot_impl(
        &self,
        pre_id: Vec<usize>,
        formatter_option: &Option<DotFormatter<N, E, H, L>>,
    ) -> String {
        let mut dot = String::new();
        if self.class().is_main() {
            dot.push_str("strict digraph ")
        } else if self.class().is_sub() {
            dot.push_str("subgraph cluster ") // shows as cluster, if supported
        }
        dot.push_str("{\n");
        // Hypergraph value
        if let Some(formatter) = formatter_option {
            dot += &format!(
                "\tlabel = \"{}\";\n",
                (formatter.hypergraph)(&pre_id, self.value())
            );
        }

        // Nodes
        let raw_nodes = self.raw_nodes();
        for post_id in raw_nodes.keys() {
            let mut id = pre_id.clone();
            id.push(*post_id);
            let label = match formatter_option {
                None => format!("{:?}", id),
                Some(formatter) => (formatter.node)(&id, &raw_nodes[post_id].0),
            };
            dot += &format!("\t\"{:?}\" [label=\"{}\"];\n", &id, label);
        }

        // Edges
        let raw_edges = self.raw_edges();
        for post_id in raw_edges.keys() {
            let mut id = pre_id.clone();
            id.push(*post_id);
            let label = match formatter_option {
                None => format!("{:?}", id),
                Some(formatter) => (formatter.edge)(&id, &raw_edges[post_id].0),
            };
            dot += &format!("\t\"{:?}\" [style = dotted, label=\"{}\"];\n", &id, label);
        }

        // Links
        let raw_links = self.raw_links();
        for post_id in raw_links.keys() {
            let mut id = pre_id.clone();
            id.push(*post_id);
            let link_full = &raw_links[post_id];
            let label = match formatter_option {
                None => format!("{:?}", id),
                Some(formatter) => (formatter.link)(&id, &link_full.0),
            };
            dot += &format!(
                "\t\"{:?}\" -> \"{:?}\" [label=\"{}\"];\n",
                &link_full.1, &link_full.2, label
            );
        }

        // Subhypergraphs
        let raw_hypergraphs = self.raw_hypergraphs();
        for post_id in raw_hypergraphs.keys() {
            let mut id = pre_id.clone();
            id.push(*post_id);
            let hypergraph_full = &raw_hypergraphs[post_id];
            dot += &hypergraph_full.0.as_dot_impl(id, formatter_option);
        }

        dot.push_str("}\n");
        dot
    }
}

impl<'a, N, E, H, L, Ty> Into<tabbycat::Graph<'a>> for &'a Hypergraph<N, E, H, L, Ty>
where
    H: Display,
{
    fn into(self) -> tabbycat::Graph<'a> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_dot() {
        let mut h = Hypergraph::<&str, &str, &str, &str>::new();
        h.add_node("zero", []).unwrap();
        h.add_node("one", []).unwrap();
        h.add_edge([0], [1], "two", []).unwrap();
        h.add_hypergraph("five", []).unwrap();
        h.add_node("six", [5]).unwrap();
        h.add_node("seven", [5]).unwrap();
        h.add_edge([5, 0], [5, 1], "eight", [5]).unwrap();
        h.add_link([2], [5, 0], "eleven", []).unwrap();
        h.add_hypergraph("twelve", [5]).unwrap();
        h.add_node("thirteen", [5, 5]).unwrap();

        let formatter = DotFormatter {
            edge: Rc::new(|_, e: &&str| e.to_string()),
            node: Rc::new(|_, n: &&str| n.to_string()),
            hypergraph: Rc::new(|_, h: &Option<&str>| match h {
                None => "?".to_string(),
                Some(v) => v.to_string(),
            }),
            link: Rc::new(|_, l: &Option<&str>| match l {
                None => "?".to_string(),
                Some(v) => v.to_string(),
            }),
        };
        println!("{}", h.as_dot(formatter));

        let formatter = DotFormatter {
            edge: Rc::new(|_, e: &&str| e.to_string()),
            node: Rc::new(|_, n: &&str| n.to_string()),
            hypergraph: Rc::new(|_, h: &Option<&str>| match h {
                None => "?".to_string(),
                Some(v) => v.to_string(),
            }),
            link: Rc::new(|_, l: &Option<&str>| match l {
                None => "?".to_string(),
                Some(v) => v.to_string(),
            }),
        };
        assert_eq!("strict digraph {\n\tlabel = \"?\";\n\t\"[0]\" [label=\"zero\"];\n\t\"[1]\" [label=\"one\"];\n\t\"[2]\" [style = dotted, label=\"two\"];\n\t\"[0]\" -> \"[2]\" [label=\"?\"];\n\t\"[2]\" -> \"[1]\" [label=\"?\"];\n\t\"[2]\" -> \"[5, 0]\" [label=\"eleven\"];\nsubgraph cluster {\n\tlabel = \"five\";\n\t\"[5, 0]\" [label=\"six\"];\n\t\"[5, 1]\" [label=\"seven\"];\n\t\"[5, 2]\" [style = dotted, label=\"eight\"];\n\t\"[5, 0]\" -> \"[5, 2]\" [label=\"?\"];\n\t\"[5, 2]\" -> \"[5, 1]\" [label=\"?\"];\nsubgraph cluster {\n\tlabel = \"twelve\";\n\t\"[5, 5, 0]\" [label=\"thirteen\"];\n}\n}\n}\n", &h.as_dot(formatter));
    }
}
