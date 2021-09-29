use core::{fmt::Debug, mem};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    direction::Direction,
    elements::*,
    errors,
    iterators::{IdIter, NeighborIter},
    traits::HypergraphClass,
};

mod add;
mod classes;

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
    ) -> Result<&'a Vec<(Vec<usize>, Direction)>, GetError> {
        match self {
            HypergraphEnum::Original(h) => h.links_of(id),
            HypergraphEnum::Sub(h) => h.links_of(id),
        }
    }

    pub fn element_type(&self, id: impl AsRef<[usize]>) -> Result<ElementType, GetError> {
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
    pub fn element_type(&mut self, id: impl AsRef<[usize]>) -> Result<ElementType, GetError> {
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

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("Failed to get element.")]
pub enum GetError {
    NoEdge(#[from] errors::NoEdge),
    NoElement(#[from] errors::NoElement),
    NoElementLinkable(#[from] errors::NoElementLinkable),
    NoHypergraph(#[from] errors::NoHypergraph),
    NoLink(#[from] errors::NoLink),
    NoNode(#[from] errors::NoNode),
    /// The method does not apply to the root hypergraph.
    OriginalHypergraph,
}

/// # Get
///
/// Access node and edge weights (associated data).
impl<N, E, H, L, Ty> Hypergraph<N, E, H, L, Ty> {
    /// Returns the next valid id.
    ///
    /// Returns `None` if `id` there is no valid id that bigger than `id`.
    ///
    /// Order is lexicographic.
    pub fn next_id(&self, id: impl AsRef<[usize]>) -> Option<Vec<usize>> {
        let mut id = id.as_ref().to_vec();
        let bound = self.id_bound();
        if id > bound {
            return None;
        } else if id.is_empty() {
            id = vec![0];
        } else {
            match self.element_type(&id) {
                Err(_) => {
                    let last_local_id = id.last_mut().unwrap(); // Never fails since id is not empty
                    *last_local_id += 1;
                    if *last_local_id >= bound[id.len() - 1] {
                        id.pop(); // Go back one level
                        let last_local_id = match id.last_mut() {
                            None => return None,
                            Some(i) => i,
                        };
                        *last_local_id += 1;
                    }
                }
                Ok(element_type) => match element_type {
                    ElementType::Edge | ElementType::Link | ElementType::Node => {
                        let last_local_id = id.last_mut().unwrap(); // Never fails since id is not empty
                        *last_local_id += 1;
                    }
                    ElementType::Hypergraph => {
                        id.push(0);
                    }
                },
            }
        }
        if self.contains(&id) {
            return Some(id);
        } else {
            return self.next_id(id);
        }
    }

    /// Returns the local id that will be given to the next element added.
    pub fn next_local_id(&self) -> usize {
        self.next_id
    }

    /// Returns an iterator over all valid ids of `self`.
    pub fn ids<'a>(&'a self) -> IdIter<'a, N, E, H, L, Ty> {
        IdIter::new(&self)
    }

    /// Returns the class marker.
    pub fn class(&self) -> &Ty {
        &self.class
    }
    /// Returns the hypergraph with id `id`, if it exists.
    ///
    /// `None` is returned when the element does not exists.
    pub fn hypergraph(
        &self,
        id: impl AsRef<[usize]>,
    ) -> Result<HypergraphEnum<&Self, &Hypergraph<N, E, H, L, Sub>>, GetError> {
        let id = id.as_ref();
        if id.is_empty() {
            return Ok(HypergraphEnum::Original(&self));
        }
        let h = self.subhypergraph(id)?;
        Ok(HypergraphEnum::Sub(h))
    }

    /// Returns the hypergraph in which `id` lives, if it exists.
    ///
    /// `None` is returned when: there is no element with id `id`; or `id` is empty.
    pub fn hypergraph_of(
        &self,
        id: impl AsRef<[usize]>,
    ) -> Result<HypergraphEnum<&Self, &Hypergraph<N, E, H, L, Sub>>, GetError> {
        let id = id.as_ref();
        match id.len() {
            0 => Err(GetError::OriginalHypergraph),
            1 => Ok(HypergraphEnum::Original(&self)),
            _ => {
                let id = &id[0..id.len() - 1];
                let subhypergraph = self.subhypergraph(&id)?;
                Ok(HypergraphEnum::Sub(subhypergraph))
            }
        }
    }

    /// Returns the hypergraph in which `id` lives, if it exists.
    ///
    /// `None` is returned when there is no element with id `id`.
    pub fn hypergraph_of_mut(
        &mut self,
        id: impl AsRef<[usize]>,
    ) -> Result<HypergraphEnum<&mut Self, &mut Hypergraph<N, E, H, L, Sub>>, GetError> {
        let id = id.as_ref();
        match id.len() {
            0 => Err(GetError::OriginalHypergraph),
            1 => Ok(HypergraphEnum::Original(self)),
            _ => {
                let id = &id[0..id.len() - 1];
                let subhypergraph = self.subhypergraph_mut(&id)?;
                Ok(HypergraphEnum::Sub(subhypergraph))
            }
        }
    }

    /// Returns the hypergraph with id `id`, if it exists.
    ///
    /// `None` is returned when the element does not exists.
    pub fn hypergraph_mut(
        &mut self,
        id: impl AsRef<[usize]>,
    ) -> Result<HypergraphEnum<&mut Self, &mut Hypergraph<N, E, H, L, Sub>>, GetError> {
        let id = id.as_ref();
        if id.is_empty() {
            return Ok(HypergraphEnum::Original(self));
        }
        match self.subhypergraph_mut(&id) {
            Some(h) => Ok(HypergraphEnum::Sub(h)),
            None => Err(errors::NoHypergraph(id.to_vec()))?,
        }
    }

    /// Returns the links of an element of the current hypergraph, `None` if the element does not exists or is a link.
    pub fn links_of(
        &self,
        id: impl AsRef<[usize]>,
    ) -> Result<&Vec<(Vec<usize>, Direction)>, GetError> {
        let id = id.as_ref();
        if !self.contains_linkable(&id) {
            Err(errors::NoElementLinkable(id.to_vec()))?;
        }
        let hypergraph = self.hypergraph_of(&id)?;
        let local_id = id.last().unwrap(); // Never fails since id refers to a linkable element
        let links = match hypergraph.element_type(&id).unwrap() // Never fails since id refers to a linkable element
        {
            ElementType::Edge => {
                hypergraph
                    .raw_edges()
                    .get(local_id)
                    .map(|edge_full| &edge_full.1)
                    .unwrap() // Never fails since id refers to a linkable element
            }
            ElementType::Hypergraph => {
                hypergraph
                    .raw_hypergraphs()
                    .get(local_id)
                    .map(|hypergraph_full| &hypergraph_full.1)
                    .unwrap() // Never fails since id refers to a linkable element
            }
            ElementType::Link => unreachable!(), // Since id is a linkable element
            ElementType::Node => {
                hypergraph
                    .raw_nodes()
                    .get(local_id)
                    .map(|node_full| &node_full.1)
                    .unwrap() // Never fails since id refers to a linkable element
            }
        };
        Ok(links)
    }

    /// Returns the links of an element of the current hypergraph, `None` if the element does not exists or is a link.
    pub(crate) fn links_of_mut(
        &mut self,
        id: impl AsRef<[usize]>,
    ) -> Result<&mut Vec<(Vec<usize>, Direction)>, GetError> {
        let id = id.as_ref();
        if !self.contains_linkable(&id) {
            Err(errors::NoElementLinkable(id.to_vec()))?;
        }
        let mut hypergraph = self.hypergraph_of_mut(&id)?;
        let local_id = id.last().unwrap(); // Never fails since id refers to a linkable element
        let links = match hypergraph.element_type([*local_id]).unwrap() // Never fails since id refers to a linkable element
        {
            ElementType::Edge => {
                hypergraph
                    .raw_edges_mut()
                    .get_mut(local_id)
                    .map(|edge_full| &mut edge_full.1)
                    .unwrap() // Never fails since id refers to a linkable element
            }
            ElementType::Hypergraph => {
                hypergraph
                    .raw_hypergraphs_mut()
                    .get_mut(local_id)
                    .map(|hypergraph_full| &mut hypergraph_full.1)
                    .unwrap() // Never fails since id refers to a linkable element
            }
            ElementType::Link => unreachable!(), // Since id is a linkable element
            ElementType::Node => {
                hypergraph
                    .raw_nodes_mut()
                    .get_mut(local_id)
                    .map(|node_full| &mut node_full.1)
                    .unwrap() // Never fails since id refers to a linkable element
            }
        };
        Ok(links)
    }

    /// Returns an iterator over outgoing neighbors.
    pub fn neighbors(
        &self,
        id: impl AsRef<[usize]>,
    ) -> Result<NeighborIter<N, E, H, L, Ty>, crate::iterators::neighbor_iter::NewError> {
        let direction = Direction::Outgoing;
        self.neighbors_directed(id, direction)
    }

    pub fn neighbors_directed(
        &self,
        id: impl AsRef<[usize]>,
        direction: Direction,
    ) -> Result<NeighborIter<N, E, H, L, Ty>, crate::iterators::neighbor_iter::NewError> {
        NeighborIter::new(self, id, direction)
    }

    pub fn element_type(&self, id: impl AsRef<[usize]>) -> Result<ElementType, GetError> {
        self.element_value(id)
            .map(|element| -> ElementType { element.into() })
    }

    pub fn element_value(
        &self,
        id: impl AsRef<[usize]>,
    ) -> Result<ElementValue<&N, &E, &H, &L>, GetError> {
        let id = id.as_ref();
        if id.is_empty() {
            return Ok(ElementValue::Hypergraph {
                value: self.value().as_ref(),
            });
        }

        let local_id = id.last().unwrap(); // Never fails by previous check

        let hypergraph = self.hypergraph_of(&id)?;

        let element;
        if let Some(edge_full) = hypergraph.raw_edges().get(local_id) {
            element = ElementValue::Edge {
                value: &edge_full.0,
            };
            return Ok(element);
        } else if let Some(hypergraph_full) = hypergraph.raw_hypergraphs().get(local_id) {
            element = ElementValue::Hypergraph {
                value: hypergraph_full.0.value.as_ref(),
            };
            return Ok(element);
        } else if let Some(link_full) = hypergraph.raw_links().get(local_id) {
            element = ElementValue::Link {
                value: (&link_full.0).as_ref(),
            };
            return Ok(element);
        } else if let Some(node_full) = hypergraph.raw_nodes().get(local_id) {
            element = ElementValue::Node {
                value: &node_full.0,
            };
            return Ok(element);
        };
        Err(errors::NoElement(id.to_vec()))?
    }

    pub fn edge_value(&self, id: impl AsRef<[usize]>) -> Result<&E, GetError> {
        let id = id.as_ref();
        if !self.contains_edge(&id) {
            Err(errors::NoEdge(id.to_vec()))?
        }
        let hypergraph = self.hypergraph_of(&id).unwrap(); // Never fails since id refers to a valid edge
        let local_id = id.last().unwrap(); // Never fails since id refers to a valid edge
        let edge_value = hypergraph
            .raw_edges()
            .get(local_id)
            .map(|edge_full| &edge_full.0)
            .unwrap(); // Never fails since id refers to a valid edge
        Ok(edge_value)
    }

    /// Returns the pair of gloalbal `id`s `(source, target)` if the link exists.
    pub fn link_endpoints(
        &self,
        id: impl AsRef<[usize]>,
    ) -> Result<(&Vec<usize>, &Vec<usize>), GetError> {
        let id = id.as_ref();
        if !self.contains_link(&id) {
            Err(errors::NoLink(id.to_vec()))?
        }
        let hypergraph = self.hypergraph_of(&id).unwrap(); // Never fails since id refers to a valid link
        let local_id = id.last().unwrap(); // Never fails since id refers to a valid link
        let link_endpoints = hypergraph
            .raw_links()
            .get(local_id)
            .map(|link_full| (&link_full.1, &link_full.2))
            .unwrap(); // Never fails since id refers to a valid link
        Ok(link_endpoints)
    }

    pub fn link_value(&self, id: impl AsRef<[usize]>) -> Result<&Option<L>, GetError> {
        let id = id.as_ref();
        if !self.contains_link(&id) {
            Err(errors::NoLink(id.to_vec()))?
        }
        let hypergraph = self.hypergraph_of(&id).unwrap(); // Never fails since id refers to a valid link
        let local_id = id.last().unwrap(); // Never fails since id refers to a valid link
        let link_value = hypergraph
            .raw_links()
            .get(local_id)
            .map(|link_full| &link_full.0)
            .unwrap(); // Never fails since id refers to a valid link
        Ok(link_value)
    }

    /// Returns the subgraph with id `id`, if it exists.
    ///
    /// `None` is returned when `id` is empty, or there is no (sub-)hypergraph with such `id`.
    pub fn subhypergraph(
        &self,
        id: impl AsRef<[usize]>,
    ) -> Result<&Hypergraph<N, E, H, L, Sub>, GetError> {
        let id = id.as_ref().to_vec();
        match id.len() {
            0 => Err(GetError::OriginalHypergraph),
            1 => match self.hypergraphs.get(&id[0]).map(|h_full| &h_full.0) {
                Some(h) => Ok(h),
                None => Err(errors::NoHypergraph(id))?,
            },
            _ => {
                let local_id = id[0]; // Never fails since id is non empty.
                let mut subhypergraph = match self.raw_hypergraphs().get(&local_id) {
                    None => Err(errors::NoHypergraph(vec![local_id]))?,
                    Some(hypergraph_full) => hypergraph_full.0,
                };
                for (counter, local_id) in id.iter().enumerate().skip(1) {
                    subhypergraph = match subhypergraph.raw_hypergraphs().get(local_id) {
                        None => Err(errors::NoHypergraph(id[0..=counter].to_vec()))?,
                        Some(hypergraph_full) => hypergraph_full.0,
                    };
                }
                Ok(&subhypergraph)
            }
        }
    }

    pub fn hypergraph_value(&self, id: impl AsRef<[usize]>) -> Result<&Option<H>, GetError> {
        let id = id.as_ref();
        if !self.contains_hypergraph(id) {
            Err(errors::NoHypergraph(id.to_vec()))?
        }

        match id.len() {
            0 => Ok(self.value()),
            _ => {
                let hypergraph = self.hypergraph_of(&id)?;
                let local_id = id.last().unwrap(); // Never fails since id is non empty.
                let hypergraph_value = hypergraph
                    .raw_hypergraphs()
                    .get(local_id)
                    .map(|hypergraph_full| &hypergraph_full.0.value)
                    .unwrap(); // Never fails since id refers to a hypergraph
                Ok(hypergraph_value)
            }
        }
    }

    pub fn node_value(&self, id: impl AsRef<[usize]>) -> Result<&N, GetError> {
        let id = id.as_ref();
        if !self.contains_node(id) {
            Err(errors::NoNode(id.to_vec()))?
        }

        match id.len() {
            0 => Err(GetError::OriginalHypergraph),
            _ => {
                let hypergraph = self.hypergraph_of(&id)?;
                let local_id = id.last().unwrap(); // Never fails since id is non empty.
                let node_value = hypergraph
                    .raw_nodes()
                    .get(local_id)
                    .map(|node_full| &node_full.0)
                    .unwrap(); // Never fails since id refers to a hypergraph
                Ok(node_value)
            }
        }
    }

    pub fn element_value_mut(
        &mut self,
        id: impl AsRef<[usize]>,
    ) -> Option<ElementValue<&mut N, &mut E, &mut H, &mut L>> {
        let id = id.as_ref();
        if id.is_empty() {
            return Some(ElementValue::Hypergraph {
                value: self.value_mut().as_mut(),
            });
        }

        match self.element_type(id) {
            Some(element_type) => {
                let local_id = id.last().unwrap(); // Never fails by previous check
                let hypergraph = self.hypergraph_of_mut(id).unwrap(); // Never fails since id is a valid element
                let element;
                match element_type {
                    ElementType::Edge => {
                        let edge_full = hypergraph.raw_edges_mut().get_mut(local_id).unwrap(); // Never fails since id is a valid element
                        element = ElementValue::Edge {
                            value: &mut edge_full.0,
                        };
                    }
                    ElementType::Hypergraph => {
                        let hypergraph_full =
                            hypergraph.raw_hypergraphs_mut().get_mut(local_id).unwrap(); // Never fails since id is a valid element
                        element = ElementValue::Hypergraph {
                            value: hypergraph_full.0.value.as_mut(),
                        };
                    }
                    ElementType::Link => {
                        let link_full = hypergraph.raw_links_mut().get_mut(local_id).unwrap(); // Never fails since id is a valid element
                        element = ElementValue::Link {
                            value: link_full.0.as_mut(),
                        };
                    }
                    ElementType::Node => {
                        let node_full = hypergraph.raw_nodes_mut().get_mut(local_id).unwrap(); // Never fails since id is a valid element
                        element = ElementValue::Node {
                            value: &mut node_full.0,
                        };
                    }
                }
                Some(element)
            }
            None => None,
        }
    }

    pub fn raw_edges(&self) -> &IndexMap<usize, (E, Vec<(Vec<usize>, Direction)>)> {
        &self.edges
    }

    pub fn raw_links(&self) -> &IndexMap<usize, (Option<L>, Vec<usize>, Vec<usize>)> {
        &self.links
    }

    pub fn raw_hypergraphs(
        &self,
    ) -> &IndexMap<usize, (Hypergraph<N, E, H, L, Sub>, Vec<(Vec<usize>, Direction)>)> {
        &self.hypergraphs
    }

    pub fn raw_nodes(&self) -> &IndexMap<usize, (N, Vec<(Vec<usize>, Direction)>)> {
        &self.nodes
    }

    pub(crate) fn raw_edges_mut<'a>(
        &'a mut self,
    ) -> &'a mut IndexMap<usize, (E, Vec<(Vec<usize>, Direction)>)> {
        &mut self.edges
    }

    pub(crate) fn raw_links_mut(
        &mut self,
    ) -> &mut IndexMap<usize, (Option<L>, Vec<usize>, Vec<usize>)> {
        &mut self.links
    }

    pub(crate) fn raw_hypergraphs_mut(
        &mut self,
    ) -> &mut IndexMap<usize, (Hypergraph<N, E, H, L, Sub>, Vec<(Vec<usize>, Direction)>)> {
        &mut self.hypergraphs
    }

    pub(crate) fn raw_nodes_mut(
        &mut self,
    ) -> &mut IndexMap<usize, (N, Vec<(Vec<usize>, Direction)>)> {
        &mut self.nodes
    }

    pub fn edge_value_mut(&mut self, id: impl AsRef<[usize]>) -> Option<&mut E> {
        if let Some(h) = self.hypergraph_of_mut(&id) {
            if let Some(local_id) = id.as_ref().last() {
                h.raw_edges_mut()
                    .get_mut(local_id)
                    .map(|edge_full| &mut edge_full.0)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn link_value_mut(&mut self, id: impl AsRef<[usize]>) -> Option<&mut Option<L>> {
        if let Some(h) = self.hypergraph_of_mut(&id) {
            if let Some(local_id) = id.as_ref().last() {
                h.raw_links_mut()
                    .get_mut(local_id)
                    .map(|link_full| &mut link_full.0)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn hypergraph_value_mut(&mut self, id: impl AsRef<[usize]>) -> Option<&mut Option<H>> {
        if id.as_ref().is_empty() {
            Some(self.value_mut())
        } else if let Some(h) = self.hypergraph_of_mut(&id) {
            if let Some(local_id) = id.as_ref().last() {
                h.raw_hypergraphs_mut()
                    .get_mut(local_id)
                    .map(|hypergraph_full| &mut hypergraph_full.0.value)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn node_value_mut(&mut self, id: impl AsRef<[usize]>) -> Option<&mut N> {
        if let Some(h) = self.hypergraph_of_mut(&id) {
            if let Some(local_id) = id.as_ref().last() {
                h.raw_nodes_mut()
                    .get_mut(local_id)
                    .map(|node_full| &mut node_full.0)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Returns the subgraph with id `id`, if it exists.
    ///
    /// `None` is returned when `id` is empty, or there is no (sub-)hypergraph with such `id`.
    ///
    /// Notice that the type of `hypergraph` is defined dynamically.
    pub fn subhypergraph_mut(
        &mut self,
        id: impl AsRef<[usize]>,
    ) -> Result<&mut Hypergraph<N, E, H, L, Sub>, GetError> {
        let id = id.as_ref().to_vec();
        match id.len() {
            0 => Err(GetError::OriginalHypergraph),
            1 => match self.hypergraphs.get_mut(&id[0]).map(|h_full| &mut h_full.0) {
                Some(h) => Ok(h),
                None => Err(errors::NoHypergraph(id))?,
            },
            _ => {
                let local_id = id[0]; // Never fails since id is non empty.
                let mut subhypergraph = match self.raw_hypergraphs_mut().get_mut(&local_id) {
                    None => Err(errors::NoHypergraph(vec![local_id]))?,
                    Some(hypergraph_full) => hypergraph_full.0,
                };
                for (counter, local_id) in id.iter().enumerate().skip(1) {
                    subhypergraph = match subhypergraph.raw_hypergraphs_mut().get_mut(local_id) {
                        None => Err(errors::NoHypergraph(id[0..=counter].to_vec()))?,
                        Some(hypergraph_full) => hypergraph_full.0,
                    };
                }
                Ok(&mut subhypergraph)
            }
        }
    }

    pub fn value(&self) -> &Option<H> {
        &self.value
    }
    pub fn value_mut(self: &mut Self) -> &mut Option<H> {
        &mut self.value
    }
}
/// # Set
impl<N, E, H, L, Ty> Hypergraph<N, E, H, L, Ty> {
    /// Change the value of the hypergraph as a whole.
    pub fn set_value(&mut self, new_value: impl Into<Option<H>>) -> &mut Self {
        self.value = new_value.into();
        self
    }

    pub fn set_element_value(
        &mut self,
        id: impl AsRef<[usize]>,
        new_value: ElementValue<N, E, H, L>,
    ) -> Option<ElementValue<N, E, H, L>> {
        match new_value {
            ElementValue::Edge { value } => {
                let old_value = match self.set_edge_value(id, value) {
                    None => return None,
                    Some(v) => v,
                };
                Some(ElementValue::Edge { value: old_value })
            }
            ElementValue::Hypergraph { value } => {
                let old_value = match self.set_hypergraph_value(id, value) {
                    None => return None,
                    Some(v) => v,
                };
                Some(ElementValue::Hypergraph { value: old_value })
            }
            ElementValue::Link { value } => {
                let old_value = match self.set_link_value(id, value) {
                    None => return None,
                    Some(v) => v,
                };
                Some(ElementValue::Link { value: old_value })
            }
            ElementValue::Node { value } => {
                let old_value = match self.set_node_value(id, value) {
                    None => return None,
                    Some(v) => v,
                };
                Some(ElementValue::Node { value: old_value })
            }
        }
    }

    pub fn set_edge_value(&mut self, id: impl AsRef<[usize]>, mut new_value: E) -> Option<E> {
        let id = id.as_ref();
        let hypergraph = match self.hypergraph_of_mut(&id) {
            None => return None,
            Some(h) => h,
        };
        let local_id = id.last().unwrap(); // Never fails since id is not empty

        let raw_edges = hypergraph.raw_edges_mut();
        match raw_edges.get_mut(local_id) {
            Some((old_value, _links)) => mem::swap(old_value, &mut new_value),
            None => return None,
        }
        Some(new_value)
    }

    pub fn set_link_value(
        &mut self,
        id: impl AsRef<[usize]>,
        new_value: impl Into<Option<L>>,
    ) -> Option<Option<L>> {
        let id = id.as_ref();
        let mut new_value = new_value.into();
        let hypergraph = match self.hypergraph_of_mut(&id) {
            None => return None,
            Some(h) => h,
        };
        let local_id = id.last().unwrap(); // Never fails since id is not empty

        let raw_links = hypergraph.raw_links_mut();
        match raw_links.get_mut(local_id) {
            Some((old_value, _, _)) => mem::swap(old_value, &mut new_value),
            None => return None,
        }
        Some(new_value)
    }

    pub fn set_hypergraph_value(
        &mut self,
        id: impl AsRef<[usize]>,
        new_value: impl Into<Option<H>>,
    ) -> Option<Option<H>> {
        let id = id.as_ref();
        let mut new_value = new_value.into();
        let hypergraph = match self.hypergraph_of_mut(&id) {
            None => return None,
            Some(h) => h,
        };
        let local_id = id.last().unwrap(); // Never fails since id is not empty

        let raw_hypergraphs = hypergraph.raw_hypergraphs_mut();
        match raw_hypergraphs.get_mut(local_id) {
            Some((h, _links)) => mem::swap(&mut h.value, &mut new_value),
            None => return None,
        }
        Some(new_value)
    }

    pub fn set_node_value(&mut self, id: impl AsRef<[usize]>, mut new_value: N) -> Option<N> {
        let id = id.as_ref();
        let hypergraph = match self.hypergraph_of_mut(&id) {
            None => return None,
            Some(h) => h,
        };
        let local_id = id.last().unwrap(); // Never fails since id is not empty

        let raw_nodes = hypergraph.raw_nodes_mut();
        match raw_nodes.get_mut(local_id) {
            Some((old_value, _links)) => mem::swap(old_value, &mut new_value),
            None => return None,
        }
        Some(new_value)
    }
}

/// # Clear
///
/// A graph that can be cleared.
impl<N, E, H, L> Hypergraph<N, E, H, L, Main> {
    pub fn clear(&mut self) -> &mut Self {
        self.clear_edges()
            .clear_hypergraphs()
            .clear_links()
            .clear_nodes()
    }

    pub fn clear_edges(&mut self) -> &mut Self {
        self.edges.clear();
        self
    }

    pub fn clear_hypergraphs(&mut self) -> &mut Self {
        self.hypergraphs.clear();
        self
    }

    pub fn clear_links(&mut self) -> &mut Self {
        self.links.clear();
        self
    }

    pub fn clear_nodes(&mut self) -> &mut Self {
        self.nodes.clear();
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
        id.is_empty() || self.contains_subhypegraph(id)
    }

    /// Returns true if `id` refers to a subhypergraph (possibly nested).
    pub fn contains_subhypegraph(&self, id: impl AsRef<[usize]>) -> bool {
        let id = id.as_ref();
        match id.len() {
            0 => false,
            1 => self.raw_hypergraphs().contains_key(&id[0]),
            _ => {
                let hypergraph = match self.raw_hypergraphs().get(&id[0]) {
                    Some(h_full) => h_full.0,
                    None => return false,
                };
                for local_id in id.iter().skip(1) {
                    hypergraph = match hypergraph.raw_hypergraphs().get(local_id) {
                        Some(hypergraph_full) => hypergraph_full.0,
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

    /// Return the number of links in the graph.
    pub fn link_count(&self) -> usize {
        self.links.len()
    }

    /// Return the number of hypergraphs in the graph (including itself).
    pub fn hypergraph_count(&self) -> usize {
        1 + self.hypergraphs.len()
    }

    /// Return the number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
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
}

impl<N, E, H, L, Ty: HypergraphClass> Hypergraph<N, E, H, L, Ty> {
    pub fn is_main(&self) -> bool {
        self.class().is_main()
    }
    pub fn is_sub(&self) -> bool {
        self.class().is_main()
    }
}

/// # Find
///
/// Find elements.
impl<N, E, H, L, Ty> Hypergraph<N, E, H, L, Ty> {
    /// Returns the id of the link that belongs to hypergraph `location` linking `source` and `target`.
    ///
    /// An empty `location` means the main hypergraph.
    ///
    /// Returns `None` if it does not exists.
    pub fn find_link_id(
        &mut self,
        source: impl AsRef<[usize]>,
        target: impl AsRef<[usize]>,
        value: &Option<L>,
        location: impl AsRef<[usize]>,
    ) -> Option<Vec<usize>>
    where
        L: PartialEq,
    {
        let location = location.as_ref();
        let hypergraph = match self.hypergraph(location) {
            Some(h) => h,
            None => return None,
        };
        let links = hypergraph.raw_links();
        let source = source.as_ref().to_vec();
        let target = target.as_ref().to_vec();
        for (local_id, link_full) in links {
            if (link_full.0.as_ref(), &link_full.1, &link_full.2)
                == (value.as_ref(), &source, &target)
            {
                let mut location = location.to_vec();
                location.push(*local_id);
                return Some(location);
            }
        }
        None
    }
    pub fn find_element_by_value(&self, value: ElementValue<&N, &E, &H, &L>) -> Option<Vec<usize>> {
        todo!()
    }

    pub fn find_node_by_value(&self, value: &N) -> Option<Vec<usize>> {
        todo!()
    }

    pub fn find_edge_by_value(&self, value: &E) -> Option<Vec<usize>> {
        todo!()
    }

    pub fn find_link_by_value(&self, value: &Option<L>) -> Option<Vec<usize>> {
        todo!()
    }

    pub fn find_hypergraph_by_value(&self, value: &Option<H>) -> Option<Vec<usize>> {
        todo!()
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("Failed to remove element.")]
pub enum RemoveError {
    GetError(#[from] GetError),
}

/// # Remove
///
/// Remove elements.
impl<N, E, H, L, Ty> Hypergraph<N, E, H, L, Ty> {
    /// Removes the element with id `id`.
    ///
    /// Returns true if the element was removed, otherwise `false`.
    pub fn remove(&mut self, id: impl AsRef<[usize]>) -> Result<(), RemoveError> {
        match self.element_type(&id)? {
            ElementType::Edge => self.remove_edge(id)?,
            ElementType::Hypergraph => self.remove_hypergraph(id)?,
            ElementType::Link => self.remove_link(id)?,
            ElementType::Node => self.remove_node(id)?,
        }
        Ok(())
    }

    pub fn remove_edge(&mut self, id: impl AsRef<[usize]>) -> Option<E> {
        let id = id.as_ref();
        match id.len() {
            0 => None,
            _ => {
                let local_id = id.last().unwrap(); // Never fails since id is not empty
                let hypergraph = match self.hypergraph_of_mut(id) {
                    None => return None,
                    Some(h) => h,
                };
                let raw_edges = hypergraph.raw_edges_mut();
                let (edge_value, edge_links) = match raw_edges.remove(local_id) {
                    None => return None,
                    Some(edge_full) => edge_full,
                };
                for (link_id, _) in edge_links {
                    self.remove_link(link_id);
                }
                Some(edge_value)
            }
        }
    }

    pub fn remove_hypergraph(&mut self, id: impl AsRef<[usize]>) -> Option<Option<H>> {
        let id = id.as_ref();
        match id.len() {
            0 => None,
            _ => {
                let local_id = id.last().unwrap(); // Never fails since id is not empty
                let hypergraph = match self.hypergraph_of_mut(id) {
                    None => return None,
                    Some(h) => h,
                };
                let raw_hypergraphs = hypergraph.raw_hypergraphs_mut();
                let (subhypergraph, subhypergraph_links) = match raw_hypergraphs.remove(local_id) {
                    None => return None,
                    Some(hypergraph_full) => hypergraph_full,
                };
                for (link_id, _) in subhypergraph_links {
                    self.remove_link(link_id);
                }
                let id = id.to_vec();
                for local_id in subhypergraph.ids() {
                    let mut gloabl_id = id.clone();
                    gloabl_id.extend(local_id);
                    self.remove(gloabl_id);
                }
                Some(subhypergraph.value)
            }
        }
    }

    pub fn remove_link(&mut self, id: impl AsRef<[usize]>) -> Option<Option<L>> {
        let id = id.as_ref();
        match id.len() {
            0 => None,
            _ => {
                let local_id = id.last().unwrap(); // Never fails since id is not empty
                let hypergraph = match self.hypergraph_of_mut(&id) {
                    None => return None,
                    Some(h) => h,
                };
                let raw_links = hypergraph.raw_links_mut();
                let (link_value, source_id, target_id) = match raw_links.remove(local_id) {
                    None => return None,
                    Some(link_full) => link_full,
                };
                self.remove_link_from(&id, source_id);
                self.remove_link_from(&id, target_id);
                Some(link_value)
            }
        }
    }

    /// Removes the link with id `link_id` from the list of links of the element `id`.  
    ///
    ///
    fn remove_link_from(&mut self, link_id: impl AsRef<[usize]>, id: impl AsRef<[usize]>) -> bool {
        let id = id.as_ref();
        let link_id = link_id.as_ref();
        match id.len() {
            0 => false,
            _ => {
                let local_id = id.last().unwrap(); // Never fails since id is not empty
                let element_type = match self.element_type(id) {
                    None => return false,
                    Some(e) => e,
                };
                let hypergraph = self.hypergraph_of_mut(&id).unwrap(); // Never fails since id refers to an element
                match element_type {
                    ElementType::Edge => {
                        let raw_edges = hypergraph.raw_edges_mut();
                        let (_, edge_links) = match raw_edges.get_mut(local_id) {
                            None => return false,
                            Some(edge_full) => edge_full,
                        };
                        let link_index =
                            match edge_links.iter().position(|(l_id, _)| link_id == l_id) {
                                None => return false, // failure
                                Some(i) => i,
                            };
                        edge_links.remove(link_index);
                        if edge_links.len() < 2 {
                            self.remove_edge(id);
                        }
                        true
                    }
                    ElementType::Hypergraph => {
                        let raw_hypergraphs = hypergraph.raw_hypergraphs_mut();
                        let (_, hyperraph_links) = match raw_hypergraphs.get_mut(local_id) {
                            None => return false,
                            Some(h) => h,
                        };
                        let link_index = match hyperraph_links
                            .iter()
                            .position(|(link_id, _)| link_id == id)
                        {
                            None => return false, // failure
                            Some(i) => i,
                        };
                        hyperraph_links.remove(link_index);
                        true
                    }
                    ElementType::Link => {
                        unreachable!() // Should fail
                    }
                    ElementType::Node => {
                        let raw_nodes = hypergraph.raw_nodes_mut();
                        let (_, node_links) = match raw_nodes.get_mut(local_id) {
                            None => return false,
                            Some(node_full) => node_full,
                        };
                        let link_index =
                            match node_links.iter().position(|(l_id, _)| link_id == l_id) {
                                None => return false, // failure
                                Some(i) => i,
                            };
                        node_links.remove(link_index);
                        true
                    }
                }
            }
        }
    }

    pub fn remove_node(&mut self, id: impl AsRef<[usize]>) -> Option<N> {
        let id = id.as_ref();
        match id.len() {
            0 => None,
            _ => {
                let local_id = id.last().unwrap(); // Never fails since id is not empty
                let hypergraph = match self.hypergraph_of_mut(id) {
                    None => return None,
                    Some(h) => h,
                };
                let raw_nodes = hypergraph.raw_nodes_mut();
                let (node_value, node_links) = match raw_nodes.remove(local_id) {
                    None => return None,
                    Some(node_full) => node_full,
                };
                for (link_id, _) in node_links {
                    self.remove_link(link_id);
                }
                Some(node_value)
            }
        }
    }

    /// Removes the first element matching `value`.
    pub fn remove_element_by_value(&mut self, value: ElementValue<&N, &E, &H, &L>) -> bool {
        match self.find_element_by_value(value) {
            Some(id) => self.remove(id),
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test]
    fn add_edge() {
        let mut h = Hypergraph::<&str, &str>::new();
        let node_0_id = h.add_node("zero", []).unwrap();
        let node_1_id = h.add_node("one", []).unwrap();
        let edge_id = h.add_edge([0], [1], "two", []).unwrap();
        assert_eq!(vec![2], edge_id);
        assert_eq!(None, h.find_link_id([0], [1], &None, []));
        assert_eq!(Some(vec![3]), h.find_link_id([0], [2], &None, []));
        assert_eq!(Some(vec![4]), h.find_link_id([2], [1], &None, []));
        assert!(h.contains_node(node_0_id));
        assert!(h.contains_node(node_1_id));
        assert_eq!(h.edge_value(&edge_id).unwrap(), &"two");
    }

    #[test_case(
     {
     	let mut h = Hypergraph::<_, ()>::new();
     	h.add_node("zero", []).unwrap();
     	h.add_link([0], [], (), [])
     }, //
     AddError::EmptyTarget; //
    	"empty target"
    )]
    #[test_case(
	    {
	    	let mut h = Hypergraph::<(), (), _>::new();
	    	h.add_hypergraph("zero", []).unwrap();
	    	h.add_hypergraph("one", []).unwrap();
	    	h.add_hypergraph("zero", [0]).unwrap();
	    	h.add_edge([0, 0], [0], (), [1])
	    }, //
	    AddError::IncoherentLink(vec![1], vec![0, 0], vec![0]); //
    	"incoherent link"
    )]
    #[test_case(
     {
     	let mut h = Hypergraph::<_, _, ()>::new();
     	h.add_node("zero", []).unwrap();
     	h.add_node("one", []).unwrap();
     	h.add_edge([0], [1], "two", []).unwrap();
     	h.add_link([3], [0], (), [])
     }, //
     AddError::LinkSource(vec![3]); //
    	"link source"
    )]
    #[test_case(
     {
     	let mut h = Hypergraph::<_, _, ()>::new();
     	h.add_node("zero", []).unwrap();
     	h.add_node("one", []).unwrap();
     	h.add_edge([0], [1], "two", []).unwrap();
     	h.add_link([0], [3], (), [])
     }, //
     AddError::LinkTarget(vec![3]); //
    	"link target"
    )]
    #[test_case(
    	{
	    	let mut h = Hypergraph::<_, ()>::new();
	    	h.add_node("zero", [1])
    	},//
    	AddError::NoHypergraph(vec![1]); //
    	"no hypergraph"
    )]
    #[test_case(
    	{
    		let mut h = Hypergraph::<(), (), (), _>::new();
    		h.add_link([1], [], "zero", [])
    	}, //
    	AddError::NoSource(vec![1]); //
    	"no source"
    )]
    #[test_case(
	    {
	     	let mut h = Hypergraph::<(), (), _>::new();
	     	h.add_hypergraph("zero", []).unwrap();
	     	h.add_hypergraph("one", [0]).unwrap();
	     	h.add_link([0, 0], [0], (), [])
	    }, //
	    AddError::Unlinkable(vec![0, 0], vec![0]); //
    	"unlinkable"
    )]
    fn add_error(result: Result<Vec<usize>, AddError>, expected: AddError) {
        println!("add output: {:?}", result);
        assert_eq!(result.err().unwrap(), expected);
    }

    #[test]
    fn add_hypergraph() {
        let mut h = Hypergraph::<u8, u8, _>::new();
        let id = h.add_hypergraph("zero", []).unwrap();
        assert_eq!(h.hypergraph_value(&id).unwrap(), &Some("zero"));
    }

    #[test]
    fn add_link() {
        let mut h = Hypergraph::<_, _, (), _>::new();
        h.add_node("zero", []).unwrap();
        h.add_node("one", []).unwrap();
        h.add_edge([0], [1], "two", []).unwrap();
        let link_id = h.add_link([0], [2], "three", []).unwrap();
        assert_eq!(h.link_value(link_id).unwrap(), &Some("three"));
    }
    #[test]
    fn add_node() {
        let mut h = Hypergraph::<_, u8>::new();
        let id = h.add_node("zero", []).unwrap();
        assert_eq!(h.node_value(id).unwrap(), &"zero");
    }

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

    #[test]
    fn remove() {
        let mut h = Hypergraph::new();
        h.add_node("zero", []).unwrap();
        h.add_node("one", []).unwrap();
        h.add_edge([0], [1], "two", []).unwrap();
        h.add_link([0], [2], "five", []).unwrap();
        h.add_hypergraph("six", []).unwrap();

        assert_eq!(h.remove([5]), true); // Ok(Some("five")));
        assert_eq!(h.remove([2]), true); // Ok("two"));
        assert_eq!(h.remove([0]), true); // Ok("zero"));
        assert_eq!(h.remove([6]), true); // Ok(Some("six")));

        assert_eq!(h.ids().collect::<Vec<_>>(), vec![vec![], vec![1]]);
    }

    #[test]
    fn set_edge_value() {
        let mut h = Hypergraph::<_, _>::new();
        h.add_node("zero", []).unwrap();
        h.add_node("one", []).unwrap();
        h.add_edge([0], [1], "two", []).unwrap();
        assert_eq!(h.edge_value([2]), Some(&"two"));
        assert_eq!(h.set_edge_value([2], "new_two"), Some("two"));
        assert_eq!(h.edge_value([2]), Some(&"new_two"));
        assert_eq!(h.neighbors([2]).unwrap().next(), Some(&vec![1]));
    }

    #[test]
    fn set_hypergraph_value() {
        let mut h = Hypergraph::<_, _, _>::new();
        h.add_hypergraph("zero", []).unwrap();
        h.add_node("one", []).unwrap();
        h.add_edge([0], [1], "two", []).unwrap();
        assert_eq!(h.hypergraph_value([0]), Some(&Some("zero")));
        assert_eq!(h.set_hypergraph_value([0], "new_zero"), Some(Some("zero")));
        assert_eq!(h.hypergraph_value([0]), Some(&Some("new_zero")));
        assert_eq!(h.neighbors([0]).unwrap().next(), Some(&vec![2]));
    }

    #[test]
    fn set_link_value() {
        let mut h = Hypergraph::<_, _, (), _>::new();
        h.add_node("zero", []).unwrap();
        h.add_node("one", []).unwrap();
        h.add_edge([0], [1], "two", []).unwrap();
        assert_eq!(h.link_value([3]), Some(&None));
        assert_eq!(h.set_link_value([3], "new_three"), Some(None));
        assert_eq!(h.link_value([3]), Some(&Some("new_three")));
    }

    #[test]
    fn set_node_value() {
        let mut h = Hypergraph::<_, _>::new();
        h.add_node("zero", []).unwrap();
        h.add_node("one", []).unwrap();
        h.add_edge([0], [1], "two", []).unwrap();
        assert_eq!(h.node_value([0]), Some(&"zero"));
        assert_eq!(h.set_node_value([0], "new_zero"), Some("zero"));
        assert_eq!(h.node_value([0]), Some(&"new_zero"));
        assert_eq!(h.neighbors([0]).unwrap().next(), Some(&vec![2]));
        assert_eq!(h.node_value([1]), Some(&"one"));
    }
}
