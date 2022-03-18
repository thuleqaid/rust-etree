use etree::{ETree, ETreeNode, XPath};
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
    let pos = tree.find("//CHILD-A"); // after inserting child3, child1_pos becomes invaild
    tree.append_child_node(pos[0], child4);
    tree.pretty("\n  ");
    tree.write_file(path).ok();
}

fn modify_xml<P:AsRef<Path>>(path_in:P, path_out:P) {
    let mut tree = ETree::parse_file(path_in);
    let subtree_pos = tree.find("//CHILD-A");
    assert!(subtree_pos.len() == 1);
    let mut subtree = tree.subtree(subtree_pos[0]);
    let subtree_child_pos = subtree.find("/SUBCHILD-A");
    assert!(subtree_child_pos.len() == 1);
    if let Some(node) = subtree.node_mut(subtree_child_pos[0]) {
        node.set_text("WEST");
    }
    // tree.append_next_tree(subtree_pos[0], subtree.clone());
    let parent_pos = tree.parent(subtree_pos[0]).unwrap();
    tree.append_child_tree(parent_pos, subtree);
    tree.write_file(path_out).ok();
}

fn clear_indent<P:AsRef<Path>>(path_in:P, path_out:P) {
    let mut tree = ETree::parse_file(path_in);
    tree.noindent();
    tree.write_file(path_out).ok();
}

fn main() {
    let file1 = "test_1.xml";
    let file2 = "test_2.xml";
    let file3 = "test_3.xml";
    create_xml(file1);
    modify_xml(file1, file2);
    clear_indent(file2, file3);
}
