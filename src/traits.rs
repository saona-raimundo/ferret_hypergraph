use core::fmt::Debug;

use crate::{iterators::WalkIter, Hypergraph};

/// Marker trait for classes of Hypergraphs.
pub trait HypergraphClass: Debug + Eq {
    fn new() -> Self;
    fn is_main(&self) -> bool {
        false
    }
    fn is_sub(&self) -> bool {
        false
    }
}

/// A walker is like an iterator, where part of the
/// information is supplied manually at each "next" call (named `walk_next`).
///
/// # Remarks
///
/// This allows to visit a Hypergraph without a fixed reference to it.
pub trait Walker<'a, N, E, H, L, Ty>: Sized {
    type Item;
    /// Advance to the next item.
    fn walk_next(&mut self, hypergraph: &'a Hypergraph<N, E, H, L, Ty>) -> Option<Self::Item>;

    /// Create an iterator fixing a reference to a hypergraph.
    fn build_iter(
        self,
        hypergraph: &'a Hypergraph<N, E, H, L, Ty>,
    ) -> WalkIter<'a, N, E, H, L, Ty, Self> {
        WalkIter::new(self, hypergraph)
    }
}
