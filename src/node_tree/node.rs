use crate::node_tree::{
    node_debug_validators::{
        validate_node_next,
        validate_node_previous,
        NodeNextValidReason,
        NodePreviousValidReason,
    },
    utils::{Direction, Inclusivity},
    fractional_index::FractionalIndex,
};
use colored::Colorize;
use std::{
    cell::{RefCell, RefMut},
    rc::{Rc, Weak},
};

/// An enum used by seek_forwards_until to control how seeking should commence.
pub enum NodeSeek<Item> {
    Continue(Item), // Seek to the next token
    Stop,           // Finish and don't include this token
    Done(Item),     // Finish and do include this token
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeMetadata {
    Empty,
    Literal(String),
    Whitespace(String),
}

/// A node is the building block of a node tree, and repreents a node in an AST-like structure.
/// Nodes are linked both as a tree (ie, parent / children / etc) as well as doubly linked as a
/// linked list (ie, next / previous) to allow for fast traversal both linearly (ie, for printing
/// outputs) and hierarchically (ie, for performing language server like tasks)
#[derive(Debug, Clone)]
pub struct InMemoryNode {
    pub index: FractionalIndex,
    pub metadata: NodeMetadata,

    // Tree data structure refs:
    pub parent: Option<Weak<RefCell<InMemoryNode>>>,
    pub children: Vec<Rc<RefCell<InMemoryNode>>>,
    pub child_index: Option<usize>,
    pub first_child: Option<Weak<RefCell<InMemoryNode>>>,
    pub last_child: Option<Weak<RefCell<InMemoryNode>>>,

    // Linked list data structure refs:
    pub next: Option<Weak<RefCell<InMemoryNode>>>,
    pub previous: Option<Weak<RefCell<InMemoryNode>>>,
}

impl InMemoryNode {
    pub fn new_empty() -> Rc<RefCell<Self>> {
        Self::new_with_metadata(NodeMetadata::Empty)
    }
    pub fn new_from_literal(literal: &str) -> Rc<RefCell<Self>> {
        Self::new_with_metadata(NodeMetadata::Literal(literal.into()))
    }
    pub fn new_with_metadata(metadata: NodeMetadata) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            index: FractionalIndex::start(),
            metadata,
            parent: None,
            children: vec![],
            child_index: None,
            first_child: None,
            last_child: None,
            next: None,
            previous: None,
        }))
    }

    /// Given a literal string, returns a token tree which represents the literal using a series of
    /// nodes all under a common parent. Each node contains an `chars_per_node` characters except
    /// for the final one, which may contain less if literal.len() % chars_per_node != 0
    pub fn new_tree_from_literal_in_chunks(literal: &str, chars_per_node: usize) -> Rc<RefCell<Self>> {
        let parent = Self::new_empty();
        let literal: String = literal.into();

        let literal_char_vector = literal.chars().collect::<Vec<char>>();
        let literal_chunks = literal_char_vector.chunks(chars_per_node).map(|char_slice| char_slice.iter().collect::<String>());

        for literal in literal_chunks {
            let chunk_node = Self::new_from_literal(&literal);
            Self::append_child(&parent, chunk_node);
        }

        parent
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
            let previous_set_correctly = validate_node_previous(wrapped_node);

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
                        NodePreviousValidReason::Yes => "YES".into(),
                        NodePreviousValidReason::ParentWeakRefMissing
                        | NodePreviousValidReason::InIsolatedTree => "N/A".bright_black(),
                        reason => format!("{reason:?}").on_red(),
                    }
                ),
            );

            flags
        };

        println!(
            "{spacer}{}. metadata={:?} next={:?} prev={:?}\t\t{}",
            node.child_index.or(Some(0)).unwrap(),
            // if let Some(index) = parent_expected_index_within_children {
            //     format!("{index}")
            // } else {
            //     "0".into()
            // },
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

    pub fn literal(node: &Rc<RefCell<Self>>) -> String {
        if let NodeMetadata::Literal(literal) = node.borrow().metadata.clone() {
            literal
        } else {
            "".into()
        }
    }
    pub fn literal_substring(node: &Rc<RefCell<Self>>, start: usize, length: usize) -> String {
        Self::literal(node).chars().skip(start).take(length).collect::<String>()
    }
    pub fn set_literal(node: &Rc<RefCell<Self>>, new_literal: &str) {
        (*node.borrow_mut()).metadata = NodeMetadata::Literal(new_literal.into());
    }

    /// This is called after a node is inserted into the tree to assign it a correct fractional
    /// index.
    // fn recompute_index(node: &Rc<RefCell<Self>>) {
    fn assign_index(mut node_mut: RefMut<'_, Self>) {
        let new_index = {
            FractionalIndex::generate_or_fallback(
                node_mut.previous.clone().map(|n| n.upgrade()).flatten().map(|n| n.borrow().index),
                node_mut.next.clone().map(|n| n.upgrade()).flatten().map(|n| n.borrow().index),
            )
        };
        (*node_mut).index = new_index;
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

    /// Given a node, get its depth in the tree - ie, how many nodes would beed to be traversed down
    /// starting at the top-level parent to get to this node?
    pub fn depth(node: &Rc<RefCell<Self>>) -> usize {
        let mut depth = 0;
        let mut cursor = node.clone();
        loop {
            let Some(parent) = cursor.borrow().parent.clone() else {
                break;
            };
            let Some(upgraded) = parent.upgrade() else {
                break;
            };
            depth += 1;
            cursor = upgraded.clone();
        }

        depth
    }

    // TODO: prepend_child

    pub fn append_child(parent: &Rc<RefCell<Self>>, child: Rc<RefCell<Self>>) {
        println!(
            "CHILD: {:?} PARENT: {:?}",
            child.borrow().metadata,
            parent.borrow().metadata
        );
        {
            let mut child_mut = child.borrow_mut();

            // Step 1: Add child.parent to be parent
            (*child_mut).parent = Some(Rc::downgrade(&parent));

            // Step 2: make the new child's next either:
            //         c. parent.(OLD) next              (If this is the first child being added to
            //                                            this parent)
            //         a. parent.(OLD) last_child.deep_last_child.next (if the old last_child has
            //                                                          children of its own)
            //         b. parent.(OLD) last_child.next
            (*child_mut).next = if parent.borrow().first_child.is_none() {
                parent.borrow().next.clone() // a
            } else {
                parent
                    .borrow()
                    .last_child
                    .clone()
                    .map(|last_child| last_child.upgrade())
                    .flatten()
                    .map(|last_child| {
                        if let Some(deep_last_child) = Self::deep_last_child(last_child.clone()) {
                            deep_last_child.borrow().next.clone() // c
                        } else {
                            last_child.borrow().next.clone() // b
                        }
                    })
                    .flatten()
            };
            println!(
                "a. {:?}.next = {:?}",
                child_mut.metadata,
                child_mut
                    .next
                    .clone()
                    .map(|n| n.upgrade())
                    .flatten()
                    .map(|n| n.borrow().metadata.clone())
            );

            // Step N: make the new child's previous either:
            //         a. parent.(OLD) last_child.deep_last_child (if the old last_child has
            //                                                     children of its own)
            //         b. parent.(OLD) last_child
            //         c. parent (if this is the first child being added to the parent)
            (*child_mut).previous = parent
                .borrow()
                .last_child
                .clone() // a
                .map(|n| n.upgrade())
                .flatten()
                .map(|n| Self::deep_last_child(n))
                .flatten()
                .map(|n| Rc::downgrade(&n))
                .or_else(|| parent.borrow().last_child.clone()) // b
                .or_else(|| Some(Rc::downgrade(&parent))); // c

            // // Step 3: Update child.next to be parent.(OLD) last_child.next
            // (*child_mut).previous = if let Some(last_child) = parent.borrow().last_child.clone() {
            //     if let Some(upgraded) = last_child.upgrade() {
            //         upgraded.borrow().next.clone()
            //     } else {
            //         None
            //     }
            // } else {
            //     None
            // };

            // Step N: set this child's index in its parent's children array 
            (*child_mut).child_index = Some(parent.borrow().children.len());

            // Step N: After linking child.previous and child.next, assign this child its new
            // fractional index
            Self::assign_index(child_mut);
        }

        // Step 4: Update the parent's next sibling's previous
        // (ie, parent.(OLD) deep_last_child.next.previous) to be child
        if let Some(deep_last_child) = InMemoryNode::deep_last_child(parent.clone()).or_else(|| Some(parent.clone())) {
            if let Some(Some(deep_last_child_next)) = deep_last_child.borrow().next.clone().map(|n| n.upgrade()) {
                deep_last_child_next.borrow_mut().previous = Some(Rc::downgrade(&child));
            }
        }

        {
            let mut parent_mut = parent.borrow_mut();

            if parent_mut.first_child.is_none() {
                // Step 3: Update parent.first_child to be child IF this is the first node being
                // added to this parent
                (*parent_mut).first_child = Some(Rc::downgrade(&child));
                println!(
                    "b. {:?}.next = {:?}",
                    parent_mut.metadata,
                    child.borrow().metadata
                );

                // Step 5: Update parent.next to be child IF this is the first node being added
                // to this parent
                (*parent_mut).next = Some(Rc::downgrade(&child));
            }

            // Step 4: set parent.(OLD) last_child.deep_last_child.next to child
            if let Some(parent_last_child) = &parent_mut.last_child {
                if let Some(parent_last_child) = parent_last_child.upgrade() {
                    if let Some(foo) = Self::deep_last_child(parent_last_child) {
                        (*foo.borrow_mut()).next = Some(Rc::downgrade(&child));
                        println!(
                            "c. {:?}.next = {:?}",
                            foo.borrow().metadata,
                            child.borrow().metadata
                        );
                    }
                }
            }

            // Step 5: Update parent.(OLD) last_child.next to be child
            if let Some(last_child) = &parent_mut.last_child {
                if let Some(upgraded_last_child) = last_child.upgrade() {
                    let parent_old_last_child = upgraded_last_child.borrow().first_child.clone(); // child.first_child

                    println!(
                        "d. {:?}.next = {:?}.or({:?})",
                        upgraded_last_child.borrow().metadata,
                        parent_old_last_child
                            .clone()
                            .map(|n| n.upgrade())
                            .flatten()
                            .map(|n| n.borrow().metadata.clone()),
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
    }

    /// Removes a child node from a tree. Returns the parent node of the removed node, or None if
    /// the node that was removed was at the top level.
    pub fn remove_child_at_index(parent: &Rc<RefCell<Self>>, index: usize) {
        println!("REMOVE: {:?} INDEX: {:?}", parent.borrow().metadata, index);
        let (child, previous_child, deep_last_child) = {
            let parent = parent.borrow();
            let child = parent.children.get(index);
            let Some(child) = child else {
                return;
            };

            let previous_child = child
                .borrow()
                .previous
                .clone()
                .map(|p| p.upgrade())
                .flatten();
            let deep_last_child = Self::deep_last_child(child.clone());

            (child.clone(), previous_child.clone(), deep_last_child)
        };

        // Step 1: child_mut.previous.next = child_mut.next
        // println!("PREV: {:?}", previous_child.clone().map(|p| p.borrow().metadata.clone()));
        if let Some(previous_child) = previous_child {
            (*previous_child.borrow_mut()).next = Self::deep_last_child(child.clone())
                .or_else(|| Some(child.clone()))
                .map(|n| n.borrow().next.clone())
                .flatten()
        };

        // Step 2: child_mut.deep_last_child.next.previous = child_mut.previous
        if let Some(deep_last_child) = deep_last_child.or_else(|| Some(child.clone())) {
            let deep_last_child_next = deep_last_child.borrow().next.clone();
            if let Some(Some(deep_last_child_next)) = deep_last_child_next.map(|n| n.upgrade()) {
                (*deep_last_child_next.borrow_mut()).previous = child.borrow().previous.clone();
            }
        };

        {
            let mut parent_mut = parent.borrow_mut();
            let max_child_index = parent_mut.children.len() - 1;

            // Step N: Reassign first_child / last_childto no longer take into account the child
            if index == 0 {
                (*parent_mut).first_child =
                    parent_mut.children.get(1).map(|child| Rc::downgrade(child));
            }
            if index == max_child_index {
                (*parent_mut).last_child = if max_child_index > 0 {
                    parent_mut
                        .children
                        .get(max_child_index - 1)
                        .map(|child| Rc::downgrade(child))
                } else {
                    None
                };
            }

            // Remove the node from `children`, which should cause the child's memory to get freed
            (*parent_mut).children.remove(index);

            // Update all `child_index` values on the children afterwards to take into account its
            // new index in `children`.
            for child_index in index..(parent_mut.children.len()) {
                let mut child_mut = parent_mut.children[child_index].borrow_mut();
                child_mut.child_index = match child_mut.child_index {
                    Some(child_index) => Some(child_index - 1),
                    None => None,
                };
            }
        }
    }

    /// When called, removes all children from a node in the node tree, relinking nodes properly so
    /// it is as if the children were never there.
    pub fn remove_all_children(parent: &Rc<RefCell<Self>>) {
        for (index, _child) in parent.borrow().children.iter().enumerate() {
            Self::remove_child_at_index(parent, index);
        };
    }

    /// When called, swaps the child within `parent` at `index` with the `new_child`.
    /// If `new_child` itself has children, this subtree is spliced in to replace the old child.
    pub fn swap_child_at_index(
        parent: &Rc<RefCell<Self>>,
        index: usize,
        new_child: Rc<RefCell<Self>>,
    ) {
        println!(
            "SWAP: {:?} INDEX: {} NEW: {:?}",
            parent.borrow().metadata,
            index,
            new_child.borrow().metadata
        );

        let (old_child, old_child_previous, old_child_deep_last_child) = {
            let parent = parent.borrow();
            let old_child = parent.children.get(index);
            let Some(old_child) = old_child else {
                return;
            };

            let previous_child = old_child
                .borrow()
                .previous
                .clone()
                .map(|p| p.upgrade())
                .flatten();
            let deep_last_child = Self::deep_last_child(old_child.clone());

            (old_child.clone(), previous_child.clone(), deep_last_child)
        };

        let new_child_deep_last_child = Self::deep_last_child(new_child.clone());

        {
            let mut new_child_mut = new_child.borrow_mut();

            // Step N: Update new_child.parent to the common parent
            (*new_child_mut).parent = Some(Rc::downgrade(parent));

            // Step N: Relink the old_child.previous's next to point to new_child
            if let Some(old_previous) = old_child_previous.clone() {
                old_previous.borrow_mut().next = Some(Rc::downgrade(&new_child));
            }

            // Step N: Relink the old_child.next's previous to point to new_child.deep_last_child
            if let Some(new_child_deep_last_child) = new_child_deep_last_child {
                (*new_child_deep_last_child.borrow_mut()).next = old_child_deep_last_child
                    .clone()
                    .map(|n| n.borrow().next.clone())
                    .flatten()
                    .or_else(|| old_child.borrow().next.clone());
            } else {
                (*new_child_mut).next = old_child_deep_last_child
                    .clone()
                    .map(|n| n.borrow().next.clone())
                    .flatten()
                    .or_else(|| old_child.borrow().next.clone());
            }

            // Step N: Update new_child.next to be old_child.deep_last_child.next
            (*new_child_mut).next = if let Some(deep_last_child) = old_child_deep_last_child.clone()
            {
                deep_last_child.borrow().next.clone()
            } else {
                old_child.borrow().next.clone()
            };

            // Step N: Update the next sibling of old_child to point back to it
            //         ie, old_child.(OLD) deep_last_child,next.previous to new_child
            if let Some(deep_last_child_next) = old_child_deep_last_child
                .clone()
                .map(|n| n.borrow().next.clone())
                .flatten()
                .map(|n| n.upgrade())
                .flatten()
            {
                (*deep_last_child_next.borrow_mut()).previous = Some(Rc::downgrade(&new_child));
            } else if let Some(old_child_next) = old_child
                .borrow()
                .next
                .clone()
                .map(|n| n.upgrade())
                .flatten()
            {
                (*old_child_next.borrow_mut()).next = Some(Rc::downgrade(&new_child));
            };

            // Step N: Update new_child.previous to be old_child.previous
            (*new_child_mut).previous = old_child.borrow().previous.clone();
        }

        {
            let mut parent_mut = parent.borrow_mut();
            let max_child_index = parent_mut.children.len() - 1;

            // Step N: Reassign first_child / last_child to point to new_child, if required
            if index == 0 {
                (*parent_mut).first_child = Some(Rc::downgrade(&new_child));
            }
            if index == max_child_index {
                (*parent_mut).last_child = Some(Rc::downgrade(&new_child));
            }

            // Step N: remove old child and add new child in its place
            (*parent_mut).children.remove(index);
            (*parent_mut).children.insert(index, new_child);
        }
    }

    pub fn seek_until<UntilFn, ResultItem>(
        node: &Rc<RefCell<Self>>,
        direction: Direction,
        current_node_included: Inclusivity,
        until_fn: UntilFn,
    ) -> std::vec::IntoIter<ResultItem>
    where
        UntilFn: FnMut(&Rc<RefCell<Self>>, usize) -> NodeSeek<ResultItem>,
    {
        match direction {
            Direction::Forwards => Self::seek_forwards_until(node, current_node_included, until_fn),
            Direction::Backwards => Self::seek_backwards_until(node, current_node_included, until_fn),
        }
    }

    /// Given a starting node `node`, seek forwards via next, calling `until_fn` repeatedly for
    /// each node to determine how to proceed.
    ///
    /// If `current_node_included` is Inclusivity::Inclusive, then `until_fn` is called with the
    /// `node` at the start before continuing the seek. If it is Inclusivity::Exclusive, then the
    /// node's next node is the first node fed into `until_fn`.
    ///
    /// Returns an iterator of the return value of each call to `until_fn` that have been properly matched.
    pub fn seek_forwards_until<UntilFn, ResultItem>(
        node: &Rc<RefCell<Self>>,
        current_node_included: Inclusivity,
        mut until_fn: UntilFn,
    ) -> std::vec::IntoIter<ResultItem>
    where
        UntilFn: FnMut(&Rc<RefCell<Self>>, usize) -> NodeSeek<ResultItem>,
    {
        let cursor = match current_node_included {
            Inclusivity::Inclusive => Some(node.clone()),
            Inclusivity::Exclusive => node.borrow().next.clone().map(|n| n.upgrade()).flatten(),
        };
        let Some(mut cursor) = cursor else {
            // The cursor node is None, so bail early!
            return (vec![]).into_iter();
        };

        let mut output = vec![];
        let mut iteration_counter: usize = 0;
        loop {
            match until_fn(&cursor, iteration_counter) {
                NodeSeek::Continue(result) => {
                    // Continue looping to the next node!
                    output.push(result);

                    let cursor_next = cursor.borrow().next.clone().map(|n| n.upgrade()).flatten();
                    let Some(cursor_next) = cursor_next else {
                        // We've reached the end!
                        break;
                    };

                    cursor = cursor_next;
                    iteration_counter += 1;
                    continue;
                }
                NodeSeek::Stop => {
                    break;
                }
                NodeSeek::Done(result) => {
                    output.push(result);
                    break;
                }
            }
        }

        output.into_iter()
    }

    /// Given a starting node `node`, seek backwards via previous, calling `until_fn` repeatedly for
    /// each node to determine how to proceed.
    ///
    /// If `current_node_included` is Inclusivity::Inclusive, then `until_fn` is called with the
    /// `node` at the start before continuing the seek. If it is Inclusivity::Exclusive, then the
    /// node's previous node is the first node fed into `until_fn`.
    ///
    /// Returns an iterator of the return value of each call to `until_fn` that have been properly matched.
    pub fn seek_backwards_until<UntilFn, ResultItem>(
        node: &Rc<RefCell<Self>>,
        current_node_included: Inclusivity,
        mut until_fn: UntilFn,
    ) -> std::vec::IntoIter<ResultItem>
    where
        UntilFn: FnMut(&Rc<RefCell<Self>>, usize) -> NodeSeek<ResultItem>,
    {
        let cursor = match current_node_included {
            Inclusivity::Inclusive => Some(node.clone()),
            Inclusivity::Exclusive => node.borrow().previous.clone().map(|n| n.upgrade()).flatten(),
        };
        let Some(mut cursor) = cursor else {
            // The cursor node is None, so bail early!
            return (vec![]).into_iter();
        };

        let mut output = vec![];
        let mut iteration_counter: usize = 0;
        loop {
            match until_fn(&cursor, iteration_counter) {
                NodeSeek::Continue(result) => {
                    // Continue looping to the previous node!
                    output.push(result);

                    let cursor_previous = cursor.borrow().previous.clone().map(|n| n.upgrade()).flatten();
                    let Some(cursor_previous) = cursor_previous else {
                        // We've reached the end!
                        break;
                    };

                    cursor = cursor_previous;
                    iteration_counter += 1;
                    continue;
                }
                NodeSeek::Stop => {
                    break;
                }
                NodeSeek::Done(result) => {
                    output.push(result);
                    break;
                }
            }
        }

        output.into_iter()
    }

    /// Given a starting node `start_node`, delete from that node the the next node and onwards,
    /// as long as the `until_fn` predicate function passes. if `start_node_included` is
    /// Inclusivity::Exclusive, begin iterating AFTER the start_node rather than right at it.
    pub fn remove_nodes_sequentially_until<UntilFn, ResultItem>(
        start_node: &Rc<RefCell<Self>>,
        start_node_included: Inclusivity,
        until_fn: UntilFn,
    ) -> std::vec::IntoIter<ResultItem>
    where
        UntilFn: Fn(&Rc<RefCell<Self>>, usize) -> NodeSeek<ResultItem>
    {
        let node_value_pairs = Self::seek_forwards_until(start_node, start_node_included, |node, index| {
            match until_fn(node, index) {
                NodeSeek::Continue(value) => NodeSeek::Continue((node.clone(), value)),
                NodeSeek::Done(value) => NodeSeek::Done((node.clone(), value)),
                NodeSeek::Stop => NodeSeek::Stop,
            }
        });

        let (nodes, values): (Vec<_>, Vec<_>) = node_value_pairs.unzip();

        // TODO: optimize this to do these deletes in bulk once it becomes a problem, a lot of
        // duplicate pointer assignment in InMemoryNode::remove_child_at_index could probably
        // be saved
        for node in nodes {
            let Some(Some(parent)) = node.borrow().parent.clone().map(|n| n.upgrade()) else {
                continue;
            };
            let Some(child_index) = node.borrow().child_index else {
                continue;
            };
            InMemoryNode::remove_child_at_index(&parent, child_index);
        }

        values.into_iter()
    }
}

impl PartialEq for InMemoryNode {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl PartialOrd for InMemoryNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.index < other.index {
            Some(std::cmp::Ordering::Less)
        } else if self.index > other.index {
            Some(std::cmp::Ordering::Greater)
        } else {
            Some(std::cmp::Ordering::Equal)
        }
    }
}

