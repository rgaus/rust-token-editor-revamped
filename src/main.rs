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
    parent: Option<Rc<RefCell<InMemoryNode>>>,
    children: Vec<Weak<RefCell<InMemoryNode>>>,

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

    fn dump(node: Rc<RefCell<Self>>) {
        Self::dump_child( &Rc::downgrade(&node), "");
    }
    fn dump_child(node: &Weak<RefCell<Self>>, spacer: &str) {
        if let Some(upgraded) = node.upgrade() {
            let borrowed = upgraded.borrow();

            println!(
                "{spacer}- metadata={:?} parent={:?}",
                borrowed.metadata,
                borrowed.parent,
            );
            if borrowed.metadata == NodeMetadata::Empty && borrowed.children.is_empty() {
                println!("{spacer}  (no children)")
            } else {
                let new_spacer = &format!("{spacer}  ");
                for child in &borrowed.children {
                    Self::dump_child(child, new_spacer);
                }
            }
        }
    }

    fn append_child(
        parent: Rc<RefCell<Self>>,
        child: Rc<RefCell<Self>>,
    ) -> (Rc<RefCell<Self>>, Rc<RefCell<Self>>) {
        {
            let mut child_mut = child.borrow_mut();

            // Step 1: Add parent as `child.parent`
            (*child_mut).parent = Some(parent.clone());
        }

        {
            let mut parent_mut = parent.borrow_mut();
            // Step 2: Add child into `parent.children`
            (*parent_mut).children.push(Rc::downgrade(&child));
        }

        (parent, child)
    }
}

fn main() {
    let parent = InMemoryNode::new_empty();
    let foo = InMemoryNode::new_from_literal("foo");
    let bar = InMemoryNode::new_from_literal("bar");

    let (parent, _child) = InMemoryNode::append_child(parent, foo);
    let (parent, _child) = InMemoryNode::append_child(parent, bar);

    InMemoryNode::dump(parent);
}
