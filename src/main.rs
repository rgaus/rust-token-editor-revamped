use std::{
    rc::{Rc, Weak},
    cell::RefCell,
};

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
        node: &Rc<RefCell<Self>>,
        spacer: &str,
        parent_expected_index_within_children: Option<usize>,
    ) {
        let borrowed = node.borrow();

        // Check to see if in the parent of the given child, there exists a child at the given
        // index with matching metadata
        let found_at_index_in_parent = if let (
            Some(parent_expected_index_within_children),
            Some(parent),
        ) = (parent_expected_index_within_children, borrowed.parent.clone()) {
            parent.upgrade().map(|parent| {
                let borrowed_parent = parent.borrow();
                let child = borrowed_parent.children.get(parent_expected_index_within_children);
                if let Some(child) = child {
                    // A child can be found at the expected index in the parent
                    child.borrow().metadata == borrowed.metadata
                } else {
                    // No child can be found at the expected index in the parent
                    false
                }
            })
        } else { None };

        println!(
            "{spacer}{}. metadata={:?} {}",
            if let Some(index) = parent_expected_index_within_children { format!("{index}") } else { "0".into() },
            borrowed.metadata,
            if let Some(result) = found_at_index_in_parent { format!("parent_seemingly_linked_right?={result}") } else { "".into() }
        );
        if borrowed.metadata == NodeMetadata::Empty && borrowed.children.is_empty() {
            println!("{spacer}  (no children)")
        } else {
            let new_spacer = &format!("{spacer}  ");
            let mut counter = 0;
            for child in &borrowed.children {
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

    let parent = InMemoryNode::append_child(parent, foo);
    let parent = InMemoryNode::append_child(parent, bar);

    InMemoryNode::dump(&parent);
}
