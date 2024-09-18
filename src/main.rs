use std::{
    rc::{Rc, Weak},
    cell::RefCell, usize,
};
use colored::Colorize;




#[derive(Debug)]
enum NodeNextValidReason {
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
fn validate_node_next(
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
            if node_next.borrow().metadata == first_element_of_node_children.borrow().metadata {
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
        parent.upgrade().map_or(NodeNextValidReason::ParentWeakRefMissing, |parent| {
            if let Some(next_element_in_children) = parent.borrow().children.get(
                if let Some(parent_expected_index_within_children) = parent_expected_index_within_children {
                    parent_expected_index_within_children+1
                } else {
                    0
                }
            ) {
                // The node.next value was equivalent to node.parent.children[(current node index)+1]
                return node_next.map_or(
                    NodeNextValidReason::UnsetExpectedNextSibling,
                    |node_next| if next_element_in_children.borrow().metadata == node_next.borrow().metadata {
                        NodeNextValidReason::Yes
                    } else {
                        NodeNextValidReason::ExpectedNextSibling(
                            next_element_in_children.borrow().metadata.clone(),
                            node_next.borrow().metadata.clone(),
                        )
                    }
                );
            }

            // It seems `node` is the last child in `node.parent.children`, so there's
            // no "next element" to fetch in parent.children.
            //
            // So, walk upwards through each node's parents and try to find the next
            // sibling "deeply" of the parent node, and THAT is the next node
            if let Some(parent_expected_index_within_children) = parent_expected_index_within_children {
                let mut cursor_index_in_its_parent = parent_expected_index_within_children;
                let mut cursor_node = Some(parent);
                let mut levels_upwards_traversed = 0;
                while let Some(cursor_node_unwrapped) = cursor_node {
                    let cursor_node_borrowed = cursor_node_unwrapped.borrow();
                    if let Some(cursor_node_next_sibling) = cursor_node_borrowed.children.get(
                        cursor_index_in_its_parent+1
                    ) {
                        // The node.next value was equivalent to node.parent.children[(current node index)+1]
                        return node_next.map_or(
                            NodeNextValidReason::UnsetExpectedRecursiveSibling(
                                cursor_node_next_sibling.borrow().metadata.clone(),
                                levels_upwards_traversed,
                            ),
                            |node_next| if cursor_node_next_sibling.borrow().metadata == node_next.borrow().metadata {
                                NodeNextValidReason::Yes
                            } else {
                                NodeNextValidReason::ExpectedRecursiveSibling(
                                    cursor_node_next_sibling.borrow().metadata.clone(),
                                    node_next.borrow().metadata.clone(),
                                )
                            }
                        );
                    }

                    let cursor_node_parent = cursor_node_borrowed.parent.clone().map(|parent| parent.upgrade()).flatten();
                    if let Some(cursor_node_parent) = cursor_node_parent.clone() {
                        if let Some(index) = cursor_node_parent.borrow().children.iter().position(
                            |n| n.borrow().metadata == cursor_node_unwrapped.borrow().metadata
                        ) {
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





#[derive(Debug, Clone, PartialEq)]
enum NodeMetadata {
    Empty,
    Literal(String),
}

#[derive(Debug, Clone)]
struct InMemoryNode {
    metadata: NodeMetadata,

    // Tree data structure refs:
    parent: Option<Weak<RefCell<InMemoryNode>>>,
    children: Vec<Rc<RefCell<InMemoryNode>>>,
    first_child: Option<Weak<RefCell<InMemoryNode>>>,
    last_child: Option<Weak<RefCell<InMemoryNode>>>,

    // Linked list data structure refs:
    next: Option<Weak<RefCell<InMemoryNode>>>,
    previous: Option<Weak<RefCell<InMemoryNode>>>,
}

impl InMemoryNode {
    fn new_empty() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            metadata: NodeMetadata::Empty,
            parent: None,
            children: vec![],
            first_child: None,
            last_child: None,
            next: None,
            previous: None,
        }))
    }
    fn new_from_literal(literal: &str) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            metadata: NodeMetadata::Literal(literal.into()),
            parent: None,
            children: vec![],
            first_child: None,
            last_child: None,
            next: None,
            previous: None,
        }))
    }

    fn dump(node: &Rc<RefCell<Self>>) {
        Self::dump_child(node, "", None);
    }
    fn dump_child(
        wrapped_node: &Rc<RefCell<Self>>,
        spacer: &str,
        parent_expected_index_within_children: Option<usize>,
    ) {
        let node = wrapped_node.borrow();

        // Compute validation results on each row
        let validation_flags = {
            // Check to see if in the parent of the given child, there exists a child at the given
            // index with matching metadata
            let node_was_found_at_expected_index_in_parent = if let (
                Some(parent_expected_index_within_children),
                Some(parent),
            ) = (parent_expected_index_within_children, node.parent.clone()) {
                parent.upgrade().map(|parent| {
                    let borrowed_parent = parent.borrow();
                    let child = borrowed_parent.children.get(parent_expected_index_within_children);
                    if let Some(child) = child {
                        // A child can be found at the expected index in the parent
                        child.borrow().metadata == node.metadata
                    } else {
                        // No child can be found at the expected index in the parent
                        false
                    }
                })
            } else { None };

            // Check to make sure node.first_child and node.last_child are equal to their corresponding
            // entries in node.children
            let first_child_set_correctly = if let Some(first_child) = node.first_child.clone() {
                first_child.upgrade().map(|first_child| {
                    if node.children.is_empty() {
                        return false;
                    }

                    if let Some(first_element_in_children) = node.children.first() {
                        // The node.first_child value was equivalent to node.children[0]
                        first_element_in_children.borrow().metadata == first_child.borrow().metadata
                    } else {
                        false
                    }
                })
            } else { None };
            let last_child_set_correctly = if let Some(last_child) = node.last_child.clone() {
                last_child.upgrade().map(|last_child| {
                    if node.children.is_empty() {
                        return false;
                    }

                    if let Some(last_element_in_children) = node.children.last() {
                        // The node.last_child value was equivalent to node.children[-1]
                        last_element_in_children.borrow().metadata == last_child.borrow().metadata
                    } else {
                        false
                    }
                })
            } else { None };

            let next_set_correctly = validate_node_next(wrapped_node, parent_expected_index_within_children);

            // TODO
            let previous_set_correctly = None;

            let flags = format!(
                "{} {} {} {} {}",
                if let Some(result) = node_was_found_at_expected_index_in_parent {
                    format!("parent?={}", if result { "YES".into() } else { "NO".on_red() })
                } else { "".into() },
                if node.metadata == NodeMetadata::Empty {
                    format!("first_child?={}", match first_child_set_correctly {
                        Some(true) => "YES".into(),
                        Some(false) => "NO".on_red(),
                        None => "N/A".bright_black(),
                    })
                } else { "".into() },
                if node.metadata == NodeMetadata::Empty {
                    format!("last_child?={}", match last_child_set_correctly {
                        Some(true) => "YES".into(),
                        Some(false) => "NO".on_red(),
                        None => "N/A".bright_black(),
                    })
                } else { "".into() },
                format!("next?={}", match next_set_correctly {
                    NodeNextValidReason::Yes => "YES".into(),
                    NodeNextValidReason::InIsolatedTree | NodeNextValidReason::ParentWeakRefMissing => "N/A".bright_black(),
                    reason => format!("{reason:?}").on_red(),
                }),
                format!("previous?={}", match previous_set_correctly {
                    Some(true) => "YES".into(),
                    Some(false) => "NO".on_red(),
                    None => "N/A".bright_black(),
                }),
            );

            flags
        };

        println!(
            "{spacer}{}. metadata={:?} next={:?} prev={:?}\t\t{}",
            if let Some(index) = parent_expected_index_within_children { format!("{index}") } else { "0".into() },
            node.metadata,
            node.next.clone().map(|next| next.upgrade()).flatten().map(|next| next.borrow().metadata.clone()),
            node.previous.clone().map(|previous| previous.upgrade()).flatten().map(|previous| previous.borrow().metadata.clone()),
            validation_flags,
        );

        if node.metadata == NodeMetadata::Empty && node.children.is_empty() {
            println!("{spacer}  (no children)")
        } else {
            let new_spacer = &format!("{spacer}  ");
            let mut counter = 0;
            for child in &node.children {
                Self::dump_child(child, new_spacer, Some(counter));
                counter += 1;
            }
        }
    }

    fn append_child(
        parent: Rc<RefCell<Self>>,
        child: Rc<RefCell<Self>>,
    ) -> Rc<RefCell<Self>> {
        {
            let mut child_mut = child.borrow_mut();

            // Step 1: Add child.parent to be parent
            (*child_mut).parent = Some(Rc::downgrade(&parent));

            // Step 2: Update child.previous to be (OLD) parent.last_child
            (*child_mut).previous = parent.borrow().last_child.clone();

            // Step 3: Update child.next to be (OLD) parent.last_child.next
            (*child_mut).previous = if let Some(last_child) = parent.borrow().last_child.clone() {
                if let Some(upgraded) = last_child.upgrade() {
                    upgraded.borrow().next.clone()
                } else {
                    None
                }
            } else {
                None
            };
        }

        {
            let mut parent_mut = parent.borrow_mut();

            // Step 3: Update parent.first_child to be child IF parent.first_child is None
            if parent_mut.first_child.is_none() {
                (*parent_mut).first_child = Some(Rc::downgrade(&child));
            }

            // Step 4: Update parent.last_child.next to be child
            if let Some(last_child) = &parent_mut.last_child {
                if let Some(upgraded_last_child) = last_child.upgrade() {
                    (*upgraded_last_child.borrow_mut()).next = Some(Rc::downgrade(&child));
                }
            }

            // Step 5: Add child into `parent.children`
            (*parent_mut).children.push(child.clone());

            // Step 6: Update parent.last_child to be child
            (*parent_mut).last_child = Some(Rc::downgrade(&child));
        }

        parent
    }
}

fn main() {
    let parent = InMemoryNode::new_empty();
    let foo = InMemoryNode::new_from_literal("foo");
    let bar = InMemoryNode::new_from_literal("bar");
    let baz = InMemoryNode::new_from_literal("baz");

    let foo = InMemoryNode::append_child(foo, baz);
    let parent = InMemoryNode::append_child(parent, foo);
    let parent = InMemoryNode::append_child(parent, bar);

    InMemoryNode::dump(&parent);
}
