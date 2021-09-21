use core::fmt::Debug;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Marker for main hypergrpah
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Main;

/// Marker for sub hypergrpah
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Sub;

pub trait HypergraphClass: Debug + Eq {
    fn new() -> Self;
    fn is_main(&self) -> bool {
        false
    }
    fn is_sub(&self) -> bool {
        false
    }
}

impl HypergraphClass for Main {
    fn new() -> Self {
        Main
    }
    fn is_main(&self) -> bool {
        true
    }
}
impl HypergraphClass for Sub {
    fn new() -> Self {
        Sub
    }
    fn is_sub(&self) -> bool {
        true
    }
}

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
/// - `Ty`: Main or sub hypergrpah marker
///
/// # Indices
///
/// Indices are represented by `Vec<usize>` by default. They are stable except upon usage
/// of any method under [`Optimization`](#optimization) (like [`shrink_to_fit`]).
///
/// # Contents
///
/// - [`Build`](#build)
/// - [`Create`](#create)
/// - [`Clear`](#clear)
/// - [`Find`](#find)
/// - [`Get`](#get)
/// - [`Inform`](#inform)
/// - [`Optimization`](#optimization)
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
    /// nodes: their weight and edge neighbors (in absolute format)
    nodes: IndexMap<usize, (N, Vec<(Vec<usize>, Direction)>)>,
    /// edges: weight and (node or hypergraph) neighbors (in absolute format)
    edges: IndexMap<usize, (E, Vec<(Vec<usize>, Direction)>)>, // This vector alsways has at least two elements
    /// links: weight, source and target
    links: IndexMap<usize, (Option<L>, Vec<usize>, Vec<usize>)>, // Links have no neighbors
    /// subhypergraps: subhypergraph and eedge neighbours
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
    fn add_local_element(&mut self, element: Element<N, E, H, L, Vec<usize>>) -> usize {
        match self {
            HypergraphEnum::Original(h) => h.add_local_element(element),
            HypergraphEnum::Sub(h) => h.add_local_element(element),
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
pub enum Element<N, E, H, L, Id> {
    /// A graph edge.
    Edge { value: E },
    /// A hypergraph.
    Hypergraph { value: Option<H> },
    /// A graph link.
    Link {
        source: Id,
        target: Id,
        value: Option<L>,
    },
    /// A graph node.
    Node { value: N },
}

impl<N, E, H, L, Id> Element<N, E, H, L, Id> {
    pub fn is_edge(&self) -> bool {
        matches!(self, Element::Edge { .. })
    }

    pub fn is_hypergraph(&self) -> bool {
        matches!(self, Element::Hypergraph { .. })
    }

    pub fn is_link(&self) -> bool {
        matches!(self, Element::Link { .. })
    }

    pub fn is_node(&self) -> bool {
        matches!(self, Element::Node { .. })
    }
    pub fn source(&self) -> Option<&Id> {
        match self {
            Element::Link { source, .. } => Some(&source),
            Element::Edge { .. } | Element::Hypergraph { .. } | Element::Node { .. } => None,
        }
    }

    pub fn target(&self) -> Option<&Id> {
        match self {
            Element::Link { target, .. } => Some(&target),
            Element::Edge { .. } | Element::Hypergraph { .. } | Element::Node { .. } => None,
        }
    }
}

impl<N, E, H, L, Id> From<ElementExt<N, E, H, L, Id>> for Element<N, E, H, L, Id> {
    fn from(element_ext: ElementExt<N, E, H, L, Id>) -> Self {
        match element_ext {
            ElementExt::Edge { value, .. } => Element::Edge { value },
            ElementExt::Link {
                source,
                target,
                value,
            } => Element::Link {
                source,
                target,
                value,
            },
            ElementExt::Hypergraph { value } => Element::Hypergraph { value },
            ElementExt::Node { value } => Element::Node { value },
        }
    }
}

// impl TryInto<ElementExt> for Element ...

#[derive(Debug)]
pub enum ElementType {
    Edge,
    Hypergraph,
    Link,
    Node,
}

impl ElementType {
    pub fn wrapping_next(self) -> Self {
        match self {
            ElementType::Edge => ElementType::Hypergraph,
            ElementType::Hypergraph => ElementType::Link,
            ElementType::Link => ElementType::Node,
            ElementType::Node => ElementType::Edge,
        }
    }
}

impl<N, E, H, L, Id> From<Element<N, E, H, L, Id>> for ElementType {
    fn from(element: Element<N, E, H, L, Id>) -> Self {
        match element {
            Element::Edge { .. } => ElementType::Edge,
            Element::Link { .. } => ElementType::Link,
            Element::Hypergraph { .. } => ElementType::Hypergraph,
            Element::Node { .. } => ElementType::Node,
        }
    }
}

/// Element extended with information to be added to a hypergraph.
///
/// `Edge` variant now has `source` and `target`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElementExt<N, E, H, L, Id> {
    /// A graph edge.
    Edge { source: Id, target: Id, value: E },
    /// A hypergraph.
    Hypergraph { value: Option<H> },
    /// A graph link.
    Link {
        source: Id,
        target: Id,
        value: Option<L>,
    },
    /// A graph node.
    Node { value: N },
}

impl<N, E, H, L, Id> ElementExt<N, E, H, L, Id> {
    pub fn is_edge(&self) -> bool {
        matches!(self, ElementExt::Edge { .. })
    }

    pub fn is_hypergraph(&self) -> bool {
        matches!(self, ElementExt::Hypergraph { .. })
    }

    pub fn is_link(&self) -> bool {
        matches!(self, ElementExt::Link { .. })
    }

    pub fn is_node(&self) -> bool {
        matches!(self, ElementExt::Node { .. })
    }

    pub fn into_source(self) -> Option<Id> {
        match self {
            ElementExt::Edge { source, .. } => Some(source),
            ElementExt::Link { source, .. } => Some(source),
            ElementExt::Hypergraph { .. } | ElementExt::Node { .. } => None,
        }
    }
    pub fn into_target(self) -> Option<Id> {
        match self {
            ElementExt::Edge { target, .. } => Some(target),
            ElementExt::Link { target, .. } => Some(target),
            ElementExt::Hypergraph { .. } | ElementExt::Node { .. } => None,
        }
    }

    pub fn source(&self) -> Option<&Id> {
        match self {
            ElementExt::Edge { source, .. } => Some(&source),
            ElementExt::Link { source, .. } => Some(&source),
            ElementExt::Hypergraph { .. } | ElementExt::Node { .. } => None,
        }
    }
    pub fn target(&self) -> Option<&Id> {
        match self {
            ElementExt::Edge { target, .. } => Some(&target),
            ElementExt::Link { target, .. } => Some(&target),
            ElementExt::Hypergraph { .. } | ElementExt::Node { .. } => None,
        }
    }
}

#[derive(Debug, thiserror::Error, Clone)]
pub enum AddError {
    #[error("Failed to add element because the desired location does not corresponds to an existing hypergraph yet (location {0:?}).")]
    NoHypergraph(Vec<usize>),
    #[error("Failed to add link because the source can not be empty.")]
    EmptySource,
    #[error("Failed to add link because the desired source does not exist yet (location {0:?}).")]
    NoSource(Vec<usize>),
    #[error("Failed to add link because the desired source is a link too (location {0:?}).")]
    LinkSource(Vec<usize>),
    #[error("Failed to add link because the source can not be empty.")]
    EmptyTarget,
    #[error("Failed to add link because the desired target does not exist yet (location {0:?}).")]
    NoTarget(Vec<usize>),
    #[error("Failed to add link because the desired target is a link too (location {0:?}).")]
    LinkTarget(Vec<usize>),
    #[error("Failed to add link because the desired pair (source, target) can not be linked (source {0:?}, target {0:?}).")]
    Unlinkable(Vec<usize>, Vec<usize>),
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
    // # Notes
    //
    // This is not public because it breaks invariants of neihgbors.
    fn add_local_edge(&mut self, value: E) -> usize {
        let element = Element::Edge { value };
        self.add_local_element(element)
    }

    // # Notes
    //
    // This is not public because it breaks invariants of neihgbors.
    fn add_local_link(
        &mut self,
        source: Vec<usize>,
        target: Vec<usize>,
        value: Option<L>,
    ) -> usize {
        let element = Element::Link {
            source,
            target,
            value,
        };
        self.add_local_element(element)
    }

    // # Notes
    //
    // This is not public because it is impossible to return a global id.
    fn add_local_node(&mut self, value: N) -> usize {
        let element = Element::Node { value };
        self.add_local_element(element)
    }

    // # Notes
    //
    // This is not public because it is impossible to return a global id.
    fn add_local_hypergraph(&mut self, value: Option<H>) -> usize {
        let element = Element::Hypergraph { value };
        self.add_local_element(element)
    }
}

/// # Build
///
/// A graph that can be extended with further nodes and edges
impl<N, E, H, L> Hypergraph<N, E, H, L, Main> {
    /// Adds an element.
    ///
    /// `location` refers to the `id` of a (sub-)hypergraph where the element should be added.
    /// If it is empty, it refers to `self`.
    ///
    /// # Errors
    ///
    /// If any `id` provided (`location` or within `element`) does not correspond to an element of the hypergraph,
    /// or if `element` is a connection (edge or link) and `source` or `target` can not be connected through `elmenet`.
    //
    // # Note
    //
    // Perform all checks and call the panicking variant.
    pub fn add_element(
        &mut self,
        element: ElementExt<N, E, H, L, Vec<usize>>,
        location: impl AsRef<[usize]>,
    ) -> Result<Vec<usize>, AddError> {
        let location = location.as_ref();
        if self.hypergraph(location).is_none() {
            return Err(AddError::NoHypergraph(location.to_vec()));
        }
        if element.is_node() | element.is_hypergraph() {
            return Ok(self.add_element_unchecked(element, location));
        }
        // Never fails since element is now either edge or link
        let global_source_id = element.source().unwrap();
        let source_option_element = self.element_value(global_source_id);
        // Check of source and target
        let source_element = match source_option_element {
            None => return Err(AddError::NoSource(element.into_source().unwrap())),
            Some(source_element) => match source_element {
                Element::Link { .. } => {
                    return Err(AddError::LinkSource(element.into_source().unwrap()))
                }
                Element::Edge { .. } => {
                    if let ElementExt::Edge { source, target, .. } = element {
                        return Err(AddError::Unlinkable(source, target)); // Edge -> Edge can not be
                    }
                    source_element
                }
                Element::Node { .. } | Element::Hypergraph { .. } => source_element,
            },
        };
        // source_element is either node or hypergrpha. or edge only if element is a link
        // Never fails since element is now either edge or link
        let global_target_id = element.target().unwrap();
        let target_option_element = self.element_value(global_target_id);
        match self.element_value(global_target_id) {
            None => return Err(AddError::NoTarget(element.into_target().unwrap())),
            Some(target_element) => match target_element {
                Element::Link { .. } => {
                    return Err(AddError::LinkTarget(element.into_target().unwrap()))
                }
                Element::Edge { .. } => {
                    if let ElementExt::Edge { source, target, .. } = element {
                        return Err(AddError::Unlinkable(source, target)); // Edge -> Edge can not be
                    }
                }
                Element::Node { .. } | Element::Hypergraph { .. } => (),
            },
        };
        // target_element is either node or hypergrpha, or edge only if element is a link
        let target_element = target_option_element.unwrap();
        // Check that we are not linking edge with edge
        if source_element.is_edge() && target_element.is_edge() {
            if let ElementExt::Link { source, target, .. } = element {
                return Err(AddError::Unlinkable(source, target)); // Edge -> Edge can not be
            }
        }
        // Check that we are linking through an edge
        if element.is_link()
            && (source_element.is_node() || source_element.is_hypergraph())
            && (target_element.is_node() || target_element.is_hypergraph())
        {
            if let ElementExt::Link { source, target, .. } = element {
                return Err(AddError::Unlinkable(source, target)); // (node or h) -> (node or h) can not be
            }
        }
        // Now the connection is valid
        Ok(self.add_element_unchecked(element, location))
    }

    /// Adds an element.
    ///
    /// `location` refers to the `id` of a (sub-)hypergraph where the element should be added.
    /// If it is empty, it refers to `self`.
    ///
    /// # Panics
    ///
    /// If any `id` provided (`location` or within `element`) does not correspond to an element of the hypergraph,
    /// or if `element` is a connection (edge or link) and `source` or `target` can not be connected through `elmenet`.
    pub fn add_element_unchecked(
        &mut self,
        element: ElementExt<N, E, H, L, Vec<usize>>,
        location: impl AsRef<[usize]>,
    ) -> Vec<usize> {
        let location = location.as_ref();
        match element {
            ElementExt::Node { .. } | ElementExt::Hypergraph { .. } => {
                let mut hypergraph = self.hypergraph_mut(location).unwrap();
                let local_id = hypergraph.add_local_element(element.into());
                let mut global_id = location.to_vec();
                global_id.push(local_id);
                return global_id;
            }
            ElementExt::Edge { .. } | ElementExt::Link { .. } => (),
        }

        match element {
            ElementExt::Edge {
                value,
                source,
                target,
            } => {
                let mut hypergraph = self.hypergraph_mut(location).unwrap();
                // Add edge
                let mut global_id = location.to_vec();
                let local_id = hypergraph.add_local_element(Element::Edge { value });
                global_id.push(local_id);
                // Add links together with the edge
                hypergraph.add_local_element(Element::Link {
                    value: None,
                    source: source.clone(),
                    target: global_id.clone(),
                });
                hypergraph.add_local_element(Element::Link {
                    value: None,
                    source: global_id.clone(),
                    target: target.clone(),
                });
                // Add new neighbors to source and target and their links
                self.neighbors_mut(&source)
                    .unwrap()
                    .push((global_id.clone(), Direction::Outgoing));

                self.neighbors_mut(&target)
                    .unwrap()
                    .push((global_id.clone(), Direction::Incoming));
                // Add links
                return global_id;
            }
            ElementExt::Link {
                value,
                source,
                target,
            } => {
                // Add new neighbors to source and target
                self.neighbors_mut(&source)
                    .unwrap()
                    .push((target.clone(), Direction::Outgoing));
                self.neighbors_mut(&target)
                    .unwrap()
                    .push((source.clone(), Direction::Incoming));
                // Add link
                let mut hypergraph = self.hypergraph_mut(location).unwrap();
                let local_id = hypergraph.add_local_element(Element::Link {
                    source,
                    target,
                    value,
                });
                // Return link id
                let mut global_id = location.to_vec();
                global_id.push(local_id);
                return global_id;
            }
            ElementExt::Node { .. } | ElementExt::Hypergraph { .. } => unreachable!(),
        }
    }

    /// Adds an edge to `self`.
    ///
    /// `location` is identifies the hypergraph where this hypergraph will be added.
    /// An empty `location` means the main hypergraph.
    ///
    /// # Remarks
    ///
    /// The returned `Vec<usize>` is the id of the new edge. But in the process of adding this edge,
    /// there are two new links created. Their id can be retrieved by the [`find_link_id`] method.
    /// The location of the new links is the same as the new edge.
    ///
    /// # Errors
    ///
    /// If `source` or `target` do not correspond to linkable elements, or if `location` does not correspond to a hypergraph.
    pub fn add_edge(
        &mut self,
        source: impl AsRef<[usize]>,
        target: impl AsRef<[usize]>,
        value: E,
        location: impl AsRef<[usize]>,
    ) -> Result<Vec<usize>, AddError> {
        let element = ElementExt::Edge {
            source: source.as_ref().to_vec(),
            target: target.as_ref().to_vec(),
            value,
        };
        self.add_element(element, location)
    }

    /// Adds a hypergraph to `self`.
    ///
    /// `location` is identifies the hypergraph where this hypergraph will be added.
    /// An empty `location` means the main hypergraph.
    ///
    /// # Errors
    ///
    /// If `location` does not correspond to a hypergraph.
    pub fn add_hypergraph(
        &mut self,
        value: impl Into<Option<H>>,
        location: impl AsRef<[usize]>,
    ) -> Result<Vec<usize>, AddError> {
        let element = ElementExt::Hypergraph {
            value: value.into(),
        };
        self.add_element(element, location)
    }

    pub fn add_link(
        &mut self,
        source: impl AsRef<[usize]>,
        target: impl AsRef<[usize]>,
        value: impl Into<Option<L>>,
        location: impl AsRef<[usize]>,
    ) -> Result<Vec<usize>, AddError> {
        let element = ElementExt::Link {
            source: source.as_ref().to_vec(),
            target: target.as_ref().to_vec(),
            value: value.into(),
        };
        self.add_element(element, location)
    }

    /// Adds a node to `self`.
    ///
    /// `location` is identifies the hypergraph where this node will be added.
    /// An empty `location` means the main hypergraph.
    ///
    /// # Errors
    ///
    /// If location does not correspond to a hypergraph.
    pub fn add_node(
        &mut self,
        value: N,
        location: impl AsRef<[usize]>,
    ) -> Result<Vec<usize>, AddError> {
        let element = ElementExt::Node { value };
        self.add_element(element, location)
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

// mod iter {
//     use crate::Hypergraph;

//     /// Iterator over the ids of a hypergraph.
//     ///
//     /// Iterator element type is `Vec<usize>`.
//     ///
//     /// Created with [`.ids()`][1].
//     ///
//     /// [1]: struct.G\Hypergraph.html#method.ids
//     #[derive(Debug)]
//     pub struct IdIter<'a, N, E, H, L, Ty> {
//         /// Original hypergraph (main or sub)
//         original: &'a Hypergraph<N, E, H, L, Ty>,
//         /// Id of original
//         original_id: Vec<usize>,
//         /// Local subhypergraph for ease of access
//         local_subhypergraph: Option<&'a Hypergraph<N, E, H, L, Sub>>,
//         /// Id of local_subhypergraph
//         local_hypergraph_id: Vec<usize>,
//         /// Current type of element to cycle
//         current_element_type: ElementType,
//         /// Next local_id: None means there are no more elements in original
//         next_local_id: Option<usize>,
//     }

//     impl<'a, N, E, H, L, Ty> Iterator for IdIter<&'a Hypergraph<N, E, H, L, Ty>> {
//         type Item = Vec<usize>;
//         fn next(&mut self) -> Option<<Self::Item> {
//             match self.next_local_id {
//                  Some(local_id) => {
//                     // Wrap up the return value
//                     let mut result = self.original_id.clone();
//                     result.extend(&local_subhypergraph_id);
//                     result.push(local_id);
//                     // Search for next element
//                     let hypergraph = self.original.hypergraph(local_hypergraph_id);

//                     if let Some(h) = self.local_subhypergraph {
//                         match self.current_element_type {
//                             ElementType::Edge => {
//                                 let local_index = h.edges.get_index_of(local_id);
//                                 match h.edges.get_index(local_index + 1) {
//                                     Some((next_local_id, _)) => {
//                                         self.next_local_id = next_local_id;
//                                     }
//                                     None => {
//                                         !!!!!
//                                     }
//                                 }
//                                 if local_index < h.edge_count() - 1 {
//                                     self.next_local_id = Some(lo)
//                                 }

//                             }
//                             _ => todo!(),
//                         }
//                     } else {
//                         self.original.element_value()
//                     }

//                     return result
//                  }
//                  None => return None,
//              }
//             let next = self.next.clone();
//             let local_id =
//             self.next =
//             next
//         }
//     }

//     /// A “walker” object that can be used to step through a hypergraph without borrowing it.
//     ///
//     /// Created with [`.detach()`](struct.IdIter.html#method.detach).
//     #[derive(Debug)]
//     pub struct IdWalker {
//         skip_start: Vec<usize>,
//         next: Vec<usize>,
//     }
// }

/// # Get
///
/// Access node and edge weights (associated data).
impl<N, E, H, L, Ty> Hypergraph<N, E, H, L, Ty> {
    // pub fn next_id(&self, id: impl AsRef<[usize]>) -> Option<Vec<usize>> {
    //     self.element_type(id);
    // }

    // pub fn ids<'a>(&'a self) -> IdIter<'a, Self> {
    //     todo!()
    // }

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
    ) -> Option<HypergraphEnum<&Self, &Hypergraph<N, E, H, L, Sub>>> {
        let id = id.as_ref();
        if id.is_empty() {
            return Some(HypergraphEnum::Original(&self));
        }
        match self.subhypergraph(id) {
            Some(h) => Some(HypergraphEnum::Sub(h)),
            None => None,
        }
    }

    /// Returns the hypergraph with id `id`, if it exists.
    ///
    /// `None` is returned when the element does not exists.
    pub fn hypergraph_mut(
        &mut self,
        id: impl AsRef<[usize]>,
    ) -> Option<HypergraphEnum<&mut Self, &mut Hypergraph<N, E, H, L, Sub>>> {
        let id = id.as_ref();
        if id.is_empty() {
            return Some(HypergraphEnum::Original(self));
        }
        match self.subhypergraph_mut(id) {
            Some(h) => Some(HypergraphEnum::Sub(h)),
            None => None,
        }
    }

    /// Returns the neighbors of an element of the current hypergraph, `None` if the element does not exists or is a link.
    pub fn neighbors_local(&self, local_id: usize) -> Option<&Vec<(Vec<usize>, Direction)>> {
        match self.element_value(&[local_id]) {
            None => None,
            Some(element) => match element {
                Element::Edge { .. } => Some(&self.edges.get(&local_id).unwrap().1),
                Element::Hypergraph { .. } => Some(&self.hypergraphs.get(&local_id).unwrap().1),
                Element::Node { .. } => Some(&self.nodes.get(&local_id).unwrap().1),
                Element::Link { .. } => None,
            },
        }
    }

    /// Returns the neighbors of an element of the current hypergraph, `None` if the element does not exists or is a link.
    pub fn neighbors_local_mut(
        &mut self,
        local_id: usize,
    ) -> Option<&mut Vec<(Vec<usize>, Direction)>> {
        match self.element_value_mut(&[local_id]) {
            None => None,
            Some(element) => match element {
                // Never fails by matching
                Element::Edge { .. } => Some(&mut self.edges.get_mut(&local_id).unwrap().1),
                Element::Hypergraph { .. } => {
                    Some(&mut self.hypergraphs.get_mut(&local_id).unwrap().1)
                }
                Element::Node { .. } => Some(&mut self.nodes.get_mut(&local_id).unwrap().1),
                Element::Link { .. } => None,
            },
        }
    }

    pub fn neighbors(&self, id: impl AsRef<[usize]>) -> Option<&Vec<(Vec<usize>, Direction)>> {
        let id = id.as_ref();
        if id.is_empty() {
            return None;
        }
        let local_id = id.last().unwrap(); // Never fails since id is not empty.
        let hypergraph = match self.hypergraph(&id[0..id.len() - 1]) {
            Some(h) => h,
            None => return None,
        };

        if let Some(edge_full) = hypergraph.raw_edges().get(local_id) {
            let (_, ref neighbors) = edge_full;
            return Some(neighbors);
        } else if let Some(hypergraph_full) = hypergraph.raw_hypergraphs().get(local_id) {
            let (_, ref neighbors) = hypergraph_full;
            return Some(neighbors);
        } else if let Some(node_full) = hypergraph.raw_nodes().get(local_id) {
            let (_, ref neighbors) = node_full;
            return Some(neighbors);
        } else {
            return None;
        }
    }

    /// Returns `None` if `id` is empty.
    pub fn neighbors_mut(
        &mut self,
        id: impl AsRef<[usize]>,
    ) -> Option<&mut Vec<(Vec<usize>, Direction)>> {
        let id = id.as_ref();

        match id.len() {
            0 => None,
            1 => {
                let hypergraph = self;
                let local_id = id.last().unwrap(); // Never fails since it is not empty
                if let Some(edge_full) = hypergraph.edges.get_mut(local_id) {
                    let (_, ref mut neighbors) = edge_full;
                    return Some(neighbors);
                } else if let Some(hypergraph_full) = hypergraph.hypergraphs.get_mut(local_id) {
                    let (_, ref mut neighbors) = hypergraph_full;
                    return Some(neighbors);
                } else if let Some(node_full) = hypergraph.nodes.get_mut(local_id) {
                    let (_, ref mut neighbors) = node_full;
                    return Some(neighbors);
                } else {
                    return None;
                }
            }
            _ => {
                let hypergraph = match self.subhypergraph_mut(&id[0..id.len() - 1]) {
                    Some(h) => h,
                    None => {
                        return None;
                    }
                };
                let local_id = id.last().unwrap(); // Never fails since it is not empty
                if let Some(edge_full) = hypergraph.edges.get_mut(local_id) {
                    let (_, ref mut neighbors) = edge_full;
                    return Some(neighbors);
                } else if let Some(hypergraph_full) = hypergraph.hypergraphs.get_mut(local_id) {
                    let (_, ref mut neighbors) = hypergraph_full;
                    return Some(neighbors);
                } else if let Some(node_full) = hypergraph.nodes.get_mut(local_id) {
                    let (_, ref mut neighbors) = node_full;
                    return Some(neighbors);
                } else {
                    return None;
                }
            }
        }
    }

    pub fn element_type(&self, id: impl AsRef<[usize]>) -> Option<ElementType> {
        self.element_value(id)
            .map(|element| -> ElementType { element.into() })
    }

    pub fn element_value(
        &self,
        id: impl AsRef<[usize]>,
    ) -> Option<Element<&N, &E, &H, &L, &Vec<usize>>> {
        let id = id.as_ref();
        if id.is_empty() {
            return Some(Element::Hypergraph {
                value: self.value().as_ref(),
            });
        }

        let local_id = id.last().unwrap(); // Never fails by previous check

        let hypergraph = match self.hypergraph(&id[0..id.len() - 1]) {
            None => return None,
            Some(h) => h,
        };

        let element;
        if let Some(edge_full) = hypergraph.raw_edges().get(local_id) {
            element = Element::Edge {
                value: &edge_full.0,
            };
            return Some(element);
        } else if let Some(hypergraph_full) = hypergraph.raw_hypergraphs().get(local_id) {
            element = Element::Hypergraph {
                value: hypergraph_full.0.value.as_ref(),
            };
            return Some(element);
        } else if let Some(link_full) = hypergraph.raw_links().get(local_id) {
            element = Element::Link {
                source: &link_full.1,
                target: &link_full.2,
                value: (&link_full.0).as_ref(),
            };
            return Some(element);
        } else if let Some(node_full) = hypergraph.raw_nodes().get(local_id) {
            element = Element::Node {
                value: &node_full.0,
            };
            return Some(element);
        };
        None
    }

    pub fn edge_value(&self, id: impl AsRef<[usize]>) -> Option<&E> {
        let id = id.as_ref();
        let local_id = match id.last() {
            Some(local_id) => local_id,
            None => return None,
        };
        if id.len() == 1 {
            return self.edges.get(local_id).map(|edge_full| &edge_full.0);
        } else {
            if let Some(h) = self.subhypergraph(&id[0..id.len() - 1]) {
                return h.edge_value(&[*local_id]);
            };
        }
        None
    }

    pub fn link_value(&self, id: impl AsRef<[usize]>) -> Option<&Option<L>> {
        let id = id.as_ref();
        let local_id = match id.last() {
            Some(local_id) => local_id,
            None => return None,
        };
        if id.len() == 1 {
            return self.links.get(local_id).map(|link_full| &link_full.0);
        } else {
            if let Some(h) = self.subhypergraph(&id[0..id.len() - 1]) {
                return h.link_value(&[*local_id]);
            };
        }
        None
    }

    /// Returns the subgraph with id `id`, if it exists.
    ///
    /// `None` is returned when `id` is empty, or there is no (sub-)hypergraph with such `id`.
    pub fn subhypergraph(&self, id: impl AsRef<[usize]>) -> Option<&Hypergraph<N, E, H, L, Sub>> {
        let id = id.as_ref();
        let mut hypergraph = None;
        for local_id in id {
            hypergraph = self.hypergraphs.get(local_id).map(|h_full| &h_full.0);
            if hypergraph.is_none() {
                return None;
            }
        }
        hypergraph
    }

    pub fn hypergraph_value(&self, id: impl AsRef<[usize]>) -> Option<&Option<H>> {
        let id = id.as_ref();
        let local_id = match id.last() {
            Some(local_id) => local_id,
            None => return Some(&self.value),
        };
        if id.len() == 1 {
            return self
                .hypergraphs
                .get(local_id)
                .map(|hypergraph_full| &hypergraph_full.0.value);
        } else {
            if let Some(h) = self.subhypergraph(&id[0..id.len() - 1]) {
                return h.hypergraph_value(&[*local_id]);
            };
        }
        None
    }

    pub fn node_value(&self, id: impl AsRef<[usize]>) -> Option<&N> {
        let id = id.as_ref();
        let local_id = match id.last() {
            Some(local_id) => local_id,
            None => return None,
        };
        if id.len() == 1 {
            return self.nodes.get(local_id).map(|node_full| &node_full.0);
        } else {
            if let Some(h) = self.subhypergraph(&id[0..id.len() - 1]) {
                return h.node_value(&[*local_id]);
            };
        }
        None
    }

    pub fn element_value_mut(
        &mut self,
        id: impl AsRef<[usize]>,
    ) -> Option<Element<&mut E, &mut H, &mut L, &mut N, &mut Vec<usize>>> {
        todo!()
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

    pub fn raw_edges_mut<'a>(
        &'a mut self,
    ) -> &'a mut IndexMap<usize, (E, Vec<(Vec<usize>, Direction)>)> {
        &mut self.edges
    }

    pub fn raw_links_mut(&mut self) -> &mut IndexMap<usize, (Option<L>, Vec<usize>, Vec<usize>)> {
        &mut self.links
    }

    pub fn raw_hypergraphs_mut(
        &mut self,
    ) -> &mut IndexMap<usize, (Hypergraph<N, E, H, L, Sub>, Vec<(Vec<usize>, Direction)>)> {
        &mut self.hypergraphs
    }

    pub fn raw_nodes_mut(&mut self) -> &mut IndexMap<usize, (N, Vec<(Vec<usize>, Direction)>)> {
        &mut self.nodes
    }

    pub fn edge_value_mut(&mut self, id: impl AsRef<[usize]>) -> Option<&mut E> {
        todo!()
    }

    pub fn link_value_mut(&mut self, id: impl AsRef<[usize]>) -> Option<&mut L> {
        todo!()
    }

    pub fn hypergraph_value_mut(&mut self, id: impl AsRef<[usize]>) -> Option<&mut H> {
        todo!()
    }

    pub fn node_value_mut(&mut self, id: impl AsRef<[usize]>) -> Option<&mut N> {
        todo!()
    }

    /// Returns the subgraph with id `id`, if it exists.
    ///
    /// `None` is returned when `id` is empty, or there is no (sub-)hypergraph with such `id`.
    ///
    /// Notice that the type of `hypergraph` is defined dynamically.
    pub fn subhypergraph_mut(
        &mut self,
        id: impl AsRef<[usize]>,
    ) -> Option<&mut Hypergraph<N, E, H, L, Sub>> {
        let id = id.as_ref();
        if id.is_empty() {
            None
        } else {
            let mut subhypergraph = self.hypergraphs.get_mut(&id[0]).map(|h_full| &mut h_full.0);
            if subhypergraph.is_none() {
                return None;
            }
            for local_id in id.into_iter().skip(1) {
                subhypergraph = subhypergraph
                    .unwrap()
                    .hypergraphs
                    .get_mut(local_id)
                    .map(|h_full| &mut h_full.0);
                if subhypergraph.is_none() {
                    return None;
                }
            }
            subhypergraph
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
        new_value: N,
    ) -> Option<Element<N, E, H, L, Vec<usize>>> {
        todo!()
    }

    pub fn set_node_value(&mut self, id: impl AsRef<[usize]>, new_value: N) -> Option<N> {
        todo!()
    }

    pub fn set_edge_value(&mut self, id: impl AsRef<[usize]>, new_value: E) -> Option<E> {
        todo!()
    }

    pub fn set_link_value(&mut self, id: impl AsRef<[usize]>, new_value: L) -> Option<L> {
        todo!()
    }

    pub fn set_hypergraph_value(&mut self, id: impl AsRef<[usize]>, new_value: H) -> Option<H> {
        todo!()
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
    fn contains_linkable_element(&self, id: impl AsRef<[usize]>) -> bool {
        let id = id.as_ref();
        self.contains_edge(id) | self.contains_hypegraph(id) | self.contains_node(id)
    }

    pub fn contains_element(&self, id: impl AsRef<[usize]>) -> bool {
        let id = id.as_ref();
        self.contains_edge(id)
            | self.contains_link(id)
            | self.contains_hypegraph(id)
            | self.contains_node(id)
    }

    pub fn contains_node(&self, id: impl AsRef<[usize]>) -> bool {
        let id = id.as_ref();
        if id.is_empty() {
            return false;
        }
        let local_id = id.last().unwrap(); // Never fails since id is not empty.
        if let Some(h) = self.hypergraph(&id[0..id.len() - 1]) {
            h.raw_nodes().contains_key(local_id)
        } else {
            false
        }
    }

    pub fn contains_edge(&self, id: impl AsRef<[usize]>) -> bool {
        let id = id.as_ref();
        if id.is_empty() {
            return false;
        }
        let local_id = id.last().unwrap(); // Never fails since id is not empty.
        if let Some(h) = self.hypergraph(&id[0..id.len() - 1]) {
            h.raw_edges().contains_key(local_id)
        } else {
            false
        }
    }

    pub fn contains_link(&self, id: impl AsRef<[usize]>) -> bool {
        let id = id.as_ref();
        if id.is_empty() {
            return false;
        }
        let local_id = id.last().unwrap(); // Never fails since id is not empty.
        if let Some(h) = self.hypergraph(&id[0..id.len() - 1]) {
            h.raw_links().contains_key(local_id)
        } else {
            false
        }
    }

    pub fn contains_hypegraph(&self, id: impl AsRef<[usize]>) -> bool {
        let id = id.as_ref();
        if id.is_empty() {
            return true;
        }
        let local_id = id.last().unwrap(); // Never fails since id is not empty.
        if let Some(h) = self.hypergraph(&id[0..id.len() - 1]) {
            h.raw_hypergraphs().contains_key(local_id)
        } else {
            false
        }
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
    /// Returns the id of the link that belongs to hypergrpah `location` linking `source` and `target`.
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
            Nonen => return None,
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
    pub fn find_element_by_value(
        &self,
        value: &Element<N, E, H, L, Vec<usize>>,
    ) -> Option<Vec<usize>> {
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

/// # Remove
///
/// Remove elements.
impl<N, E, H, L, Ty> Hypergraph<N, E, H, L, Ty> {
    pub fn remove_element(&mut self, id: impl AsRef<[usize]>) -> bool {
        todo!()
    }

    pub fn remove_edge(&mut self, id: impl AsRef<[usize]>) -> bool {
        todo!()
    }

    pub fn remove_hypergraph(&mut self, id: impl AsRef<[usize]>) -> bool {
        todo!()
    }

    pub fn remove_link(&mut self, id: impl AsRef<[usize]>) -> bool {
        todo!()
    }

    pub fn remove_node(&mut self, id: impl AsRef<[usize]>) -> bool {
        todo!()
    }

    /// Removes the first element matching `value`.
    pub fn remove_element_by_value(&mut self, value: Element<N, E, H, L, Vec<usize>>) -> bool {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test]
    fn new() {
        Hypergraph::<(), ()>::new();
    }

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
        assert_eq!(
            h.neighbors_local(0).unwrap(),
            &vec![(edge_id.clone(), Direction::Outgoing)]
        );
        assert_eq!(
            h.neighbors_local(1).unwrap(),
            &vec![(edge_id, Direction::Incoming)]
        );
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
        let node_0_id = h.add_node("zero", []).unwrap();
        h.add_node("one", []).unwrap();
        let edge_id = h.add_edge([0], [1], "two", []).unwrap();
        let link_id = h.add_link([0], [2], "three", []).unwrap();
        assert_eq!(h.link_value(link_id).unwrap(), &Some("three"));
        assert_eq!(
            h.neighbors_local(0).unwrap(),
            &vec![
                (edge_id.clone(), Direction::Outgoing),
                (edge_id, Direction::Outgoing)
            ]
        );
        assert_eq!(
            h.neighbors_local(2).unwrap(),
            &vec![(node_0_id, Direction::Incoming)]
        );
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
    fn contains_linkable_element() {
        let mut h = Hypergraph::new();
        let node_id = h.add_node("zero", []).unwrap();
        let hypergraph_id = h.add_hypergraph("one", []).unwrap();
        let edge_id = h.add_edge([0], [1], "two", []).unwrap();
        let link_id = h.add_link([0], [2], "three", []).unwrap();
        assert!(h.contains_linkable_element(node_id));
        assert!(h.contains_linkable_element(hypergraph_id));
        assert!(h.contains_linkable_element(edge_id));
        assert!(!h.contains_linkable_element(link_id));
    }
}
