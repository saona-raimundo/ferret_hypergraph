use thiserror::Error;

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
#[error("Failed to convert to ElementLinkable because element is a link.")]
pub struct LinkPresent;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("There is no node with id {0:?}.")]
pub struct NoNode(pub Vec<usize>);
