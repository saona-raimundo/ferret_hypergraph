use core::mem;

use crate::{errors, Hypergraph, Main, Sub};

/// # Add
///
/// A graph that can be extended with further nodes and edges
impl<N, E, H, L> Hypergraph<N, E, H, L, Main> {
    /// Clones and adds all elements in `other` into a new hypergraph inside `location`.
    pub fn extend_from_hypegraph<Ty>(
        &mut self,
        other: &Hypergraph<N, E, H, L, Ty>,
        location: impl AsRef<[usize]>,
    ) -> Result<Vec<usize>, errors::AddError>
    where
        N: Clone,
        E: Clone,
        H: Clone,
        L: Clone,
        Ty: Clone,
        Sub: From<Ty>,
    {
        let location = location.as_ref();
        let mut other: Hypergraph<N, E, H, L, Sub> = other.clone().into_sub();
        let new_hypergraph_id = self.add_hypergraph_in(other.value().clone(), &location)?;
        let subhypergraph = self.subhypergraph_mut(&new_hypergraph_id).unwrap(); // Never fails since new_hypergraph_id refers to a hypergraph
        mem::swap(subhypergraph, &mut other);
        subhypergraph.preappend_id(location);

        Ok(new_hypergraph_id)
    }
}
