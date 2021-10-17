use core::fmt::{Debug, Display};
use std::{fs, io, io::Write, process, rc::Rc};

use crate::{traits::HypergraphClass, Hypergraph};

pub struct DotFormatter<N, E, H, L> {
    pub edge: Rc<dyn Fn(&Vec<usize>, &E) -> String>,
    pub node: Rc<dyn Fn(&Vec<usize>, &N) -> String>,
    pub hypergraph: Rc<dyn Fn(&Vec<usize>, &Option<H>) -> String>,
    pub link: Rc<dyn Fn(&Vec<usize>, &Option<L>) -> String>,
}

impl<N, E, H, L> DotFormatter<N, E, H, L> {
    /// Creates a new `DotFormatter` that forwards the `Debug` implementation in all fields
    ///
    /// Values `None` are left blank.
    pub fn debug() -> Self
    where
        N: Debug,
        E: Debug,
        H: Debug,
        L: Debug,
    {
        let mut dotformatter = Self::new();
        dotformatter
            .set_edge(|_, edge| format!("{:?}", edge))
            .set_hypergraph(|_, hypergraph_option| {
                if let Some(hypergraph) = hypergraph_option {
                    format!("{:?}", hypergraph)
                } else {
                    String::new()
                }
            })
            .set_link(|_, link_option| {
                if let Some(link) = link_option {
                    format!("{:?}", link)
                } else {
                    String::new()
                }
            })
            .set_node(|_, node| format!("{:?}", node));
        dotformatter
    }

    /// Creates a new `DotFormatter` that forwards the `Display` implementation in all fields.
    ///
    /// Values `None` are left blank.
    pub fn display() -> Self
    where
        N: Display,
        E: Display,
        H: Display,
        L: Display,
    {
        let mut dotformatter = Self::new();
        dotformatter
            .set_edge(|_, edge| format!("{}", edge))
            .set_hypergraph(|_, hypergraph_option| {
                if let Some(hypergraph) = hypergraph_option {
                    format!("{}", hypergraph)
                } else {
                    String::new()
                }
            })
            .set_link(|_, link_option| {
                if let Some(link) = link_option {
                    format!("{}", link)
                } else {
                    String::new()
                }
            })
            .set_node(|_, node| format!("{}", node));
        dotformatter
    }

    /// Creates a new `DotFormatter` with default settings.
    pub fn new() -> Self {
        Self::default()
    }
    pub fn set_edge<F: 'static + Fn(&Vec<usize>, &E) -> String>(
        &mut self,
        edge_formatter: F,
    ) -> &mut Self {
        self.edge = Rc::new(edge_formatter);
        self
    }

    pub fn set_hypergraph<F: 'static + Fn(&Vec<usize>, &Option<H>) -> String>(
        &mut self,
        hypergraph_formatter: F,
    ) -> &mut Self {
        self.hypergraph = Rc::new(hypergraph_formatter);
        self
    }

    pub fn set_link<F: 'static + Fn(&Vec<usize>, &Option<L>) -> String>(
        &mut self,
        link_formatter: F,
    ) -> &mut Self {
        self.link = Rc::new(link_formatter);
        self
    }

    pub fn set_node<F: 'static + Fn(&Vec<usize>, &N) -> String>(
        &mut self,
        node_formatter: F,
    ) -> &mut Self {
        self.node = Rc::new(node_formatter);
        self
    }
}

impl<N, E, H, L> Default for DotFormatter<N, E, H, L> {
    /// Creates a new `DotFormatter`.
    ///
    /// The label of every element is its `id`.
    fn default() -> Self {
        DotFormatter {
            edge: Rc::new(|id, _| format!("{:?}", id)),
            node: Rc::new(|id, _| format!("{:?}", id)),
            hypergraph: Rc::new(|id, _| format!("{:?}", id)),
            link: Rc::new(|id, _| format!("{:?}", id)),
        }
    }
}

/// # Visualize
///
/// Visualize hypergraphs.
impl<N, E, H, L, Ty: HypergraphClass> Hypergraph<N, E, H, L, Ty> {
    /// Transforms into a [dot language](https://graphviz.org/doc/info/lang.html) representation, from Graphviz.
    ///
    /// Hyperedges are represented as nodes without borders.
    pub fn as_dot<F>(&self, formatter: F) -> String
    where
        F: Into<Option<DotFormatter<N, E, H, L>>>,
    {
        self.as_dot_impl(vec![], &formatter.into())
    }
    fn as_dot_impl(
        &self,
        pre_id: Vec<usize>,
        formatter_option: &Option<DotFormatter<N, E, H, L>>,
    ) -> String {
        let mut dot = String::new();
        if self.class().is_main() {
            dot.push_str("digraph \"[]\" ")
        } else if self.class().is_sub() {
            dot += &format!("subgraph \"cluster_{:?}\" ", pre_id) // shows as cluster, if supported
        }
        dot.push_str("{\n\tcompound = true;\n");
        // Hypergraph value
        match formatter_option {
            Some(formatter) => {
                dot += &format!(
                    "\tlabel = \"{}\";\n",
                    (formatter.hypergraph)(&pre_id, self.value())
                );
            }
            None => {
                dot += &format!("\tlabel = \"{:?}\";\n", pre_id);
            }
        }
        // Invisible node to refer to the hypergraph in edges
        dot += &format!(
            "\t\"{:?}\" [label = \"\", height = 0, width = 0, style = invisible];\n",
            pre_id
        );

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
            let mut atributes = String::new();
            atributes += &format!("label = \"{}\"", label);
            // Recall: Links in a hypergraph can only refer to elements inside that hypergraph.
            let local_source: Vec<_> = link_full.1.clone().into_iter().skip(pre_id.len()).collect();
            if self.contains_subhypergraph(&local_source) {
                atributes += &format!(", ltail = \"cluster_{:?}\"", link_full.1);
            }
            let local_target: Vec<_> = link_full.2.clone().into_iter().skip(pre_id.len()).collect();
            if self.contains_subhypergraph(&local_target) {
                atributes += &format!(", lhead = \"cluster_{:?}\"", link_full.2);
            }
            dot += &format!(
                "\t\"{:?}\" -> \"{:?}\" [{}];\n",
                &link_full.1, &link_full.2, atributes
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

    /// Saves the output of [`as_dot`] and draws and saves the graph as a svg file.
    ///
    /// The files are named through `file_name`.
    ///
    /// This is just a shorthand for running the command [`dot`] of Graphviz in the result of the [`as_dot`] method
    /// and saving all files.
    ///
    /// # Requirements
    ///
    /// [`Graphviz`] needs to be install in your system. In particular, [`dot`] must be a command accessible from PATH.
    ///
    /// # Safety
    ///
    /// As this calls an external command (`dot`), there is no safety guarantee.
    ///
    /// [`dot`]: https://graphviz.org/doc/info/command.html
    /// [`Graphviz`]: https://graphviz.org/
    pub fn draw<F>(&self, formatter: F, file_name: impl Display) -> io::Result<process::Child>
    where
        F: Into<Option<DotFormatter<N, E, H, L>>>,
    {
        fs::create_dir_all("target/ferret_hypergraph/dot/")?;
        let dot_path = format!("target/ferret_hypergraph/dot/{}.dot", file_name);
        let mut dot_file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&dot_path)?;
        write!(dot_file, "{}", self.as_dot(formatter))?;

        fs::create_dir_all("target/ferret_hypergraph/svg/")?;
        let child = process::Command::new("dot")
            .arg("-Tsvg")
            .arg(&dot_path)
            .args(&[
                "-o",
                &format!("target/ferret_hypergraph/svg/{}.svg", file_name),
            ])
            .spawn()
            .expect("failed running graphviz dot. Is graphviz installed?");

        Ok(child)
    }

    /// On top of applying the [`draw`] method, it (asynchroniously) renders the svg file into a png file
    /// and opens it (using [`emulsion`]) for quick inspection.
    ///
    /// This is just a shorthand for running the method [`draw`], then commands [`resvg`] and [`emulsion`].
    ///
    /// # Requirements
    ///
    /// - [`resvg`] needs to be install in your system.
    /// - [`emulsion`] needs to be install in your system.
    ///
    /// # Safety
    ///
    /// This calls an external commands ([`resvg`] and [`emulsion`]). There is no safety guarantee.
    ///
    /// [`resvg`]: https://crates.io/crates/resvg
    /// [`emulsion`]: https://github.com/ArturKovacs/emulsion
    pub fn draw_and_show<F>(
        &self,
        formatter: F,
        file_name: impl Display,
    ) -> io::Result<process::Child>
    where
        F: Into<Option<DotFormatter<N, E, H, L>>>,
    {
        self.draw(formatter, &file_name)?
            .wait()
            .expect("dot failed to run.");
        fs::create_dir_all("target/ferret_hypergraph/svg/")?;
        fs::create_dir_all("target/ferret_hypergraph/png/")?;

        process::Command::new("resvg")
            .arg(&format!("target/ferret_hypergraph/svg/{}.svg", file_name))
            .arg(&format!("target/ferret_hypergraph/png/{}.png", file_name))
            .spawn()
            .expect("failed running resvg to transform svg to png format. Is resvg installed?")
            .wait()
            .expect("resvg failed to run.");

        let child = process::Command::new("emulsion")
            .arg(&format!("target/ferret_hypergraph/png/{}.png", file_name))
            .spawn()
            .expect("failed running emulsion to open png. Is emulsion installed?");

        Ok(child)
    }

    /// Renders the hypergraph as a png (using [`dot`])
    /// and opens it (using [`emulsion`]) for quick inspection.
    ///
    /// This is the fastest way to visualize a hypergraph.
    ///
    /// This is just a shorthand for running the commands [`dot`] to generate a png file and open it with [`emulsion`].
    ///
    /// # Requirements
    ///
    /// - [`dot`] needs to be install in your system.
    /// - [`emulsion`] needs to be install in your system.
    ///
    /// # Safety
    ///
    /// As this calls external commands, here is no safety guarantee.
    ///
    /// [`dot`]: https://graphviz.org/doc/info/command.html
    /// [`emulsion`]: https://github.com/ArturKovacs/emulsion
    pub fn show<F>(&self, formatter: F, file_name: impl Display) -> io::Result<process::Child>
    where
        F: Into<Option<DotFormatter<N, E, H, L>>>,
    {
        fs::create_dir_all("target/ferret_hypergraph/png/")?;

        let mut child = process::Command::new("dot")
            .arg("-Tpng")
            .stdin(process::Stdio::piped())
            .args(&[
                "-o",
                &format!("target/ferret_hypergraph/png/{}.png", file_name),
            ])
            .spawn()
            .expect("failed running graphviz dot. Is graphviz installed?");

        child
	        .stdin
	        .as_mut()
	        .unwrap()
	        .write(self.as_dot(formatter).as_bytes())
	        .expect("Writing failed in child process. We could not pass the dot representation of the hypergraph to dot.");
        child.wait()
            .expect("failed running graphviz dot. If graphviz is running well in your computer, contact us!");

        let child = process::Command::new("emulsion")
            .arg(&format!("target/ferret_hypergraph/png/{}.png", file_name))
            .spawn()
            .expect("failed running emulsion to open png. Is emulsion installed?");

        Ok(child)
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

        let mut formatter = DotFormatter::new();
        formatter
            .set_edge(|_, e: &&str| e.to_string())
            .set_node(|_, n: &&str| n.to_string())
            .set_hypergraph(|_, h: &Option<&str>| match h {
                None => "?".to_string(),
                Some(v) => v.to_string(),
            })
            .set_link(|_, l: &Option<&str>| match l {
                None => "?".to_string(),
                Some(v) => v.to_string(),
            });
        println!("{}", h.as_dot(formatter));

        let mut formatter = DotFormatter::new();
        formatter
            .set_edge(|_, e: &&str| e.to_string())
            .set_node(|_, n: &&str| n.to_string())
            .set_hypergraph(|_, h: &Option<&str>| match h {
                None => "?".to_string(),
                Some(v) => v.to_string(),
            })
            .set_link(|_, l: &Option<&str>| match l {
                None => "?".to_string(),
                Some(v) => v.to_string(),
            });
        assert_eq!(
        	&h.as_dot(formatter),
        	"digraph \"[]\" {\n\tcompound = true;\n\tlabel = \"?\";\n\t\"[]\" [label = \"\", height = 0, width = 0, style = invisible];\n\t\"[0]\" [label=\"zero\"];\n\t\"[1]\" [label=\"one\"];\n\t\"[2]\" [style = dotted, label=\"two\"];\n\t\"[0]\" -> \"[2]\" [label = \"?\"];\n\t\"[2]\" -> \"[1]\" [label = \"?\"];\n\t\"[2]\" -> \"[5, 0]\" [label = \"eleven\"];\nsubgraph \"cluster_[5]\" {\n\tcompound = true;\n\tlabel = \"five\";\n\t\"[5]\" [label = \"\", height = 0, width = 0, style = invisible];\n\t\"[5, 0]\" [label=\"six\"];\n\t\"[5, 1]\" [label=\"seven\"];\n\t\"[5, 2]\" [style = dotted, label=\"eight\"];\n\t\"[5, 0]\" -> \"[5, 2]\" [label = \"?\"];\n\t\"[5, 2]\" -> \"[5, 1]\" [label = \"?\"];\nsubgraph \"cluster_[5, 5]\" {\n\tcompound = true;\n\tlabel = \"twelve\";\n\t\"[5, 5]\" [label = \"\", height = 0, width = 0, style = invisible];\n\t\"[5, 5, 0]\" [label=\"thirteen\"];\n}\n}\n}\n",
        	);
    }
}
