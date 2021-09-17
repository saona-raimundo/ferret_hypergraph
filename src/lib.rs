//! Hypergraphs data structure library.
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
//!
//! # Inspirations
//!
//! There are many graph data structures out there, but these are the ones that
//! had the mos influence in the construction: [`petgraph`], [`Neo4j`], [`CMapTool`], [`hypergraph`]
//!
//! [`petgraph`]: https://crates.io/crates/petgraph
//! [`Neo4j`]: https://neo4j.com/
//! [`CMapTool`]: https://cmap.ihmc.us/
//! [`hypergraph`]: https://crates.io/crates/hypergraph

mod hyper_graph;

pub use hyper_graph::HyperGraph;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
