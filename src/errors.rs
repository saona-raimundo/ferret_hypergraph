use thiserror::Error;

/// # Basic
#[derive(Copy, Debug, Error, Clone, PartialEq, Eq)]
#[error("Source can not be empty.")]
pub struct EmptySource;

#[derive(Copy, Debug, Error, Clone, PartialEq, Eq)]
#[error("Target can not be empty.")]
pub struct EmptyTarget;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("The link is incoherent (location {0:?}, source {1:?}, target {2:?}).")]
pub struct IncoherentLink(pub Vec<usize>, pub Vec<usize>, pub Vec<usize>);

#[derive(Copy, Debug, Error, Clone, PartialEq, Eq)]
#[error("Failed to convert to ElementLinkable because element is a link.")]
pub struct LinkPresent;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("Source can not be a link.")]
pub struct LinkSource(pub Vec<usize>);

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("Target can not be a link.")]
pub struct LinkTarget(pub Vec<usize>);

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("There is no edge with id {0:?}.")]
pub struct NoEdge(pub Vec<usize>);

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("There is no element with id {0:?}.")]
pub struct NoElement(pub Vec<usize>);

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("There is no linkable element with id {0:?}.")]
pub struct NoElementLinkable(pub Vec<usize>);

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("There is no hypergraph with id {0:?}.")]
pub struct NoHypergraph(pub Vec<usize>);

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("There is no link with id {0:?}.")]
pub struct NoLink(pub Vec<usize>);

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("There is no node with id {0:?}.")]
pub struct NoNode(pub Vec<usize>);

#[derive(Copy, Debug, Error, Clone, PartialEq, Eq)]
#[error("The method does not apply to the root hypergraph.")]
pub struct RootHypergraph;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("These elements can not be linked (source {0:?}, target {0:?}).")]
pub struct Unlinkable(pub Vec<usize>, pub Vec<usize>);

/// # Compound
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("Failed to add element.")]
pub enum AddError {
    EmptySource(#[from] EmptySource),
    EmptyTarget(#[from] EmptyTarget),
    IncoherentLink(#[from] IncoherentLink),
    LinkSource(#[from] LinkSource),
    LinkTarget(#[from] LinkTarget),
    NoLocation(#[from] NoHypergraph),
    NoSource(#[source] NoElementLinkable),
    NoTarget(#[source] NoElementLinkable),
    Unlinkable(#[from] Unlinkable),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("Failed to find the element.")]
pub enum FindError {
    NoEdge,
    NoElement,
    NoHypergraph,
    NoLink,
    NoNode,
    NoLocation(#[from] NoHypergraph),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("Failed to get an element.")]
pub enum GetError {
    NoEdge(#[from] NoEdge),
    NoElement(#[from] NoElement),
    NoElementLinkable(#[from] NoElementLinkable),
    NoHypergraph(#[from] NoHypergraph),
    NoLink(#[from] NoLink),
    NoNode(#[from] NoNode),
    RootHypergraph(#[from] RootHypergraph),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("Failed to remove element.")]
pub enum RemoveError {
    NoEdge(#[from] NoEdge),
    NoElement(#[from] NoElement),
    NoHypergraph(#[from] NoHypergraph),
    NoLink(#[from] NoLink),
    NoNode(#[from] NoNode),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("Failed to set an element.")]
pub enum SetError {
    NoEdge(#[from] NoEdge),
    NoElement(#[from] NoElement),
    NoElementLinkable(#[from] NoElementLinkable),
    NoHypergraph(#[from] NoHypergraph),
    NoLink(#[from] NoLink),
    NoNode(#[from] NoNode),
}
