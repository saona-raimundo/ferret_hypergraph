use indexmap::IndexMap;

use crate::{
    direction::Direction,
    elements::{ElementType, ElementValue},
    errors, iterators,
    traits::Walker,
    walkers, Hypergraph, HypergraphEnum, Sub,
};

/// # Get
///
/// Access node and edge weights (associated data).
impl<N, E, H, L, Ty> Hypergraph<N, E, H, L, Ty> {
    /// Returns the class marker.
    pub fn class(&self) -> &Ty {
        &self.class
    }

    // /// Returns an iterator over all valid edge ids.
    // pub fn edge_ids<'a>(&'a self) -> EdgeIterIds<'a, N, E, H, L, Ty> {
    //     EdgeIterIds::new(&self)
    // }

    pub fn edge_value(&self, id: impl AsRef<[usize]>) -> Result<&E, errors::GetError> {
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

    pub fn edge_value_mut(&mut self, id: impl AsRef<[usize]>) -> Result<&mut E, errors::GetError> {
        let id = id.as_ref();
        if !self.contains_edge(&id) {
            Err(errors::NoEdge(id.to_vec()))?
        }
        let hypergraph = self.hypergraph_of_mut(&id).unwrap(); // Never fails since id refers to a valid edge
        let local_id = id.last().unwrap(); // Never fails since id refers to a valid edge
        let edge_value = hypergraph
            .raw_edges_mut()
            .get_mut(local_id)
            .map(|edge_full| &mut edge_full.0)
            .unwrap(); // Never fails since id refers to a valid edge
        Ok(edge_value)
    }

    pub fn element_type(&self, id: impl AsRef<[usize]>) -> Result<ElementType, errors::GetError> {
        self.element_value(id)
            .map(|element| -> ElementType { element.into() })
    }

    pub fn element_value(
        &self,
        id: impl AsRef<[usize]>,
    ) -> Result<ElementValue<&N, &E, &H, &L>, errors::GetError> {
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

    pub fn element_value_mut(
        &mut self,
        id: impl AsRef<[usize]>,
    ) -> Result<ElementValue<&mut N, &mut E, &mut H, &mut L>, errors::GetError> {
        let id = id.as_ref();
        if id.is_empty() {
            return Ok(ElementValue::Hypergraph {
                value: self.value_mut().as_mut(),
            });
        }

        let local_id = id.last().unwrap(); // Never fails by previous check
        let mut hypergraph = self.hypergraph_of_mut(&id)?;

        let element = match hypergraph.element_type([*local_id])? {
            ElementType::Edge => {
                let edge_full = hypergraph.raw_edges_mut().get_mut(local_id).unwrap();
                ElementValue::Edge {
                    value: &mut edge_full.0,
                }
            }
            ElementType::Hypergraph => {
                let hypergraph_full = hypergraph.raw_hypergraphs_mut().get_mut(local_id).unwrap();
                ElementValue::Hypergraph {
                    value: hypergraph_full.0.value.as_mut(),
                }
            }
            ElementType::Link => {
                let link_full = hypergraph.raw_links_mut().get_mut(local_id).unwrap();
                ElementValue::Link {
                    value: link_full.0.as_mut(),
                }
            }
            ElementType::Node => {
                let node_full = hypergraph.raw_nodes_mut().get_mut(local_id).unwrap();
                ElementValue::Node {
                    value: &mut node_full.0,
                }
            }
        };
        Ok(element)
    }

    /// Returns the hypergraph with id `id`, if it exists.
    ///
    /// `None` is returned when the element does not exists.
    pub fn hypergraph(
        &self,
        id: impl AsRef<[usize]>,
    ) -> Result<HypergraphEnum<&Self, &Hypergraph<N, E, H, L, Sub>>, errors::GetError> {
        let id = id.as_ref();
        if id.is_empty() {
            return Ok(HypergraphEnum::Original(&self));
        }
        let h = self.subhypergraph(id)?;
        Ok(HypergraphEnum::Sub(h))
    }

    /// Returns the hypergraph with id `id`, if it exists.
    ///
    /// `None` is returned when the element does not exists.
    pub fn hypergraph_mut(
        &mut self,
        id: impl AsRef<[usize]>,
    ) -> Result<HypergraphEnum<&mut Self, &mut Hypergraph<N, E, H, L, Sub>>, errors::GetError> {
        let id = id.as_ref();
        if id.is_empty() {
            return Ok(HypergraphEnum::Original(self));
        }
        let subhypergraph = self.subhypergraph_mut(&id)?;
        Ok(HypergraphEnum::Sub(subhypergraph))
    }

    /// Returns the hypergraph in which `id` lives, if it exists.
    ///
    /// `None` is returned when: there is no element with id `id`; or `id` is empty.
    pub fn hypergraph_of(
        &self,
        id: impl AsRef<[usize]>,
    ) -> Result<HypergraphEnum<&Self, &Hypergraph<N, E, H, L, Sub>>, errors::GetError> {
        let id = id.as_ref();
        match id.len() {
            0 => Err(errors::RootHypergraph)?,
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
    ) -> Result<HypergraphEnum<&mut Self, &mut Hypergraph<N, E, H, L, Sub>>, errors::GetError> {
        let id = id.as_ref();
        match id.len() {
            0 => Err(errors::RootHypergraph)?,
            1 => Ok(HypergraphEnum::Original(self)),
            _ => {
                let id = &id[0..id.len() - 1];
                let subhypergraph = self.subhypergraph_mut(&id)?;
                Ok(HypergraphEnum::Sub(subhypergraph))
            }
        }
    }

    pub fn hypergraph_value(
        &self,
        id: impl AsRef<[usize]>,
    ) -> Result<&Option<H>, errors::GetError> {
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

    pub fn hypergraph_value_mut(
        &mut self,
        id: impl AsRef<[usize]>,
    ) -> Result<&mut Option<H>, errors::GetError> {
        let id = id.as_ref();
        if !self.contains_hypergraph(id) {
            Err(errors::NoHypergraph(id.to_vec()))?
        }

        match id.len() {
            0 => Ok(self.value_mut()),
            _ => {
                let hypergraph = self.hypergraph_of_mut(&id)?;
                let local_id = id.last().unwrap(); // Never fails since id is non empty.
                let hypergraph_value = hypergraph
                    .raw_hypergraphs_mut()
                    .get_mut(local_id)
                    .map(|hypergraph_full| &mut hypergraph_full.0.value)
                    .unwrap(); // Never fails since id refers to a hypergraph
                Ok(hypergraph_value)
            }
        }
    }

    /// Returns an iterator over all valid ids of `self`.
    pub fn ids<'a>(&'a self) -> iterators::WalkIter<'a, N, E, H, L, Ty, walkers::WalkIds> {
        walkers::WalkIds::new().build_iter(self)
        // IterIds::new(&self)
    }

    /// Returns the pair of gloalbal `id`s `(source, target)` if the link exists.
    pub fn link_endpoints(
        &self,
        id: impl AsRef<[usize]>,
    ) -> Result<(&Vec<usize>, &Vec<usize>), errors::GetError> {
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

    /// Returns the links of an element of the current hypergraph, `None` if the element does not exists or is a link.
    pub fn links_of(
        &self,
        id: impl AsRef<[usize]>,
    ) -> Result<&Vec<(Vec<usize>, Direction)>, errors::GetError> {
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
    ///
    /// # Notes
    ///
    /// Not meant to be public.
    /// Be very careful when using this method! All invariants of the datastructure must hold!
    pub(crate) fn links_of_mut(
        &mut self,
        id: impl AsRef<[usize]>,
    ) -> Result<&mut Vec<(Vec<usize>, Direction)>, errors::GetError> {
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

    pub fn link_value(&self, id: impl AsRef<[usize]>) -> Result<&Option<L>, errors::GetError> {
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

    pub fn link_value_mut(
        &mut self,
        id: impl AsRef<[usize]>,
    ) -> Result<&mut Option<L>, errors::GetError> {
        let id = id.as_ref();
        if !self.contains_link(&id) {
            Err(errors::NoLink(id.to_vec()))?
        }
        let hypergraph = self.hypergraph_of_mut(&id).unwrap(); // Never fails since id refers to a valid link
        let local_id = id.last().unwrap(); // Never fails since id refers to a valid link
        let link_value = hypergraph
            .raw_links_mut()
            .get_mut(local_id)
            .map(|link_full| &mut link_full.0)
            .unwrap(); // Never fails since id refers to a valid link
        Ok(link_value)
    }

    /// Returns an iterator over outgoing neighbors.
    ///
    /// If `id` is not a valid element, the iterator returns always `None`.
    pub fn neighbors<'a>(
        &'a self,
        id: impl AsRef<[usize]>,
    ) -> iterators::WalkIter<'a, N, E, H, L, Ty, walkers::WalkNeighbors> {
        let direction = Direction::Outgoing;
        walkers::WalkNeighbors::new(direction, id).build_iter(self)
    }

    pub fn neighbors_directed<'a>(
        &'a self,
        id: impl AsRef<[usize]>,
        direction: Direction,
    ) -> iterators::WalkIter<'a, N, E, H, L, Ty, walkers::WalkNeighbors> {
        walkers::WalkNeighbors::new(direction, id).build_iter(self)
    }

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

    pub fn node_value(&self, id: impl AsRef<[usize]>) -> Result<&N, errors::GetError> {
        let id = id.as_ref();
        if !self.contains_node(id) {
            Err(errors::NoNode(id.to_vec()))?
        }

        match id.len() {
            0 => Err(errors::RootHypergraph)?,
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

    pub fn node_value_mut(&mut self, id: impl AsRef<[usize]>) -> Result<&mut N, errors::GetError> {
        let id = id.as_ref();
        if !self.contains_node(id) {
            Err(errors::NoNode(id.to_vec()))?
        }

        match id.len() {
            0 => Err(errors::RootHypergraph)?,
            _ => {
                let hypergraph = self.hypergraph_of_mut(&id)?;
                let local_id = id.last().unwrap(); // Never fails since id is non empty.
                let node_value = hypergraph
                    .raw_nodes_mut()
                    .get_mut(local_id)
                    .map(|node_full| &mut node_full.0)
                    .unwrap(); // Never fails since id refers to a hypergraph
                Ok(node_value)
            }
        }
    }

    pub fn raw_edges(&self) -> &IndexMap<usize, (E, Vec<(Vec<usize>, Direction)>)> {
        &self.edges
    }

    pub(crate) fn raw_edges_mut<'a>(
        &'a mut self,
    ) -> &'a mut IndexMap<usize, (E, Vec<(Vec<usize>, Direction)>)> {
        &mut self.edges
    }

    pub fn raw_links(&self) -> &IndexMap<usize, (Option<L>, Vec<usize>, Vec<usize>)> {
        &self.links
    }

    pub(crate) fn raw_links_mut(
        &mut self,
    ) -> &mut IndexMap<usize, (Option<L>, Vec<usize>, Vec<usize>)> {
        &mut self.links
    }

    pub fn raw_hypergraphs(
        &self,
    ) -> &IndexMap<usize, (Hypergraph<N, E, H, L, Sub>, Vec<(Vec<usize>, Direction)>)> {
        &self.hypergraphs
    }

    pub(crate) fn raw_hypergraphs_mut(
        &mut self,
    ) -> &mut IndexMap<usize, (Hypergraph<N, E, H, L, Sub>, Vec<(Vec<usize>, Direction)>)> {
        &mut self.hypergraphs
    }

    pub fn raw_nodes(&self) -> &IndexMap<usize, (N, Vec<(Vec<usize>, Direction)>)> {
        &self.nodes
    }

    pub(crate) fn raw_nodes_mut(
        &mut self,
    ) -> &mut IndexMap<usize, (N, Vec<(Vec<usize>, Direction)>)> {
        &mut self.nodes
    }

    /// Returns the subgraph with id `id`, if it exists.
    ///
    /// `None` is returned when `id` is empty, or there is no (sub-)hypergraph with such `id`.
    pub fn subhypergraph(
        &self,
        id: impl AsRef<[usize]>,
    ) -> Result<&Hypergraph<N, E, H, L, Sub>, errors::GetError> {
        let id = id.as_ref().to_vec();
        match id.len() {
            0 => Err(errors::RootHypergraph)?,
            1 => match self.hypergraphs.get(&id[0]).map(|h_full| &h_full.0) {
                Some(h) => Ok(h),
                None => Err(errors::NoHypergraph(id))?,
            },
            _ => {
                let local_id = id[0]; // Never fails since id is non empty.
                let mut subhypergraph = match self.raw_hypergraphs().get(&local_id) {
                    None => Err(errors::NoHypergraph(vec![local_id]))?,
                    Some(hypergraph_full) => &hypergraph_full.0,
                };
                for (counter, local_id) in id.iter().enumerate().skip(1) {
                    subhypergraph = match subhypergraph.raw_hypergraphs().get(local_id) {
                        None => Err(errors::NoHypergraph(id[0..=counter].to_vec()))?,
                        Some(hypergraph_full) => &hypergraph_full.0,
                    };
                }
                Ok(subhypergraph)
            }
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
    ) -> Result<&mut Hypergraph<N, E, H, L, Sub>, errors::GetError> {
        let id = id.as_ref().to_vec();
        match id.len() {
            0 => Err(errors::RootHypergraph)?,
            1 => match self.hypergraphs.get_mut(&id[0]).map(|h_full| &mut h_full.0) {
                Some(h) => Ok(h),
                None => Err(errors::NoHypergraph(id))?,
            },
            _ => {
                let local_id = id[0]; // Never fails since id is non empty.
                let mut subhypergraph = match self.raw_hypergraphs_mut().get_mut(&local_id) {
                    None => Err(errors::NoHypergraph(vec![local_id]))?,
                    Some(hypergraph_full) => &mut hypergraph_full.0,
                };
                for (counter, local_id) in id.iter().enumerate().skip(1) {
                    subhypergraph = match subhypergraph.raw_hypergraphs_mut().get_mut(local_id) {
                        None => Err(errors::NoHypergraph(id[0..=counter].to_vec()))?,
                        Some(hypergraph_full) => &mut hypergraph_full.0,
                    };
                }
                Ok(subhypergraph)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{traits::HypergraphClass, Main};
    use test_case::test_case;

    #[test_case(Hypergraph::<u8, u8>::new(), Main; "Main")]
    #[test_case(Hypergraph::<u8, u8, u8, u8, Sub>::new(), Sub; "Sub")]
    fn class<N, E, H, L, Ty: HypergraphClass>(h: Hypergraph<N, E, H, L, Ty>, expected: Ty) {
        assert_eq!(h.class(), &expected)
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

        assert!(h.neighbors(vec![]).next().is_none());
        assert!(h.neighbors(vec![3]).next().is_none());
        assert!(h.neighbors(vec![4]).next().is_none());

        let mut neighbors = h.neighbors(vec![0]);
        assert_eq!(neighbors.next(), Some(&vec![2]));
        assert_eq!(neighbors.next(), Some(&vec![2]));
        assert_eq!(neighbors.next(), None);
        let mut neighbors = h.neighbors(vec![1]);
        assert_eq!(neighbors.next(), None);
        let mut neighbors = h.neighbors(vec![2]);
        assert_eq!(neighbors.next(), Some(&vec![1]));
        assert_eq!(neighbors.next(), None);
        let mut neighbors = h.neighbors(vec![6]);
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
    fn node_value() {
        let mut h = Hypergraph::<_, ()>::new();
        h.add_node("zero", []).unwrap();
        h.add_node("one", []).unwrap();
        assert_eq!(h.node_value([0]).unwrap(), &"zero");
        assert_eq!(h.node_value([1]).unwrap(), &"one");
    }
}
