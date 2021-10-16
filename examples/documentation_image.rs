use ferret_hypergraph::{visualize::DotFormatter, Hypergraph};

fn main() -> anyhow::Result<()> {
    let mut h = Hypergraph::<&str, &str, &str, &str>::new();
    h.set_value("(main)hypergraph");
    h.add_node("node", [])?;
    h.add_node("node", [])?;
    h.add_edge([0], [1], "edge", [])?;
    h.set_link_value([3], "link")?;
    h.set_link_value([4], "link")?;
    h.add_node("node", [])?;
    h.add_link([2], [5], "link", [])?;

    h.add_hypergraph("(sub)hypergraph", [])?;
    h.add_node("node", [7])?;
    h.add_node("node", [7])?;
    h.add_edge([7, 0], [7, 1], "edge", [7])?;
    h.set_link_value([7, 3], "link")?;
    h.set_link_value([7, 4], "link")?;

    h.add_link([2], [7, 0], "link", [])?;

    // Visualize
    println!("{}", h.as_dot(None));
    h.draw_and_show(DotFormatter::display(), "hypergraph_concept")?;

    Ok(())
}
