use crate::{
    direction::Direction,
    elements::{Element, ElementExt, ElementValue},
    errors, Hypergraph, Main,
};

/// # Add
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
    ///
    /// Also, if element is an edge or a link, `location` must be coherent with the pair `(source, target)`.
    /// This prevents to have links in locations unrelated to `source` or `target`.
    /// To be coherent means satisfying one of the following rules:
    ///  - `source` and `target` are in the same hypergraph with `id` `location`.
    ///  - `source` and `target` are in hypergraphs which are nested,
    /// and `location` refers to to either one of these hypergraphs or another hypergraph that contains both of them.
    ///  -  `source` and `target` are in nonintersecting hypergraphs,
    /// and `location` refers to a hypergraph that contains both of them.
    //
    // # Note
    //
    // This method performs all checks and call the unchecked variant.
    pub fn add_element(
        &mut self,
        element: ElementExt<N, E, H, L, Vec<usize>>,
        location: impl AsRef<[usize]>,
    ) -> Result<Vec<usize>, errors::AddError> {
        let location = location.as_ref();
        if !self.contains_hypergraph(location) {
            Err(errors::NoHypergraph(location.to_vec()))?
        }
        if element.is_node() | element.is_hypergraph() {
            return Ok(self.add_element_unchecked(element, location));
        }
        // Never fails since element is now either edge or link
        let global_source_id = element.source().unwrap();
        if global_source_id.is_empty() {
            Err(errors::EmptySource)?
        }

        let source_element = match self.element_value(&global_source_id) {
            Err(_) => {
                return Err(errors::AddError::NoSource(errors::NoElementLinkable(
                    global_source_id.to_vec(),
                )));
            }
            Ok(source_element) => match source_element {
                ElementValue::Link { .. } => {
                    return Err(errors::AddError::LinkSource(errors::LinkSource(
                        element.into_source().unwrap(),
                    )))
                }
                ElementValue::Edge { .. } => {
                    if let ElementExt::Edge { source, target, .. } = element {
                        return Err(errors::AddError::Unlinkable(errors::Unlinkable(
                            source, target,
                        )));
                        // Edge -> Edge can not be
                    }
                    source_element
                }
                ElementValue::Node { .. } | ElementValue::Hypergraph { .. } => source_element,
            },
        };

        let global_target_id = element.target().unwrap();
        if global_target_id.is_empty() {
            Err(errors::EmptyTarget)?;
        }

        let target_element = match self.element_value(&global_target_id) {
            Err(_) => {
                return Err(errors::AddError::NoSource(errors::NoElementLinkable(
                    global_target_id.to_vec(),
                )));
            }
            Ok(target_element) => match target_element {
                ElementValue::Link { .. } => {
                    return Err(errors::AddError::LinkTarget(errors::LinkTarget(
                        element.into_target().unwrap(),
                    )))
                }
                ElementValue::Edge { .. } => {
                    if let ElementExt::Edge { source, target, .. } = element {
                        return Err(errors::AddError::Unlinkable(errors::Unlinkable(
                            source, target,
                        )));
                        // Edge -> Edge can not be
                    }
                    target_element
                }
                ElementValue::Node { .. } | ElementValue::Hypergraph { .. } => target_element,
            },
        };

        // target_element is either node or hypergrpha, or edge only if element is a link
        // Check that we are not linking edge with edge
        if source_element.is_edge() && target_element.is_edge() {
            if let ElementExt::Link { source, target, .. } = element {
                return Err(errors::AddError::Unlinkable(errors::Unlinkable(
                    source, target,
                ))); // Edge -> Edge can not be
            }
        }
        // Check that we are linking through an edge
        if element.is_link()
            && (source_element.is_node() || source_element.is_hypergraph())
            && (target_element.is_node() || target_element.is_hypergraph())
        {
            if let ElementExt::Link { source, target, .. } = element {
                return Err(errors::AddError::Unlinkable(errors::Unlinkable(
                    source, target,
                )));
                // (node or h) -> (node or h) can not be
            }
        }
        // Check coherence of location with respect to source and target
        let source_hypergraph_id = &global_source_id[0..global_source_id.len() - 1];
        let target_hypergraph_id = &global_target_id[0..global_target_id.len() - 1];
        fn contains_or_equals(one: &[usize], other: &[usize]) -> bool {
            if one.len() <= other.len() {
                one == &other[0..one.len()]
            } else {
                false
            }
        }
        fn are_strictly_nested(one: &[usize], other: &[usize]) -> bool {
            if one.len() < other.len() {
                one == &other[0..one.len()]
            } else {
                &one[0..other.len()] == other
            }
        }
        // let location = location.to_vec();
        let coherent_rule_same_hypergraph =
            (source_hypergraph_id == target_hypergraph_id) && (source_hypergraph_id == location);
        let coherent_rule_nested = are_strictly_nested(source_hypergraph_id, target_hypergraph_id)
            && ((location == source_hypergraph_id)
                || (location == target_hypergraph_id)
                || (contains_or_equals(location, source_hypergraph_id)
                    && contains_or_equals(location, target_hypergraph_id)));
        let coherent_rule_nonintersecting = contains_or_equals(location, source_hypergraph_id)
            && contains_or_equals(location, target_hypergraph_id);

        if !(coherent_rule_same_hypergraph || coherent_rule_nested || coherent_rule_nonintersecting)
        {
            Err(errors::IncoherentLink(
                location.to_vec(),
                global_source_id.clone(),
                global_target_id.clone(),
            ))?
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
    /// If any `id` provided (`location` or within `element_ext`) does not correspond to an element of the hypergraph,
    /// or if `element_ext` is a connection (edge or link) and `source` or `target` can not be connected through `elmenet`.
    pub fn add_element_unchecked(
        &mut self,
        element_ext: ElementExt<N, E, H, L, Vec<usize>>,
        location: impl AsRef<[usize]>,
    ) -> Vec<usize> {
        let location = location.as_ref();
        match element_ext {
            ElementExt::Node { .. } | ElementExt::Hypergraph { .. } => {
                let mut hypergraph = self.hypergraph_mut(location).unwrap();
                let local_id = hypergraph.add_local_element(element_ext.into());
                let mut new_element_id = location.to_vec();
                new_element_id.push(local_id);
                return new_element_id;
            }
            ElementExt::Edge { .. } | ElementExt::Link { .. } => (),
        }

        match element_ext {
            ElementExt::Edge {
                value,
                source,
                target,
            } => {
                let mut hypergraph = self.hypergraph_mut(location).unwrap();
                // Add edge
                let mut new_edge_id = location.to_vec();
                let edge_local_id = hypergraph.add_local_element(Element::Edge { value });
                new_edge_id.push(edge_local_id);
                // Add links
                let mut link_source_id = location.to_vec();
                let link_source_local_id = hypergraph.add_local_element(Element::Link {
                    value: None,
                    source: source.clone(),
                    target: new_edge_id.clone(),
                });
                link_source_id.push(link_source_local_id);
                let mut link_target_id = location.to_vec();
                let link_target_local_id = hypergraph.add_local_element(Element::Link {
                    value: None,
                    source: new_edge_id.clone(),
                    target: target.clone(),
                });
                link_target_id.push(link_target_local_id);
                // Add new neighbors to edge, source and target and their links
                let vec = self.links_of_mut(&new_edge_id).unwrap();
                vec.push((link_source_id.clone(), Direction::Incoming));
                vec.push((link_target_id.clone(), Direction::Outgoing));
                self.links_of_mut(&source)
                    .unwrap()
                    .push((link_source_id.clone(), Direction::Outgoing));
                self.links_of_mut(&target)
                    .unwrap()
                    .push((link_target_id.clone(), Direction::Incoming));
                // Return new edge id
                return new_edge_id;
            }
            ElementExt::Link {
                value,
                source,
                target,
            } => {
                let mut hypergraph = self.hypergraph_mut(location).unwrap();

                // Add link
                let mut new_link_id = location.to_vec();
                let local_id = hypergraph.add_local_element(Element::Link {
                    source: source.clone(),
                    target: target.clone(),
                    value,
                });
                new_link_id.push(local_id);
                // Add new neighbors to source and target
                self.hypergraph_of_mut(&source)
                    .unwrap() // Never fails since source is a valid element
                    .add_local_neighbor_unchecked(
                        *source.last().unwrap(), // Never fails since source is a valid id by contract
                        (new_link_id.clone(), Direction::Outgoing),
                    );
                self.hypergraph_of_mut(&target)
                    .unwrap() // Never fails since source is a valid element
                    .add_local_neighbor_unchecked(
                        *target.last().unwrap(), // Never fails since target is a valid id by contract
                        (new_link_id.clone(), Direction::Incoming),
                    );
                // Return link id
                return new_link_id;
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
    ) -> Result<Vec<usize>, errors::AddError> {
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
    ) -> Result<Vec<usize>, errors::AddError> {
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
    ) -> Result<Vec<usize>, errors::AddError> {
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
    ) -> Result<Vec<usize>, errors::AddError> {
        let element = ElementExt::Node { value };
        self.add_element(element, location)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test]
    fn add_edge() {
        let mut h = Hypergraph::<&str, &str>::new();
        let node_0_id = h.add_node("zero", []).unwrap();
        let node_1_id = h.add_node("one", []).unwrap();
        let edge_id = h.add_edge([0], [1], "two", []).unwrap();
        assert_eq!(vec![2], edge_id);
        assert_eq!(
            Err(errors::FindError::NoLink),
            h.find_link_id([0], [1], &None, [])
        );
        assert_eq!(Ok(vec![3]), h.find_link_id([0], [2], &None, []));
        assert_eq!(Ok(vec![4]), h.find_link_id([2], [1], &None, []));
        assert!(h.contains_node(node_0_id));
        assert!(h.contains_node(node_1_id));
        assert_eq!(h.edge_value(&edge_id).unwrap(), &"two");
    }

    #[test_case(
        {
            let mut h = Hypergraph::<_, ()>::new();
            h.add_node("zero", []).unwrap();
            h.add_link([0], [], (), [])
        }, //
        errors::AddError::EmptyTarget(errors::EmptyTarget); //
        "empty target"
    )]
    #[test_case(
        {
            let mut h = Hypergraph::<(), (), _>::new();
            h.add_hypergraph("zero", []).unwrap();
            h.add_hypergraph("one", []).unwrap();
            h.add_hypergraph("zero", [0]).unwrap();
            h.add_edge([0, 0], [0], (), [1])
        }, //
        errors::AddError::IncoherentLink(errors::IncoherentLink(vec![1], vec![0, 0], vec![0])); //
        "incoherent link"
    )]
    #[test_case(
     {
        let mut h = Hypergraph::<_, _, ()>::new();
        h.add_node("zero", []).unwrap();
        h.add_node("one", []).unwrap();
        h.add_edge([0], [1], "two", []).unwrap();
        h.add_link([3], [0], (), [])
     }, //
     errors::AddError::LinkSource(errors::LinkSource(vec![3])); //
        "link source"
    )]
    #[test_case(
     {
        let mut h = Hypergraph::<_, _, ()>::new();
        h.add_node("zero", []).unwrap();
        h.add_node("one", []).unwrap();
        h.add_edge([0], [1], "two", []).unwrap();
        h.add_link([0], [3], (), [])
     }, //
     errors::AddError::LinkTarget(errors::LinkTarget(vec![3])); //
        "link target"
    )]
    #[test_case(
        {
            let mut h = Hypergraph::<_, ()>::new();
            h.add_node("zero", [1])
        },//
        errors::AddError::NoLocation(errors::NoHypergraph(vec![1])); //
        "no hypergraph"
    )]
    #[test_case(
        {
            let mut h = Hypergraph::<(), (), (), _>::new();
            h.add_link([1], [], "zero", [])
        }, //
        errors::AddError::NoSource(errors::NoElementLinkable(vec![1])); //
        "no source"
    )]
    #[test_case(
        {
            let mut h = Hypergraph::<(), (), _>::new();
            h.add_hypergraph("zero", []).unwrap();
            h.add_hypergraph("one", [0]).unwrap();
            h.add_link([0, 0], [0], (), [])
        }, //
        errors::AddError::Unlinkable(errors::Unlinkable(vec![0, 0], vec![0])); //
        "unlinkable"
    )]
    fn add_error(result: Result<Vec<usize>, errors::AddError>, expected: errors::AddError) {
        println!("add output: {:?}", result);
        assert_eq!(result.err().unwrap(), expected);
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
        h.add_node("zero", []).unwrap();
        h.add_node("one", []).unwrap();
        h.add_edge([0], [1], "two", []).unwrap();
        let link_id = h.add_link([0], [2], "three", []).unwrap();
        assert_eq!(h.link_value(link_id).unwrap(), &Some("three"));
    }
    #[test]
    fn add_node() {
        let mut h = Hypergraph::<_, u8>::new();
        let id = h.add_node("zero", []).unwrap();
        assert_eq!(h.node_value(id).unwrap(), &"zero");
    }
}
