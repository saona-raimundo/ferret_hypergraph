use core::fmt::Debug;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{direction::Direction, elements::*, errors, traits::HypergraphClass};

mod add;
mod classes;
mod clear;
mod find;
mod get;
mod remove;
mod set;
mod visualize;

pub use classes::{Main, Sub};

/// Directed-hyper-multi-graphs.
///
/// Directed graphs allow connections to have a direction.
/// Hyper-graphs allow edges to connect more than two elements.
/// Multi-graphs allow more than one connection between two elements.
/// `Hypergraph` is a directed-hyper-multi-graph that is also recursive:
/// it can contain another `Hypergraph` inside it
/// (with a marker `Sub` which restricts its methods).
///
/// # Data structure
///
/// In a nutshell, (hyper)edges are treated the same as nodes, while links take the role
/// of simple edges. Nodes and hypergraphs can be connected through (hyper)edges,
/// for which links are used in the middle.
/// `Hypergraph` is a multi-graph in two sense:
/// - There can be more than one (hyper)edge connecting two elements.
/// - There can be more than one link between a node or hypergraph and an edge.
///
/// # Type paramenters
///
/// - `E`: hyperedge
/// - `H`: hypergraph
/// - `L`: link (simple edge)
/// - `N`: node
/// - `Ty`: Main or sub hypergraph marker
///
/// # Indices
///
/// Indices are represented by `Vec<usize>` by default. They are stable except upon usage
/// of any method under [`Optimization`](#optimization) (like [`shrink_to_fit`]).
///
/// # Contents
///
/// - [`Add`](#add)
/// - [`Create`](#create)
/// - [`Clear`](#clear)
/// - [`Find`](#find)
/// - [`Get`](#get)
/// - [`Inform`](#inform)
/// - [`Optimization`](#optimization)
/// - [`Remove`](#remove)
/// - [`Set`](#set)
/// - [`Visualize`](#visualize)
//
// # Note
//
// You might want to change Vec by SmallVec in the future
// and let the user decide the storage capacity (ie. how many nested structures are there).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypergraph<N, E, H = (), L = (), Ty = Main> {
    /// Value of the hypergraph as a whole.
    value: Option<H>,
    /// nodes: their weight and links (in absolute format)
    nodes: IndexMap<usize, (N, Vec<(Vec<usize>, Direction)>)>,
    /// edges: weight and links (in absolute format)
    edges: IndexMap<usize, (E, Vec<(Vec<usize>, Direction)>)>, // This vector always has at least two elements
    /// links: weight, source and target ids (in absolute format)
    links: IndexMap<usize, (Option<L>, Vec<usize>, Vec<usize>)>, // Links have no neighbors
    /// subhypergraps: subhypergraph and links
    hypergraphs: IndexMap<usize, (Hypergraph<N, E, H, L, Sub>, Vec<(Vec<usize>, Direction)>)>,
    /// Counter for the next id when adding elements. It also serves as an upper bound on the number of elements.
    next_id: usize,
    /// Type (either Main or Sub)
    class: Ty,
}

/// Wrapper for ease of implementation.
///
/// It is not meant to be part of the public API.
//
// # Alternative
//
// Implement thorugh [enum_dispatch](https://crates.io/crates/enum_dispatch)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HypergraphEnum<O, S> {
    Original(O),
    Sub(S),
}

impl<O, S> HypergraphEnum<O, S> {
    pub fn is_original(&self) -> bool {
        matches!(self, HypergraphEnum::Original(_))
    }

    pub fn is_sub(&self) -> bool {
        matches!(self, HypergraphEnum::Sub(_))
    }
}

impl<'a, N, E, H, L, Ty>
    HypergraphEnum<&'a Hypergraph<N, E, H, L, Ty>, &'a Hypergraph<N, E, H, L, Sub>>
{
    pub fn contains(&self, id: impl AsRef<[usize]>) -> bool {
        match self {
            HypergraphEnum::Original(h) => h.contains(id),
            HypergraphEnum::Sub(h) => h.contains(id),
        }
    }
    pub fn links_of(
        &self,
        id: impl AsRef<[usize]>,
    ) -> Result<&'a Vec<(Vec<usize>, Direction)>, errors::GetError> {
        match self {
            HypergraphEnum::Original(h) => h.links_of(id),
            HypergraphEnum::Sub(h) => h.links_of(id),
        }
    }

    pub fn element_type(&self, id: impl AsRef<[usize]>) -> Result<ElementType, errors::GetError> {
        match self {
            HypergraphEnum::Original(h) => h.element_type(id),
            HypergraphEnum::Sub(h) => h.element_type(id),
        }
    }

    pub fn raw_edges(&self) -> &'a IndexMap<usize, (E, Vec<(Vec<usize>, Direction)>)> {
        match self {
            HypergraphEnum::Original(h) => h.raw_edges(),
            HypergraphEnum::Sub(h) => h.raw_edges(),
        }
    }

    pub fn raw_links(&self) -> &'a IndexMap<usize, (Option<L>, Vec<usize>, Vec<usize>)> {
        match self {
            HypergraphEnum::Original(h) => h.raw_links(),
            HypergraphEnum::Sub(h) => h.raw_links(),
        }
    }

    pub fn raw_hypergraphs(
        &self,
    ) -> &'a IndexMap<usize, (Hypergraph<N, E, H, L, Sub>, Vec<(Vec<usize>, Direction)>)> {
        match self {
            HypergraphEnum::Original(h) => h.raw_hypergraphs(),
            HypergraphEnum::Sub(h) => h.raw_hypergraphs(),
        }
    }

    pub fn raw_nodes(&self) -> &'a IndexMap<usize, (N, Vec<(Vec<usize>, Direction)>)> {
        match self {
            HypergraphEnum::Original(h) => h.raw_nodes(),
            HypergraphEnum::Sub(h) => h.raw_nodes(),
        }
    }
}

impl<'a, N, E, H, L, Ty>
    HypergraphEnum<&'a mut Hypergraph<N, E, H, L, Ty>, &'a mut Hypergraph<N, E, H, L, Sub>>
{
    pub fn element_type(
        &mut self,
        id: impl AsRef<[usize]>,
    ) -> Result<ElementType, errors::GetError> {
        match self {
            HypergraphEnum::Original(h) => h.element_type(id),
            HypergraphEnum::Sub(h) => h.element_type(id),
        }
    }
    fn add_local_element(&mut self, element: Element<N, E, H, L, Vec<usize>>) -> usize {
        match self {
            HypergraphEnum::Original(h) => h.add_local_element(element),
            HypergraphEnum::Sub(h) => h.add_local_element(element),
        }
    }
    /// Add a neighbor to the local element with id `local_id`.
    /// the neighbor corresponds to `(link_id, Direction)`.
    ///
    /// # Contract
    ///
    /// `local_id` exists and refers to a linkable element and neighbor's id is a valid link.
    fn add_local_neighbor_unchecked(
        &mut self,
        local_id: usize,
        link_info: (Vec<usize>, Direction),
    ) {
        match self {
            HypergraphEnum::Original(h) => h.add_local_neighbor_unchecked(local_id, link_info),
            HypergraphEnum::Sub(h) => h.add_local_neighbor_unchecked(local_id, link_info),
        }
    }

    pub fn raw_edges_mut(self) -> &'a mut IndexMap<usize, (E, Vec<(Vec<usize>, Direction)>)> {
        match self {
            HypergraphEnum::Original(h) => h.raw_edges_mut(),
            HypergraphEnum::Sub(h) => h.raw_edges_mut(),
        }
    }

    pub fn raw_links_mut(self) -> &'a mut IndexMap<usize, (Option<L>, Vec<usize>, Vec<usize>)> {
        match self {
            HypergraphEnum::Original(h) => h.raw_links_mut(),
            HypergraphEnum::Sub(h) => h.raw_links_mut(),
        }
    }

    pub fn raw_hypergraphs_mut(
        self,
    ) -> &'a mut IndexMap<usize, (Hypergraph<N, E, H, L, Sub>, Vec<(Vec<usize>, Direction)>)> {
        match self {
            HypergraphEnum::Original(h) => h.raw_hypergraphs_mut(),
            HypergraphEnum::Sub(h) => h.raw_hypergraphs_mut(),
        }
    }

    pub fn raw_nodes_mut(self) -> &'a mut IndexMap<usize, (N, Vec<(Vec<usize>, Direction)>)> {
        match self {
            HypergraphEnum::Original(h) => h.raw_nodes_mut(),
            HypergraphEnum::Sub(h) => h.raw_nodes_mut(),
        }
    }
}

// # Note
//
// This should not be public.
impl<N, E, H, L, Ty> Hypergraph<N, E, H, L, Ty> {
    fn add_local_element(&mut self, element: Element<N, E, H, L, Vec<usize>>) -> usize {
        match element {
            Element::Edge { value } => {
                self.edges.insert(self.next_id, (value, Vec::new()));
            }
            Element::Hypergraph { value } => {
                let hypergraph = {
                    let mut h = Hypergraph::<N, E, H, L, Sub>::new();
                    h.set_value(value);
                    h
                };
                self.hypergraphs
                    .insert(self.next_id, (hypergraph, Vec::new()));
            }
            Element::Link {
                source,
                target,
                value,
            } => {
                self.links.insert(self.next_id, (value, source, target));
            }
            Element::Node { value } => {
                self.nodes.insert(self.next_id, (value, Vec::new()));
            }
        }
        self.next_id += 1;
        self.next_id - 1
    }

    fn add_local_neighbor_unchecked(
        &mut self,
        local_id: usize,
        link_info: (Vec<usize>, Direction),
    ) {
        if let Some(edge_full) = self.raw_edges_mut().get_mut(&local_id) {
            let (_, ref mut links_info) = edge_full;
            links_info.push(link_info);
        } else if let Some(hypergraph_full) = self.raw_hypergraphs_mut().get_mut(&local_id) {
            let (_, ref mut links_info) = hypergraph_full;
            links_info.push(link_info);
        } else if let Some(node_full) = self.raw_nodes_mut().get_mut(&local_id) {
            let (_, ref mut links_info) = node_full;
            links_info.push(link_info);
        } else {
            panic!("The local id {} is not a valid.", local_id);
        }
    }
}

/// # Create
///
/// A graph that can be created.
impl<N, E, H, L, Ty: HypergraphClass> Hypergraph<N, E, H, L, Ty> {
    pub fn new() -> Self {
        let nodes = IndexMap::new();
        let edges = IndexMap::new();
        let links = IndexMap::new();
        let hypergraphs = IndexMap::new();
        let next_id = 0;
        Hypergraph {
            value: None,
            nodes,
            edges,
            links,
            hypergraphs,
            next_id,
            class: Ty::new(),
        }
    }

    pub fn with_capacity(nodes: usize, edges: usize, links: usize, hypergraphs: usize) -> Self {
        let nodes = IndexMap::with_capacity(nodes);
        let edges = IndexMap::with_capacity(edges);
        let links = IndexMap::with_capacity(links);
        let hypergraphs = IndexMap::with_capacity(hypergraphs);
        let next_id = 0;
        Hypergraph {
            value: None,
            nodes,
            edges,
            links,
            hypergraphs,
            next_id,
            class: Ty::new(),
        }
    }

    /// Reserve `additional` in all underlying maps of `self`.
    pub fn reserve(&mut self, additional: usize) -> &mut Self {
        self.reserve_edges(additional)
            .reserve_hypergraphs(additional)
            .reserve_links(additional)
            .reserve_nodes(additional)
    }

    pub fn reserve_edges(&mut self, additional: usize) -> &mut Self {
        self.edges.reserve(additional);
        self
    }

    pub fn reserve_hypergraphs(&mut self, additional: usize) -> &mut Self {
        self.hypergraphs.reserve(additional);
        self
    }

    pub fn reserve_links(&mut self, additional: usize) -> &mut Self {
        self.links.reserve(additional);
        self
    }
    pub fn reserve_nodes(&mut self, additional: usize) -> &mut Self {
        self.nodes.reserve(additional);
        self
    }
}

/// # Inform
///
/// Various information about the hypergraph.
impl<N, E, H, L, Ty> Hypergraph<N, E, H, L, Ty> {
    /// Returns the current capacity of the underlying `Map`s.
    ///
    /// The output is ordered allhabetically: edges, hypergraphs, links, nodes.
    pub fn capacities(&self) -> (usize, usize, usize, usize) {
        (
            self.raw_edges().capacity(),
            self.raw_hypergraphs().capacity(),
            self.raw_links().capacity(),
            self.raw_nodes().capacity(),
        )
    }

    /// Returns `true` if `id` corresponds to an existing element of `self`
    /// and it can be linked (node, edge or hypergraph).
    pub fn contains_linkable(&self, id: impl AsRef<[usize]>) -> bool {
        let id = id.as_ref();
        !id.is_empty()
            & (self.contains_edge(id) | self.contains_hypergraph(id) | self.contains_node(id))
    }

    pub fn contains(&self, id: impl AsRef<[usize]>) -> bool {
        let mut id = id.as_ref().to_vec();
        match id.len() {
            0 => true,
            1 => {
                self.contains_edge(&id)
                    | self.contains_link(&id)
                    | self.contains_hypergraph(&id)
                    | self.contains_node(&id)
            }
            _ => {
                let local_id = id.pop().unwrap(); // Never fails since id is non empty.
                let hypergraph = match self.hypergraph(id) {
                    Ok(h) => h,
                    Err(_) => return false,
                };
                hypergraph.contains([local_id])
            }
        }
    }

    pub fn contains_node(&self, id: impl AsRef<[usize]>) -> bool {
        let mut id = id.as_ref().to_vec();
        match id.len() {
            0 => false,
            _ => {
                let local_id = id.pop().unwrap(); // Never fails since id is non empty.
                let hypergraph = match self.hypergraph(id) {
                    Ok(h) => h,
                    Err(_) => return false,
                };
                hypergraph.raw_nodes().contains_key(&local_id)
            }
        }
    }

    pub fn contains_edge(&self, id: impl AsRef<[usize]>) -> bool {
        let mut id = id.as_ref().to_vec();
        match id.len() {
            0 => false,
            _ => {
                let local_id = id.pop().unwrap(); // Never fails since id is non empty.
                let hypergraph = match self.hypergraph(id) {
                    Ok(h) => h,
                    Err(_) => return false,
                };
                hypergraph.raw_edges().contains_key(&local_id)
            }
        }
    }

    pub fn contains_link(&self, id: impl AsRef<[usize]>) -> bool {
        let mut id = id.as_ref().to_vec();
        match id.len() {
            0 => false,
            _ => {
                let local_id = id.pop().unwrap(); // Never fails since id is non empty.
                let hypergraph = match self.hypergraph(id) {
                    Ok(h) => h,
                    Err(_) => return false,
                };
                hypergraph.raw_links().contains_key(&local_id)
            }
        }
    }

    pub fn contains_hypergraph(&self, id: impl AsRef<[usize]>) -> bool {
        let id = id.as_ref();
        id.is_empty() || self.contains_subhypergraph(id)
    }

    /// Returns true if `id` refers to a subhypergraph (possibly nested).
    pub fn contains_subhypergraph(&self, id: impl AsRef<[usize]>) -> bool {
        let id = id.as_ref();
        match id.len() {
            0 => false,
            1 => self.raw_hypergraphs().contains_key(&id[0]),
            _ => {
                let mut hypergraph = match self.raw_hypergraphs().get(&id[0]) {
                    Some(h_full) => &h_full.0,
                    None => return false,
                };
                for local_id in id.iter().skip(1) {
                    hypergraph = match hypergraph.raw_hypergraphs().get(local_id) {
                        Some(hypergraph_full) => &hypergraph_full.0,
                        None => return false,
                    };
                }
                true
            }
        }
    }

    /// Returns the number of levels of nested hypergraphs.
    pub fn depth(&self) -> usize {
        let mut recursive = 0;
        for (_, (h, _)) in &self.hypergraphs {
            recursive = h.depth().max(recursive);
        }
        recursive + 1
    }

    /// Return the number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Returns a bound on valid ids.
    ///
    /// All valid ids are strictly smaller than the output (in lexicographic order).
    pub fn id_bound(&self) -> Vec<usize> {
        let mut result = vec![self.next_local_id()];
        match self.hypergraphs.last() {
            None => result,
            Some((_, (h, _))) => {
                result.extend(h.id_bound());
                result
            }
        }
    }

    /// Returns `true` if there are no nodes or hypergraphs.
    ///
    /// # Esamples
    ///
    /// New hypergraphs are always empty.
    /// ```
    /// # use ferret_hypergraph::Hypergraph;
    /// let h = Hypergraph::<(), ()>::new();
    /// assert!(h.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.raw_nodes().is_empty() && self.raw_hypergraphs().is_empty()
    }

    /// Return the number of hypergraphs in the graph (including itself).
    pub fn hypergraph_count(&self) -> usize {
        1 + self.hypergraphs.len()
    }

    /// Return the number of links in the graph.
    pub fn link_count(&self) -> usize {
        self.links.len()
    }

    /// Return the number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

impl<N, E, H, L, Ty: HypergraphClass> Hypergraph<N, E, H, L, Ty> {
    pub fn is_main(&self) -> bool {
        self.class().is_main()
    }
    pub fn is_sub(&self) -> bool {
        self.class().is_main()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(Hypergraph::<u8, u8>::new(), Main; "Main")]
    #[test_case(Hypergraph::<u8, u8, u8, u8, Sub>::new(), Sub; "Sub")]
    fn class<N, E, H, L, Ty: HypergraphClass>(h: Hypergraph<N, E, H, L, Ty>, expected: Ty) {
        assert_eq!(h.class(), &expected)
    }

    #[test]
    fn contains_linkable() {
        let mut h = Hypergraph::new();
        let node_id = h.add_node("zero", []).unwrap();
        let hypergraph_id = h.add_hypergraph("one", []).unwrap();
        let edge_id = h.add_edge([0], [1], "two", []).unwrap();
        let link_id = h.add_link([0], [2], "three", []).unwrap();
        assert!(h.contains_linkable(node_id));
        assert!(h.contains_linkable(hypergraph_id));
        assert!(h.contains_linkable(edge_id));
        assert!(!h.contains_linkable(link_id));
    }

    #[test_case(Hypergraph::<(), ()>::new(), 1; "no recursion")]
    #[test_case(
        {
            let mut h = Hypergraph::<(), ()>::new();
            h.add_hypergraph((), []).unwrap();
            h
        }, //
        2; //
        "one recursion"
    )]
    #[test_case(
        {
            let mut h = Hypergraph::<(), ()>::new();
            h.add_hypergraph((), []).unwrap();
            h.add_hypergraph((), [0]).unwrap();
            h.add_hypergraph((), [0, 0]).unwrap();
            h.add_hypergraph((), []).unwrap();
            h
        }, //
        4; //
        "tree"
    )]
    fn depth<N, E, H, L, Ty: HypergraphClass>(h: Hypergraph<N, E, H, L, Ty>, expected: usize) {
        assert_eq!(h.depth(), expected)
    }

    #[test]
    fn edge_value() {
        let mut h = Hypergraph::<_, _>::new();
        h.add_node("zero", []).unwrap();
        h.add_node("one", []).unwrap();
        h.add_edge([0], [1], "two", []).unwrap();
        assert_eq!(h.edge_value([2]).unwrap(), &"two");
    }

    #[test]
    fn element_value() {
        let mut h = Hypergraph::new();
        h.add_node("zero", []).unwrap();
        h.add_node("one", []).unwrap();
        h.add_edge([0], [1], "two", []).unwrap();
        h.add_link([0], [2], "three", []).unwrap();
        h.add_hypergraph("six", []).unwrap();
        assert_eq!(
            h.element_value([0]).unwrap(),
            ElementValue::Node { value: &"zero" }
        );
        assert_eq!(
            h.element_value([1]).unwrap(),
            ElementValue::Node { value: &"one" }
        );
        assert_eq!(
            h.element_value([2]).unwrap(),
            ElementValue::Edge { value: &"two" }
        );
        assert_eq!(
            h.element_value([3]).unwrap(),
            ElementValue::Link { value: None }
        );
        assert_eq!(
            h.element_value([4]).unwrap(),
            ElementValue::Link { value: None }
        );
        assert_eq!(
            h.element_value([5]).unwrap(),
            ElementValue::Link {
                value: Some(&"three")
            }
        );
        assert_eq!(
            h.element_value([6]).unwrap(),
            ElementValue::Hypergraph {
                value: Some(&"six")
            }
        );
    }

    #[test]
    fn element_value_mut() {
        let mut h = Hypergraph::<_, ()>::new();
        h.add_node("zero", []).unwrap();
        let element_value = h.element_value_mut([0]).unwrap();
        if let ElementValue::Node { value } = element_value {
            *value = "changed";
        }
        assert_eq!(
            h.element_value([0]).unwrap(),
            ElementValue::Node { value: &"changed" }
        );
    }

    #[test]
    fn hypergraph_value() {
        let mut h = Hypergraph::new();
        h.add_node("zero", []).unwrap();
        h.add_node("one", []).unwrap();
        h.add_edge([0], [1], "two", []).unwrap();
        h.add_link([0], [2], "three", []).unwrap();
        h.add_hypergraph("six", []).unwrap();
        assert_eq!(h.hypergraph_value([6]).unwrap(), &Some("six"));
    }

    #[test]
    fn ids() {
        let mut h = Hypergraph::new();
        h.add_node("zero", []).unwrap();
        h.add_node("one", []).unwrap();
        h.add_edge([0], [1], "two", []).unwrap();
        h.add_link([0], [2], "three", []).unwrap();
        h.add_hypergraph("six", []).unwrap();
        assert_eq!(
            h.ids().collect::<Vec<_>>(),
            vec![
                vec![],
                vec![0],
                vec![1],
                vec![2],
                vec![3],
                vec![4],
                vec![5],
                vec![6]
            ]
        );
    }

    #[test]
    fn links_of() {
        let mut h = Hypergraph::<&str, &str>::new();
        h.add_node("zero", []).unwrap();
        h.add_node("one", []).unwrap();
        h.add_edge([0], [1], "two", []).unwrap();
        assert_eq!(
            h.links_of([0]).unwrap(),
            &vec![(vec![3], Direction::Outgoing)]
        );
        assert_eq!(
            h.links_of([1]).unwrap(),
            &vec![(vec![4], Direction::Incoming)]
        );
        assert_eq!(
            h.links_of([2]).unwrap(),
            &vec![
                (vec![3], Direction::Incoming),
                (vec![4], Direction::Outgoing)
            ]
        );
    }

    #[test]
    fn link_value() {
        let mut h = Hypergraph::<_, _, (), _>::new();
        h.add_node("zero", []).unwrap();
        h.add_node("one", []).unwrap();
        h.add_edge([0], [1], "two", []).unwrap();
        h.add_link([0], [2], "three", []).unwrap();
        assert_eq!(h.link_value([3]).unwrap(), &None);
        assert_eq!(h.link_value([4]).unwrap(), &None);
        assert_eq!(h.link_value([5]).unwrap(), &Some("three"));
    }

    #[test]
    fn neighbors() {
        let mut h = Hypergraph::new();
        h.add_node("zero", []).unwrap();
        h.add_node("one", []).unwrap();
        h.add_edge([0], [1], "two", []).unwrap();
        h.add_link([0], [2], "five", []).unwrap();
        h.add_hypergraph("six", []).unwrap();

        assert!(h.neighbors(vec![]).is_err());
        assert!(h.neighbors(vec![3]).is_err());
        assert!(h.neighbors(vec![4]).is_err());

        let mut neighbors = h.neighbors(vec![0]).unwrap();
        assert_eq!(neighbors.next(), Some(&vec![2]));
        assert_eq!(neighbors.next(), Some(&vec![2]));
        assert_eq!(neighbors.next(), None);
        let mut neighbors = h.neighbors(vec![1]).unwrap();
        assert_eq!(neighbors.next(), None);
        let mut neighbors = h.neighbors(vec![2]).unwrap();
        assert_eq!(neighbors.next(), Some(&vec![1]));
        assert_eq!(neighbors.next(), None);
        let mut neighbors = h.neighbors(vec![6]).unwrap();
        assert_eq!(neighbors.next(), None);
    }

    #[test]
    fn next_id() {
        let mut h = Hypergraph::new();
        h.add_node("zero", []).unwrap();
        h.add_node("one", []).unwrap();
        h.add_edge([0], [1], "two", []).unwrap();
        h.add_link([0], [2], "three", []).unwrap();
        h.add_hypergraph("six", []).unwrap();
        assert_eq!(h.next_id([]).unwrap(), vec![0]);
        assert_eq!(h.next_id([0]).unwrap(), vec![1]);
        assert_eq!(h.next_id([1]).unwrap(), vec![2]);
        assert_eq!(h.next_id([2]).unwrap(), vec![3]);
        assert_eq!(h.next_id([3]).unwrap(), vec![4]);
        assert_eq!(h.next_id([4]).unwrap(), vec![5]);
        assert_eq!(h.next_id([5]).unwrap(), vec![6]);
        assert_eq!(h.next_id([6]), None);
        assert_eq!(h.next_id([0, 0]).unwrap(), vec![1]);
    }

    #[test]
    fn new() {
        Hypergraph::<(), ()>::new();
    }

    #[test]
    fn node_value() {
        let mut h = Hypergraph::<_, ()>::new();
        h.add_node("zero", []).unwrap();
        h.add_node("one", []).unwrap();
        assert_eq!(h.node_value([0]).unwrap(), &"zero");
        assert_eq!(h.node_value([1]).unwrap(), &"one");
    }
}
