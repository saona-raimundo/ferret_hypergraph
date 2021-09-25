use core::convert::TryFrom;
use serde::{Deserialize, Serialize};

use crate::errors::LinkPresent;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElementValue<N, E, H, L> {
    /// A graph edge.
    Edge { value: E },
    /// A hypergraph.
    Hypergraph { value: Option<H> },
    /// A graph link.
    Link { value: Option<L> },
    /// A graph node.
    Node { value: N },
}

impl<N, E, H, L> ElementValue<N, E, H, L> {
    pub fn is_edge(&self) -> bool {
        matches!(self, ElementValue::Edge { .. })
    }

    pub fn is_hypergraph(&self) -> bool {
        matches!(self, ElementValue::Hypergraph { .. })
    }

    pub fn is_link(&self) -> bool {
        matches!(self, ElementValue::Link { .. })
    }

    pub fn is_node(&self) -> bool {
        matches!(self, ElementValue::Node { .. })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElementLinkable<N, E, H> {
    /// A graph edge.
    Edge { value: E },
    /// A hypergraph.
    Hypergraph { value: Option<H> },
    /// A graph node.
    Node { value: N },
}

impl<N, E, H, L, Id> TryFrom<Element<N, E, H, L, Id>> for ElementLinkable<N, E, H> {
    type Error = LinkPresent;
    fn try_from(element: Element<N, E, H, L, Id>) -> Result<ElementLinkable<N, E, H>, Self::Error> {
        match element {
            Element::Edge { value } => Ok(ElementLinkable::Edge { value }),
            Element::Hypergraph { value } => Ok(ElementLinkable::Hypergraph { value }),
            Element::Node { value } => Ok(ElementLinkable::Node { value }),
            Element::Link { .. } => Err(LinkPresent),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Element<N, E, H, L, Id> {
    /// A graph edge.
    Edge { value: E },
    /// A hypergraph.
    Hypergraph { value: Option<H> },
    /// A graph link.
    Link {
        source: Id,
        target: Id,
        value: Option<L>,
    },
    /// A graph node.
    Node { value: N },
}

impl<N, E, H, L, Id> Element<N, E, H, L, Id> {
    pub fn is_edge(&self) -> bool {
        matches!(self, Element::Edge { .. })
    }

    pub fn is_hypergraph(&self) -> bool {
        matches!(self, Element::Hypergraph { .. })
    }

    pub fn is_link(&self) -> bool {
        matches!(self, Element::Link { .. })
    }

    pub fn is_node(&self) -> bool {
        matches!(self, Element::Node { .. })
    }
    pub fn source(&self) -> Option<&Id> {
        match self {
            Element::Link { source, .. } => Some(&source),
            Element::Edge { .. } | Element::Hypergraph { .. } | Element::Node { .. } => None,
        }
    }

    pub fn target(&self) -> Option<&Id> {
        match self {
            Element::Link { target, .. } => Some(&target),
            Element::Edge { .. } | Element::Hypergraph { .. } | Element::Node { .. } => None,
        }
    }
}

impl<N, E, H, L, Id> From<ElementExt<N, E, H, L, Id>> for Element<N, E, H, L, Id> {
    fn from(element_ext: ElementExt<N, E, H, L, Id>) -> Self {
        match element_ext {
            ElementExt::Edge { value, .. } => Element::Edge { value },
            ElementExt::Link {
                source,
                target,
                value,
            } => Element::Link {
                source,
                target,
                value,
            },
            ElementExt::Hypergraph { value } => Element::Hypergraph { value },
            ElementExt::Node { value } => Element::Node { value },
        }
    }
}

impl<N, E, H, L, Id> From<ElementLinkable<N, E, H>> for Element<N, E, H, L, Id> {
    fn from(element_linkable: ElementLinkable<N, E, H>) -> Self {
        match element_linkable {
            ElementLinkable::Edge { value } => Element::Edge { value },
            ElementLinkable::Hypergraph { value } => Element::Hypergraph { value },
            ElementLinkable::Node { value } => Element::Node { value },
        }
    }
}

// impl TryInto<ElementExt> for Element ...

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElementType {
    Edge,
    Hypergraph,
    Link,
    Node,
}

impl ElementType {
    pub fn wrapping_next(self) -> Self {
        match self {
            ElementType::Edge => ElementType::Hypergraph,
            ElementType::Hypergraph => ElementType::Link,
            ElementType::Link => ElementType::Node,
            ElementType::Node => ElementType::Edge,
        }
    }
}

impl<N, E, H, L> From<ElementValue<N, E, H, L>> for ElementType {
    fn from(element_value: ElementValue<N, E, H, L>) -> Self {
        match element_value {
            ElementValue::Edge { .. } => ElementType::Edge,
            ElementValue::Link { .. } => ElementType::Link,
            ElementValue::Hypergraph { .. } => ElementType::Hypergraph,
            ElementValue::Node { .. } => ElementType::Node,
        }
    }
}

impl<N, E, H, L, Id> From<Element<N, E, H, L, Id>> for ElementType {
    fn from(element: Element<N, E, H, L, Id>) -> Self {
        match element {
            Element::Edge { .. } => ElementType::Edge,
            Element::Link { .. } => ElementType::Link,
            Element::Hypergraph { .. } => ElementType::Hypergraph,
            Element::Node { .. } => ElementType::Node,
        }
    }
}

/// Element extended with information to be added to a hypergraph.
///
/// `Edge` variant now has `source` and `target`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElementExt<N, E, H, L, Id> {
    /// A graph edge.
    Edge { source: Id, target: Id, value: E },
    /// A hypergraph.
    Hypergraph { value: Option<H> },
    /// A graph link.
    Link {
        source: Id,
        target: Id,
        value: Option<L>,
    },
    /// A graph node.
    Node { value: N },
}

impl<N, E, H, L, Id> ElementExt<N, E, H, L, Id> {
    pub fn is_edge(&self) -> bool {
        matches!(self, ElementExt::Edge { .. })
    }

    pub fn is_hypergraph(&self) -> bool {
        matches!(self, ElementExt::Hypergraph { .. })
    }

    pub fn is_link(&self) -> bool {
        matches!(self, ElementExt::Link { .. })
    }

    pub fn is_node(&self) -> bool {
        matches!(self, ElementExt::Node { .. })
    }

    pub fn into_source(self) -> Option<Id> {
        match self {
            ElementExt::Edge { source, .. } => Some(source),
            ElementExt::Link { source, .. } => Some(source),
            ElementExt::Hypergraph { .. } | ElementExt::Node { .. } => None,
        }
    }
    pub fn into_target(self) -> Option<Id> {
        match self {
            ElementExt::Edge { target, .. } => Some(target),
            ElementExt::Link { target, .. } => Some(target),
            ElementExt::Hypergraph { .. } | ElementExt::Node { .. } => None,
        }
    }

    pub fn source(&self) -> Option<&Id> {
        match self {
            ElementExt::Edge { source, .. } => Some(&source),
            ElementExt::Link { source, .. } => Some(&source),
            ElementExt::Hypergraph { .. } | ElementExt::Node { .. } => None,
        }
    }
    pub fn target(&self) -> Option<&Id> {
        match self {
            ElementExt::Edge { target, .. } => Some(&target),
            ElementExt::Link { target, .. } => Some(&target),
            ElementExt::Hypergraph { .. } | ElementExt::Node { .. } => None,
        }
    }
}
