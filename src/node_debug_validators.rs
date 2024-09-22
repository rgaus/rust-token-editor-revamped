use crate::node::{InMemoryNode, NodeMetadata};
use std::{cell::RefCell, rc::Rc};

/// Currently, there is no way to check to see if nodes that could be copies of each other are
/// equal. I may add an id or something like that. Until then, check to see if their metadata
/// matches, which should be good enough a large percentage of the time.
fn nodes_equal_by_hueristic(a: &Rc<RefCell<InMemoryNode>>, b: &Rc<RefCell<InMemoryNode>>) -> bool {
    a.borrow().metadata == b.borrow().metadata
}

#[derive(Debug)]
pub enum NodeNextValidReason {
    Yes,
    UnsetExpectedFirstChild,
    ExpectedFirstChild(NodeMetadata, NodeMetadata),
    UnsetExpectedNextSibling,
    ExpectedNextSibling(NodeMetadata, NodeMetadata),
    UnsetExpectedRecursiveSibling(NodeMetadata, usize /* levels_upwards_traversed */),
    ExpectedRecursiveSibling(NodeMetadata, NodeMetadata),
    SetExpectedEOF(NodeMetadata),
    ParentWeakRefMissing,
    InIsolatedTree,
}

/// Given a InMemoryNode with potentially questionable weak refs, validate that `wrapped_node.next`
/// is weakref'd to the correct node.
///
/// Note that the only references this function assumes are correct are those in `wrapped_node.children`.
pub fn validate_node_next(
    wrapped_node: &Rc<RefCell<InMemoryNode>>,
    parent_expected_index_within_children: Option<usize>,
) -> NodeNextValidReason {
    let node = wrapped_node.borrow();

    let node_next = if let Some(node_next) = node.next.clone() {
        node_next.upgrade()
    } else {
        None
    };

    if let Some(first_element_of_node_children) = node.children.first() {
        // This node has children, so `node.next` should be the first child
        if let Some(node_next) = node_next {
            if nodes_equal_by_hueristic(&node_next, first_element_of_node_children) {
                NodeNextValidReason::Yes
            } else {
                NodeNextValidReason::ExpectedFirstChild(
                    node_next.borrow().metadata.clone(),
                    first_element_of_node_children.borrow().metadata.clone(),
                )
            }
        } else {
            NodeNextValidReason::UnsetExpectedFirstChild
        }
    } else if let Some(parent) = &node.parent {
        // This node does not have children, so its next value is its next sibling
        // (ie, node.parent.children[(current node index)+1])
        parent
            .upgrade()
            .map_or(NodeNextValidReason::ParentWeakRefMissing, |parent| {
                if let Some(next_element_in_children) = parent.borrow().children.get(
                    if let Some(parent_expected_index_within_children) =
                        parent_expected_index_within_children
                    {
                        parent_expected_index_within_children + 1
                    } else {
                        0
                    },
                ) {
                    // The node.next value was equivalent to node.parent.children[(current node index)+1]
                    return node_next.map_or(
                        NodeNextValidReason::UnsetExpectedNextSibling,
                        |node_next| {
                            if nodes_equal_by_hueristic(&node_next, next_element_in_children) {
                                NodeNextValidReason::Yes
                            } else {
                                NodeNextValidReason::ExpectedNextSibling(
                                    next_element_in_children.borrow().metadata.clone(),
                                    node_next.borrow().metadata.clone(),
                                )
                            }
                        },
                    );
                }

                // It seems `node` is the last child in `node.parent.children`, so there's
                // no "next element" to fetch in parent.children.
                //
                // So, walk upwards through each node's parents and try to find the next
                // sibling "deeply" of the parent node, and THAT is the next node
                if let Some(parent_expected_index_within_children) =
                    parent_expected_index_within_children
                {
                    let mut cursor_index_in_its_parent = parent_expected_index_within_children;
                    let mut cursor_node = Some(parent);
                    let mut levels_upwards_traversed = 0;
                    while let Some(cursor_node_unwrapped) = cursor_node {
                        let cursor_node_borrowed = cursor_node_unwrapped.borrow();
                        if let Some(cursor_node_next_sibling) = cursor_node_borrowed
                            .children
                            .get(cursor_index_in_its_parent + 1)
                        {
                            // The node.next value was equivalent to node.parent.children[(current node index)+1]
                            return node_next.map_or(
                                NodeNextValidReason::UnsetExpectedRecursiveSibling(
                                    cursor_node_next_sibling.borrow().metadata.clone(),
                                    levels_upwards_traversed,
                                ),
                                |node_next| {
                                    if nodes_equal_by_hueristic(&node_next, cursor_node_next_sibling) {
                                        NodeNextValidReason::Yes
                                    } else {
                                        NodeNextValidReason::ExpectedRecursiveSibling(
                                            cursor_node_next_sibling.borrow().metadata.clone(),
                                            node_next.borrow().metadata.clone(),
                                        )
                                    }
                                },
                            );
                        }

                        let cursor_node_parent = cursor_node_borrowed
                            .parent
                            .clone()
                            .map(|parent| parent.upgrade())
                            .flatten();
                        if let Some(cursor_node_parent) = cursor_node_parent.clone() {
                            if let Some(index) =
                                cursor_node_parent.borrow().children.iter().position(|n| {
                                    nodes_equal_by_hueristic(n, &cursor_node_unwrapped)
                                })
                            {
                                cursor_index_in_its_parent = index;
                            }
                        }

                        cursor_node = cursor_node_parent;
                        levels_upwards_traversed += 1;
                    }
                }

                // If we've walked all the way up to the root node and not found a
                // sibling after this, this must be the final leaf node. And in this case,
                // node.next should be None.
                if let Some(node_next) = node_next {
                    NodeNextValidReason::SetExpectedEOF(node_next.borrow().metadata.clone())
                } else {
                    NodeNextValidReason::Yes
                }
            })
    } else {
        // No parent AND no children? This node seems to be in a tree all on
        // its own.
        NodeNextValidReason::InIsolatedTree
    }
}
