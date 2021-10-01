use crate::{Hypergraph, Main};

/// # Clear
///
/// A hypergraph that can be cleared.
impl<N, E, H, L> Hypergraph<N, E, H, L, Main> {
    /// Clears the hypergraph, removing everything inside.
    ///
    /// # Remarks
    ///
    /// This method has no effect on the allocated capacity.
    pub fn clear(&mut self) -> &mut Self {
        self.raw_edges_mut().clear();
        self.raw_hypergraphs_mut().clear();
        self.raw_links_mut().clear();
        self.raw_nodes_mut().clear();
        self
    }

    /// Clears the edges of the top level. Nested hypergraphs remain unchanged.
    ///
    /// # Remarks
    ///
    /// This method has no effect on the allocated capacity.
    pub fn clear_edges(&mut self) -> &mut Self {
        let local_ids: Vec<_> = self.raw_edges().keys().cloned().collect();
        for local_id in local_ids {
            self.remove_edge([local_id]).unwrap(); // Never fails since local_id is valid
        }
        self.raw_edges_mut().clear();
        self
    }

    /// Clears the hypergraphs of the top level. Nested hypergraphs remain unchanged.
    ///
    /// # Remarks
    ///
    /// This method has no effect on the allocated capacity.
    pub fn clear_hypergraphs(&mut self) -> &mut Self {
        let local_ids: Vec<_> = self.raw_hypergraphs().keys().cloned().collect();
        for local_id in local_ids {
            self.remove_subhypergraph([local_id]).unwrap(); // Never fails since local_id is valid
        }
        self.raw_hypergraphs_mut().clear();
        self
    }

    /// Clears the links of the top level. Nested hypergraphs remain unchanged.
    ///
    /// # Remarks
    ///
    /// This method has no effect on the allocated capacity.
    pub fn clear_links(&mut self) -> &mut Self {
        let local_ids: Vec<_> = self.raw_links().keys().cloned().collect();
        for local_id in local_ids {
            self.remove_link([local_id]).unwrap(); // Never fails since local_id is valid
        }
        self.raw_links_mut().clear();
        self
    }

    /// Clears the nodes of the top level. Nested hypergraphs remain unchanged.
    ///
    /// # Remarks
    ///
    /// This method has no effect on the allocated capacity.
    pub fn clear_nodes(&mut self) -> &mut Self {
        let local_ids: Vec<_> = self.raw_nodes().keys().cloned().collect();
        for local_id in local_ids {
            self.remove_node([local_id]).unwrap(); // Never fails since local_id is valid
        }
        self.raw_nodes_mut().clear();
        self
    }
}
