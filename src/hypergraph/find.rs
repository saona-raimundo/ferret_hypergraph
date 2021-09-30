use crate::{elements::ElementValue, errors, Hypergraph};

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
    ) -> Result<Vec<usize>, errors::FindError>
    where
        L: PartialEq,
    {
        let location = location.as_ref();
        if !self.contains_hypergraph(location) {
            Err(errors::NoHypergraph(location.to_vec()))?
        }
        let hypergraph = self.hypergraph(location).unwrap(); // Never fails since location refers to a hypergraph
        let links = hypergraph.raw_links();
        let source = source.as_ref().to_vec();
        let target = target.as_ref().to_vec();
        for (local_id, link_full) in links {
            if (link_full.0.as_ref(), &link_full.1, &link_full.2)
                == (value.as_ref(), &source, &target)
            {
                let mut location = location.to_vec();
                location.push(*local_id);
                return Ok(location);
            }
        }
        Err(errors::FindError::NoLink)
    }

    pub fn find_element_by_value(
        &self,
        value: ElementValue<&N, &E, &H, &L>,
    ) -> Result<Vec<usize>, errors::FindError> {
        todo!()
    }

    pub fn find_node_by_value(&self, value: &N) -> Result<Vec<usize>, errors::FindError> {
        todo!()
    }

    pub fn find_edge_by_value(&self, value: &E) -> Result<Vec<usize>, errors::FindError> {
        todo!()
    }

    pub fn find_link_by_value(&self, value: &Option<L>) -> Result<Vec<usize>, errors::FindError> {
        todo!()
    }

    pub fn find_hypergraph_by_value(
        &self,
        value: &Option<H>,
    ) -> Result<Vec<usize>, errors::FindError> {
        todo!()
    }
}
