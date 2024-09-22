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


#[derive(Debug)]
pub enum NodePreviousValidReason {
    Yes,
    UnsetExpectedParent,
    ExpectedParent(NodeMetadata, NodeMetadata),
    UnsetExpectedPreviousSiblingDeepLastChild,
    ExpectedPreviousSiblingDeepLastChild(NodeMetadata, NodeMetadata, usize /* levels_downwards_traversed */),
    UnsetExpectedPreviousSibling,
    ExpectedPreviousSibling(NodeMetadata, NodeMetadata),
    ExpectedParentlessNodeToHavePreviousNone(NodeMetadata),
    ParentWeakRefMissing,
    InIsolatedTree,
}

pub fn validate_node_previous(wrapped_node: &Rc<RefCell<InMemoryNode>>) -> NodePreviousValidReason {
    let node = wrapped_node.borrow();

    let node_previous = if let Some(node_previous) = node.previous.clone() {
        node_previous.upgrade()
    } else {
        None
    };

    if let Some(parent) = &node.parent {
        if let Some(parent_upgraded) = parent.upgrade() {
            let parent_children = &parent_upgraded.borrow().children;
            let node_index_in_parent = parent_children.iter().position(|n| nodes_equal_by_hueristic(n, wrapped_node));
            // println!("FOO: {:?} {:?}", node.metadata, node_index_in_parent);

            // 1. Is this node the first sibling of its parent? Then the previous is `parent`.
            if let Some(0) = node_index_in_parent {
                if let Some(node_previous) = node_previous {
                    if nodes_equal_by_hueristic(&node_previous, &parent_upgraded) {
                        NodePreviousValidReason::Yes
                    } else {
                        NodePreviousValidReason::ExpectedParent(
                            parent_upgraded.borrow().metadata.clone(),
                            node_previous.borrow().metadata.clone(),
                        )
                    }
                } else {
                    NodePreviousValidReason::UnsetExpectedParent
                }

            // 2. Does the node have at least one sibling before it?
            } else if let Some(node_index_in_parent) = node_index_in_parent {
                if node_index_in_parent == 0 {
                    panic!("Error: node_index_in_parent == 0, but this should be impossible because this is what #1 checks for!");
                }
                let previous_sibling_index_in_parent = node_index_in_parent - 1;
                let previous_sibling_in_parent = &parent_children[previous_sibling_index_in_parent];

                // a. Does this previous sibling have children? If so, get the deep last child of that
                // previous sibling, and that should be `previous`
                //
                // NOTE: the below is a reimplementation of `InMemoryNode::deep_last_child` which
                // does not rely on anything except for `children` being properly set in each node.
                let (
                    previous_sibling_deep_last_child,
                    levels_downwards_traversed,
                ) = if let Some(initial_last_child) = previous_sibling_in_parent.borrow().children.last().clone() {
                    let mut cursor_node = initial_last_child.clone();
                    let mut levels_downwards_traversed = 0;
                    loop {
                        let cursor_node_cloned = cursor_node.borrow().clone();
                        let Some(last_child) = cursor_node_cloned.children.last().clone() else {
                            break;
                        };
                        cursor_node = last_child.clone();
                        levels_downwards_traversed += 1;
                    }

                    (Some(cursor_node), levels_downwards_traversed)
                } else {
                    (None, 0)
                };

                if let Some(previous_sibling_deep_last_child) = previous_sibling_deep_last_child {
                    if let Some(node_previous) = node_previous {
                        if nodes_equal_by_hueristic(&node_previous, &previous_sibling_deep_last_child.clone()) {
                            NodePreviousValidReason::Yes
                        } else {
                            NodePreviousValidReason::ExpectedPreviousSiblingDeepLastChild(
                                previous_sibling_deep_last_child.borrow().metadata.clone(),
                                node_previous.borrow().metadata.clone(),
                                levels_downwards_traversed,
                            )
                        }
                    } else {
                        NodePreviousValidReason::UnsetExpectedPreviousSiblingDeepLastChild
                    }
                } else {
                    // b. If the previous sibling does not have children, then `previous` is that sibling
                    if let Some(node_previous) = node_previous {
                        if nodes_equal_by_hueristic(&node_previous, previous_sibling_in_parent) {
                            NodePreviousValidReason::Yes
                        } else {
                            NodePreviousValidReason::ExpectedPreviousSibling(
                                previous_sibling_in_parent.borrow().metadata.clone(),
                                node_previous.borrow().metadata.clone(),
                            )
                        }
                    } else {
                        NodePreviousValidReason::UnsetExpectedPreviousSibling
                    }
                }
            } else {
                // No parent AND no children? This node seems to be in a tree all on
                // its own.
                NodePreviousValidReason::InIsolatedTree
            }
        } else {
            NodePreviousValidReason::ParentWeakRefMissing
        }
    } else {
        // 3. Does this node not have a parent? Then `previous` is None
        if let Some(node_previous) = node_previous {
            NodePreviousValidReason::ExpectedParentlessNodeToHavePreviousNone(
                node_previous.borrow().metadata.clone()
            )
        } else {
            NodePreviousValidReason::Yes
        }
    }
}
