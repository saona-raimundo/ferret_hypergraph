use thiserror::Error;

#[derive(Debug, thiserror::Error, Clone)]
#[error("Failed to create an iterator over neighbors because the desired id does not corresponds to an existing linkable element yet (location {0:?}).")]
pub struct NoElementLinkable(pub Vec<usize>);

#[derive(Debug, thiserror::Error, Clone)]
#[error("Failed to convert to ElementLinkable because element is a link.")]
pub struct LinkPresent;

#[derive(Debug, Error, Clone)]
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
