//! Hypergraphs data structure library.
//!
//! This library provides the [`Hypergraph`] struct.
//!
#![cfg_attr(
	all(),
	doc = ::embed_doc_image::embed_image!("hypergraph_concept", "images/hypergraph_concept.png")
)]
//!
//! ![Image of a hypergraph and its elements][hypergraph_concept]
//!
//! # Features
//!
//! - Hyper graph: Edges can connect more than two nodes.
//! - Recursive: Databases can contain other databases.
//! - Visualization: Simple graphical representation.
//! - Serialization: Serialize as you like.
//! - Thread safe: Implements `Send` and `Sync`.
//! - Node and edges are important: Both are first class citizen.
//! - File support: Any element may have attached files or urls.
//! - No hashing needed: Types are totally arbitrary.
//!
//! # Inspirations
//!
//! There are many graph data structures out there, but these are the ones that
//! had the mos influence in the construction: [`petgraph`], [`Neo4j`], [`CMapTool`], [`hypergraph`](https://crates.io/crates/hypergraph)
//!
//! ## Similar datastructures
//!
//! ### Petgraph
//!
//! A `petgraph::Graph<NodeEnum, L, Directed, usize>`, where `NodeEnum` is an enum with three variants: `Node<N>`, `Edge<E>` and `Hypergraph<H>`
//! is very similar to `Hypergraph<N, E, H, L>`: Edges can connect to multiple nodes and hypergraphs.
//! But there are some differences too:
//! - [`Hypergraph`] allows multiedges
//! - [`Hypergraph`] allows links from nodes of one `Hypergraph` to another.
//! - [`Hypergraph`] does not allow links between nodes and hypergraphs, there must be an edge in between.
//!
//! [`Hypergraph`]: struct.Hypergraph.html
//! [`petgraph`]: https://crates.io/crates/petgraph
//! [`Neo4j`]: https://neo4j.com/
//! [`CMapTool`]: https://cmap.ihmc.us/

mod direction;
/// Elements of a hypergraph, in all variants.
pub mod elements;
/// All basic errors in this crate.
pub mod errors;
mod hypergraph;
/// Iterators for a hypergraph.
pub mod iterators;
/// All traits in this crate.
pub mod traits;
/// Walkers for a hypergraph.
pub mod walkers;

pub use direction::Direction;
pub use hypergraph::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
