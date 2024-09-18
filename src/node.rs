use crate::node_debug_validators::{validate_node_next, NodeNextValidReason};
use colored::Colorize;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

#[derive(Debug, Clone, PartialEq)]
pub enum NodeMetadata {
    Empty,
    Literal(String),
}

#[derive(Debug, Clone)]
pub struct InMemoryNode {
    pub metadata: NodeMetadata,

    // Tree data structure refs:
    pub parent: Option<Weak<RefCell<InMemoryNode>>>,
    pub children: Vec<Rc<RefCell<InMemoryNode>>>,
    pub first_child: Option<Weak<RefCell<InMemoryNode>>>,
    pub last_child: Option<Weak<RefCell<InMemoryNode>>>,

    // Linked list data structure refs:
    pub next: Option<Weak<RefCell<InMemoryNode>>>,
    pub previous: Option<Weak<RefCell<InMemoryNode>>>,
}

impl InMemoryNode {
    pub fn new_empty() -> Rc<RefCell<Self>> {
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
    pub fn new_from_literal(literal: &str) -> Rc<RefCell<Self>> {
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

    pub fn dump(node: &Rc<RefCell<Self>>) {
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
            let node_was_found_at_expected_index_in_parent =
                if let (Some(parent_expected_index_within_children), Some(parent)) =
                    (parent_expected_index_within_children, node.parent.clone())
                {
                    parent.upgrade().map(|parent| {
                        let borrowed_parent = parent.borrow();
                        let child = borrowed_parent
                            .children
                            .get(parent_expected_index_within_children);
                        if let Some(child) = child {
                            // A child can be found at the expected index in the parent
                            child.borrow().metadata == node.metadata
                        } else {
                            // No child can be found at the expected index in the parent
                            false
                        }
                    })
                } else {
                    None
                };

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
            } else {
                None
            };
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
            } else {
                None
            };

            let next_set_correctly =
                validate_node_next(wrapped_node, parent_expected_index_within_children);

            // TODO
            let previous_set_correctly = None;

            let flags = format!(
                "{} {} {} {} {}",
                if let Some(result) = node_was_found_at_expected_index_in_parent {
                    format!(
                        "parent?={}",
                        if result { "YES".into() } else { "NO".on_red() }
                    )
                } else {
                    "".into()
                },
                if node.metadata == NodeMetadata::Empty {
                    format!(
                        "first_child?={}",
                        match first_child_set_correctly {
                            Some(true) => "YES".into(),
                            Some(false) => "NO".on_red(),
                            None => "N/A".bright_black(),
                        }
                    )
                } else {
                    "".into()
                },
                if node.metadata == NodeMetadata::Empty {
                    format!(
                        "last_child?={}",
                        match last_child_set_correctly {
                            Some(true) => "YES".into(),
                            Some(false) => "NO".on_red(),
                            None => "N/A".bright_black(),
                        }
                    )
                } else {
                    "".into()
                },
                format!(
                    "next?={}",
                    match next_set_correctly {
                        NodeNextValidReason::Yes => "YES".into(),
                        NodeNextValidReason::InIsolatedTree
                        | NodeNextValidReason::ParentWeakRefMissing => "N/A".bright_black(),
                        reason => format!("{reason:?}").on_red(),
                    }
                ),
                format!(
                    "previous?={}",
                    match previous_set_correctly {
                        Some(true) => "YES".into(),
                        Some(false) => "NO".on_red(),
                        None => "N/A".bright_black(),
                    }
                ),
            );

            flags
        };

        println!(
            "{spacer}{}. metadata={:?} next={:?} prev={:?}\t\t{}",
            if let Some(index) = parent_expected_index_within_children {
                format!("{index}")
            } else {
                "0".into()
            },
            node.metadata,
            node.next
                .clone()
                .map(|next| next.upgrade())
                .flatten()
                .map(|next| next.borrow().metadata.clone()),
            node.previous
                .clone()
                .map(|previous| previous.upgrade())
                .flatten()
                .map(|previous| previous.borrow().metadata.clone()),
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

    /// Given a node, gets its "deep last child" - ie, the last child of the last child
    /// of the ... etc
    ///
    /// This is an important value when doing certain relinking operations.
    pub fn deep_last_child(node: Rc<RefCell<Self>>) -> Option<Rc<RefCell<Self>>> {
        if node.borrow().last_child.is_none() {
            return None;
        };

        let mut cursor = node;
        loop {
            let Some(last_child) = cursor.borrow().last_child.clone() else {
                break;
            };
            let Some(upgraded) = last_child.upgrade() else {
                break;
            };
            cursor = upgraded.clone();
        }

        Some(cursor)
    }

    pub fn append_child(parent: Rc<RefCell<Self>>, child: Rc<RefCell<Self>>) -> Rc<RefCell<Self>> {
        println!("CHILD: {:?} PARENT: {:?}", child.borrow().metadata, parent.borrow().metadata);
        {
            let mut child_mut = child.borrow_mut();

            // Step 1: Add child.parent to be parent
            (*child_mut).parent = Some(Rc::downgrade(&parent));

            // Step 2: Update child.next to be (OLD) parent.last_child.next
            (*child_mut).next = parent.borrow().last_child.clone()
                .map(|last_child| last_child.upgrade())
                .flatten()
                .map(|last_child| last_child.borrow().next.clone())
                .flatten();

            // // Step 3: Update child.next to be (OLD) parent.last_child.next
            // (*child_mut).previous = if let Some(last_child) = parent.borrow().last_child.clone() {
            //     if let Some(upgraded) = last_child.upgrade() {
            //         upgraded.borrow().next.clone()
            //     } else {
            //         None
            //     }
            // } else {
            //     None
            // };
        }

        {
            let mut parent_mut = parent.borrow_mut();

            // Step 3: Update parent.first_child to be child IF parent.first_child is None
            if parent_mut.first_child.is_none() {
                (*parent_mut).first_child = Some(Rc::downgrade(&child));
                println!("a. {:?}.next = {:?}", parent_mut.metadata, child.borrow().metadata);
                (*parent_mut).next = Some(Rc::downgrade(&child));
            }

            // Step 4: set parent.(OLD) last_child.deep_last_child.next to child
            if let Some(parent_last_child) = &parent_mut.last_child {
                if let Some(parent_last_child) = parent_last_child.upgrade() {
                    if let Some(foo) = Self::deep_last_child(parent_last_child) {
                        (*foo.borrow_mut()).next = Some(Rc::downgrade(&child));
                        println!("b. {:?}.next = {:?}", foo.borrow().metadata, child.borrow().metadata);
                    }
                }
            }

            // Step 5: Update parent.(OLD) last_child.next to be child
            if let Some(last_child) = &parent_mut.last_child {
                if let Some(upgraded_last_child) = last_child.upgrade() {
                    let parent_old_last_child = upgraded_last_child.borrow().first_child.clone(); // child.first_child

                    println!(
                        "c. {:?}.next = {:?}.or({:?})",
                        upgraded_last_child.borrow().metadata,
                        parent_old_last_child.clone().map(|n| n.upgrade()).flatten().map(|n| n.borrow().metadata.clone()),
                        child.borrow().metadata,
                    );

                    // parent.(OLD) last_child
                    // or fall back to the child itself if there are no nodes inside
                    let new_next = parent_old_last_child.or_else(|| Some(Rc::downgrade(&child)));
                    (*upgraded_last_child.borrow_mut()).next = new_next;
                }
            }

            // Step 6: Add child into `parent.children`
            (*parent_mut).children.push(child.clone());

            // Step 7: Update parent.last_child to be child
            (*parent_mut).last_child = Some(Rc::downgrade(&child));
        }

        parent
    }
}
