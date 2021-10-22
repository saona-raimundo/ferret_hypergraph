use core::mem;

use crate::{elements::ElementValue, errors, Hypergraph};

/// # Set
///
/// Set the value of elements
impl<N, E, H, L, Ty> Hypergraph<N, E, H, L, Ty> {
    pub fn set_edge_value(
        &mut self,
        id: impl AsRef<[usize]>,
        mut new_value: E,
    ) -> Result<E, errors::SetError> {
        let id = id.as_ref();
        if !self.contains_edge(id) {
            Err(errors::NoEdge(id.to_vec()))?
        }
        let old_value = self.edge_value_mut(id).unwrap(); // Never fails since id refers to an edge
        mem::swap(old_value, &mut new_value);
        Ok(new_value)
    }

    pub fn set_element_value(
        &mut self,
        id: impl AsRef<[usize]>,
        new_value: ElementValue<N, E, H, L>,
    ) -> Result<ElementValue<N, E, H, L>, errors::SetError> {
        match new_value {
            ElementValue::Edge { value } => {
                let old_value = self.set_edge_value(id, value)?;
                Ok(ElementValue::Edge { value: old_value })
            }
            ElementValue::Hypergraph { value } => {
                let old_value = self.set_hypergraph_value(id, value)?;
                Ok(ElementValue::Hypergraph { value: old_value })
            }
            ElementValue::Link { value } => {
                let old_value = self.set_link_value(id, value)?;
                Ok(ElementValue::Link { value: old_value })
            }
            ElementValue::Node { value } => {
                let old_value = self.set_node_value(id, value)?;
                Ok(ElementValue::Node { value: old_value })
            }
        }
    }

    pub fn set_link_value(
        &mut self,
        id: impl AsRef<[usize]>,
        new_value: impl Into<Option<L>>,
    ) -> Result<Option<L>, errors::SetError> {
        let id = id.as_ref();
        let mut new_value = new_value.into();
        if !self.contains_link(id) {
            Err(errors::NoLink(id.to_vec()))?
        }
        let old_value = self.link_value_mut(id).unwrap(); // Never fails since id refers to a link
        mem::swap(old_value, &mut new_value);
        Ok(new_value)
    }

    pub fn set_hypergraph_value(
        &mut self,
        id: impl AsRef<[usize]>,
        new_value: impl Into<Option<H>>,
    ) -> Result<Option<H>, errors::SetError> {
        let id = id.as_ref();
        let mut new_value = new_value.into();
        if !self.contains_hypergraph(id) {
            Err(errors::NoHypergraph(id.to_vec()))?
        }
        let old_value = self.hypergraph_value_mut(id).unwrap(); // Never fails since id refers to a link
        mem::swap(old_value, &mut new_value);
        Ok(new_value)
    }

    pub fn set_node_value(
        &mut self,
        id: impl AsRef<[usize]>,
        mut new_value: N,
    ) -> Result<N, errors::SetError> {
        let id = id.as_ref();
        if !self.contains_node(id) {
            Err(errors::NoNode(id.to_vec()))?
        }
        let old_value = self.node_value_mut(id).unwrap(); // Never fails since id refers to a link
        mem::swap(old_value, &mut new_value);
        Ok(new_value)
    }

    /// Change the value of the hypergraph as a whole.
    pub fn set_value(&mut self, new_value: impl Into<Option<H>>) -> Option<H> {
        let mut new_value = new_value.into();
        let old_value = self.value_mut();
        mem::swap(old_value, &mut new_value);
        new_value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_edge_value() {
        let mut h = Hypergraph::<_, _>::new();
        h.add_node("zero", []).unwrap();
        h.add_node("one", []).unwrap();
        h.add_edge([0], [1], "two", []).unwrap();
        assert_eq!(h.edge_value([2]), Ok(&"two"));
        assert_eq!(h.set_edge_value([2], "new_two"), Ok("two"));
        assert_eq!(h.edge_value([2]), Ok(&"new_two"));
        assert_eq!(h.neighbors([2]).next(), Some(&vec![1]));
    }

    #[test]
    fn set_hypergraph_value() {
        let mut h = Hypergraph::<_, _, _>::new();
        h.add_hypergraph("zero", []).unwrap();
        h.add_node("one", []).unwrap();
        h.add_edge([0], [1], "two", []).unwrap();
        assert_eq!(h.hypergraph_value([0]), Ok(&Some("zero")));
        assert_eq!(h.set_hypergraph_value([0], "new_zero"), Ok(Some("zero")));
        assert_eq!(h.hypergraph_value([0]), Ok(&Some("new_zero")));
        assert_eq!(h.neighbors([0]).next(), Some(&vec![2]));
    }

    #[test]
    fn set_link_value() {
        let mut h = Hypergraph::<_, _, (), _>::new();
        h.add_node("zero", []).unwrap();
        h.add_node("one", []).unwrap();
        h.add_edge([0], [1], "two", []).unwrap();
        assert_eq!(h.link_value([3]), Ok(&None));
        assert_eq!(h.set_link_value([3], "new_three"), Ok(None));
        assert_eq!(h.link_value([3]), Ok(&Some("new_three")));
    }

    #[test]
    fn set_node_value() {
        let mut h = Hypergraph::<_, _>::new();
        h.add_node("zero", []).unwrap();
        h.add_node("one", []).unwrap();
        h.add_edge([0], [1], "two", []).unwrap();
        assert_eq!(h.node_value([0]), Ok(&"zero"));
        assert_eq!(h.set_node_value([0], "new_zero"), Ok("zero"));
        assert_eq!(h.node_value([0]), Ok(&"new_zero"));
        assert_eq!(h.neighbors([0]).next(), Some(&vec![2]));
        assert_eq!(h.node_value([1]), Ok(&"one"));
    }
}
