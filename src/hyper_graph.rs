use core::{
    fmt::{Display, Formatter},
    marker::PhantomData,
};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Marker for main hypergrpah
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Main;

/// Marker for sub hypergrpah
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Sub;

pub trait HyperGraphStructure {}

impl HyperGraphStructure for Main {}
impl HyperGraphStructure for Sub {}

/// Hyper graphs
///
/// - `usize`: id
/// - `N`: node
/// - `E`: hyperedge
/// - `L`: link (simple edge)
/// - `H`: hypergraph
//
// # Note
//
// You might want to change Vec by SmallVec in the future
// and let the user decide the storage capacity (ie. how many nested structures are there).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperGraph<N, E, L, H, Ty> {
    /// Value of the hypergraph as a whole.
    value: Option<H>,
    /// nodes, and their weight and neigbours (in absolute format)
    nodes: IndexMap<usize, (N, Vec<(Vec<usize>, Direction)>)>,
    /// edges, and their weight and neigbours (in absolute format)
    edges: IndexMap<usize, (E, Vec<(Vec<usize>, Direction)>)>, // This vector alsways has at least two elements
    /// links, and their weight
    links: IndexMap<usize, (L, Vec<((Vec<usize>, Vec<usize>), Direction)>)>,
    /// subhypergraps and their weight, content and neigbours
    hypergraphs: IndexMap<
        usize,
        (
            HyperGraph<usize, N, E, L, Sub>,
            Vec<(Vec<usize>, Direction)>,
        ),
    >,
    phantom: PhantomData<Ty>,
}

// #[derive(Clone, Default, Serialize, Deserialize)]
// pub struct SubHyperGraph<N, E, L, H> {
//     /// nodes, and their weight and neigbours (in absolute format)
//     nodes: IndexMap<usize, (N, Vec<(Vec<usize>, Direction)>)>,
//     /// edges, and their weight and neigbours (in absolute format)
//     edges: IndexMap<usize, (E, Vec<(Vec<usize>, Direction)>)>, // This vector alsways has at least two elements
//     /// links, and their weight
//     links: IndexMap<usize, (L, Vec<((Vec<usize>, Vec<usize>), Direction)>)>,
//     /// subhypergraps and their weight, content and neigbours (in absolute format)
//     hypergraphs: IndexMap<
//         usize,
//         (
//             H,
//             SubHyperGraph<usize, N, E, L>,
//             Vec<(Vec<usize>, Direction)>,
//         ),
//     >,
// }

/// Edge direction.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    /// An `Outgoing` edge is an outward link *from* the current element.
    Outgoing,
    /// An `Incoming` edge is an inbound link *to* the current element.
    Incoming,
}

impl Direction {
    /// Return the opposite `Direction`.
    pub fn opposite(self) -> Direction {
        match self {
            Direction::Outgoing => Direction::Incoming,
            Direction::Incoming => Direction::Outgoing,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Element<N, E, L, H> {
    /// A graph node.
    Node { value: N },
    /// A graph edge.
    Edge { value: E },
    /// A graph link.
    Link {
        source: Vec<usize>,
        target: Vec<usize>,
        value: L,
    },
    /// A hypergraph.
    HyperGraph { value: H },
}

#[derive(Debug, thiserror::Error, Clone)]
struct BuildError {
    faulty_location: Vec<usize>,
}

impl Display for BuildError {
    // add code here
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        todo!()
    }
}

/// # Build
///
/// A graph that can be extended with further nodes and edges
impl<N, E, L, H> HyperGraph<N, E, L, H, Main> {
    ///
    ///
    /// To identify the hypergraph this node will exist on, `location` is used.
    /// A value of `None` means the main hypergraph.
    ///
    /// # Errors
    ///
    /// If location does not correspond to a hypergraph.
    pub fn add_node<O>(&mut self, value: N, location: O) -> Result<Vec<usize>, BuildError>
    where
        O: Into<Option<Vec<usize>>>,
    {
        todo!()
    }

    pub fn add_edge<O>(
        &mut self,
        a: Vec<usize>,
        b: Vec<usize>,
        value: E,
        location: O,
    ) -> Result<Vec<usize>, BuildError>
    where
        O: Into<Option<Vec<usize>>>,
    {
        todo!()
    }

    pub fn add_link<O>(
        &mut self,
        a: Vec<usize>,
        b: Vec<usize>,
        value: E,
        location: O,
    ) -> Result<Vec<usize>, BuildError>
    where
        O: Into<Option<Vec<usize>>>,
    {
        todo!()
    }

    pub fn add_hypergraph<O>(&mut self, value: H, location: O) -> Result<Vec<usize>, BuildError>
    where
        O: Into<Option<Vec<usize>>>,
    {
        todo!()
    }
}

/// # Create
///
/// A graph that can be created.
impl<N, E, L, H, Ty: HyperGraphStructure> HyperGraph<N, E, L, H, Ty> {
    pub fn new() -> Self {
        let nodes = IndexMap::new();
        let edges = IndexMap::new();
        let links = IndexMap::new();
        let hypergraphs = IndexMap::new();
        HyperGraph {
            value: None,
            nodes,
            edges,
            links,
            hypergraphs,
            phantom: PhantomData,
        }
    }

    pub fn with_capacity(nodes: usize, edges: usize, links: usize, hypergraphs: usize) -> Self {
        let nodes = IndexMap::with_capacity(nodes);
        let edges = IndexMap::with_capacity(edges);
        let links = IndexMap::with_capacity(links);
        let hypergraphs = IndexMap::with_capacity(hypergraphs);
        HyperGraph {
            value: None,
            nodes,
            edges,
            links,
            hypergraphs,
            phantom: PhantomData,
        }
    }

    pub fn reserve(&mut self, additional: usize) -> &mut Self {
        todo!()
    }

    pub fn reserve_nodes(&mut self, additional: usize) -> &mut Self {
        todo!()
    }

    pub fn reserve_edges(&mut self, additional: usize) -> &mut Self {
        todo!()
    }

    pub fn reserve_links(&mut self, additional: usize) -> &mut Self {
        todo!()
    }

    pub fn reserve_hypergraphs(&mut self, additional: usize) -> &mut Self {
        todo!()
    }
}

/// # Getters and setters
///
/// Access node and edge weights (associated data).
impl<N, E, L, H, Ty: HyperGraphStructure> HyperGraph<N, E, L, H, Ty> {
    pub fn element_value(&self, id: Vec<usize>) -> Option<&Element<N, E, L, H>> {
        todo!()
    }

    pub fn node_value(&self, id: Vec<usize>) -> Option<&N> {
        todo!()
    }

    pub fn edge_value(&self, id: Vec<usize>) -> Option<&E> {
        todo!()
    }

    pub fn link_value(&self, id: Vec<usize>) -> Option<&L> {
        todo!()
    }
    pub fn hypergraph_value(&self, id: Vec<usize>) -> Option<&H> {
        todo!()
    }

    pub fn element_value_mut(&mut self, id: Vec<usize>) -> Option<&mut Element<N, E, L, H>> {
        todo!()
    }

    pub fn node_value_mut(&mut self, id: Vec<usize>) -> Option<&mut N> {
        todo!()
    }

    pub fn edge_value_mut(&mut self, id: Vec<usize>) -> Option<&mut E> {
        todo!()
    }

    pub fn link_value_mut(&mut self, id: Vec<usize>) -> Option<&mut L> {
        todo!()
    }
    pub fn hypergraph_value_mut(&mut self, id: Vec<usize>) -> Option<&mut H> {
        todo!()
    }

    pub fn set_element_value(
        &mut self,
        id: Vec<usize>,
        new_value: N,
    ) -> Option<Element<N, E, L, H>> {
        todo!()
    }

    pub fn set_node_value(&mut self, id: Vec<usize>, new_value: N) -> Option<N> {
        todo!()
    }

    pub fn set_edge_value(&mut self, id: Vec<usize>, new_value: E) -> Option<E> {
        todo!()
    }

    pub fn set_link_value(&mut self, id: Vec<usize>, new_value: L) -> Option<L> {
        todo!()
    }

    pub fn set_hypergraph_value(&mut self, id: Vec<usize>, new_value: H) -> Option<H> {
        todo!()
    }
}

/// # Clear
///
/// A graph that can be cleared.
impl<N, E, L, H> HyperGraph<N, E, L, H, Main> {
    pub fn clear(&mut self) -> &mut Self {
        todo!()
    }

    pub fn clear_nodes(&mut self) -> &mut Self {
        todo!()
    }

    pub fn clear_edges(&mut self) -> &mut Self {
        todo!()
    }

    pub fn clear_links(&mut self) -> &mut Self {
        todo!()
    }

    pub fn clear_hypergraphs(&mut self) -> &mut Self {
        todo!()
    }
}

/// # Informations
///
/// Various information about the hypergraph.
impl<N, E, L, H, Ty: HyperGraphStructure> HyperGraph<N, E, L, H, Ty> {
    pub fn capacity(&self) -> () {
        todo!()
    }

    pub fn contains_element(&self, id: Vec<usize>) -> bool {
        todo!()
    }

    pub fn contains_node(&self, id: Vec<usize>) -> bool {
        todo!()
    }

    pub fn contains_edge(&self, id: Vec<usize>) -> bool {
        todo!()
    }

    pub fn contains_link(&self, id: Vec<usize>) -> bool {
        todo!()
    }

    pub fn contains_hypegraph(&self, id: Vec<usize>) -> bool {
        todo!()
    }
}

/// # Find
///
/// Find elements by value.
impl<N, E, L, H, Ty: HyperGraphStructure> HyperGraph<N, E, L, H, Ty> {
    pub fn find_element_by_value(&self, value: &Element<N, E, L, H>) -> Option<Vec<usize>> {
        todo!()
    }

    pub fn find_node_by_value(&self, value: &N) -> Option<Vec<usize>> {
        todo!()
    }

    pub fn find_edge_by_value(&self, value: &E) -> Option<Vec<usize>> {
        todo!()
    }

    pub fn find_link_by_value(&self, value: &L) -> Option<Vec<usize>> {
        todo!()
    }

    pub fn find_hypergraph_by_value(&self, value: &H) -> Option<Vec<usize>> {
        todo!()
    }
}
