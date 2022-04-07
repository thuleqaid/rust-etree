Rust-Etree
=========================
[![Rust-Etree on Travis CI][travis-image]][travis]

[travis-image]: https://travis-ci.org/thuleqaid/rust-etree.png
[travis]: https://travis-ci.org/thuleqaid/rust-etree

XML DOM library with XPath support.

## Usage

Put this in your `Cargo.toml`:

```toml
[dependencies]
etree = "0.1"
```

```rust
use etree::*;
use std::path::Path;

fn create_xml<P:AsRef<Path>>(path:P) {
    let mut tree:ETree = ETree::from(ETreeNode::new("ROOT"));
    tree.set_encoding("UTF-8");
    tree.set_standalone("no");
    let root_pos = tree.root();

    // append first child
    let mut child1:ETreeNode = ETreeNode::new("CHILD-A");
    child1.set_attr("DEST", "CHN");
    let child1_pos = tree.append_child_node(root_pos, child1).unwrap();
    // append another child after first child
    let mut child2:ETreeNode = ETreeNode::new("CHILD-B");
    child2.set_text("Shanghail");
    tree.append_next_node(child1_pos, child2);
    // append another child before first child
    let mut child3:ETreeNode = ETreeNode::new("CHILD-C");
    child3.set_attr("DEST", "CHN");
    child3.set_text("Shanghail");
    tree.append_previous_node(child1_pos, child3);
    // append a child in the first child
    let mut child4:ETreeNode = ETreeNode::new("SUBCHILD-A");
    child4.set_text("EAST");
    let pos = tree.find("//CHILD-A").unwrap(); // after inserting child3, child1_pos becomes invaild
    tree.append_child_node(pos, child4);
    tree.pretty("\n  ");
    tree.write_file(path).ok();
}

fn modify_xml<P:AsRef<Path>>(path_in:P, path_out:P) {
    let mut tree = ETree::parse_file(path_in);
    let subtree_pos = tree.find("//CHILD-A").unwrap();
    let mut subtree = tree.subtree(subtree_pos);
    let subtree_child_pos = subtree.find("/SUBCHILD-A").unwrap();
    if let Some(node) = subtree.node_mut(subtree_child_pos) {
        node.set_text("WEST");
    }
    // tree.append_next_tree(subtree_pos, subtree.clone());
    let parent_pos = tree.parent(subtree_pos).unwrap();
    tree.append_child_tree(parent_pos, subtree);
    tree.write_file(path_out).ok();
}
```

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
