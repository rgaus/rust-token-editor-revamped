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
            next: None,
            previous: None,
        }))
    }
    fn new_from_literal(literal: &str) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            metadata: NodeMetadata::Literal(literal.into()),
            parent: None,
            children: vec![],
            next: None,
            previous: None,
        }))
    }

    fn dump(node: &Rc<RefCell<Self>>) {
        Self::dump_child(node, "", None);
    }
    fn dump_child(node: &Rc<RefCell<Self>>, spacer: &str, parent_expected_index: Option<usize>) {
        let borrowed = node.borrow();

        // Check to see if in the parent of the given child, there exists a child at the given
        // index with matching metadata
        let found_at_index_in_parent = if let (Some(parent_expected_index), Some(parent)) = (parent_expected_index, borrowed.parent.clone()) {
            parent.upgrade().map(|parent| {
                let borrowed_parent = parent.borrow();
                let child = borrowed_parent.children.get(parent_expected_index);
                if let Some(child) = child {
                    // A child can be found at the expected index in the parent
                    child.borrow().metadata == borrowed.metadata
                } else {
                    // No child can be found at the expected index in the parent
                    false
                }
            })
        } else { None };

        // borrowed.parent.children.contains(borrowed)
        println!(
            "{spacer}{}. metadata={:?} {}",
            if let Some(index) = parent_expected_index { format!("{index}") } else { "0".into() },
            borrowed.metadata,
            if let Some(result) = found_at_index_in_parent { format!("parent_linked_right?={result}") } else { "".into() }
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

            // Step 1: Add parent as `child.parent`
            (*child_mut).parent = Some(Rc::downgrade(&parent));
        }

        {
            let mut parent_mut = parent.borrow_mut();
            // Step 2: Add child into `parent.children`
            (*parent_mut).children.push(child.clone());
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
