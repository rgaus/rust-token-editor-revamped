use std::{rc::Rc, cell::RefCell};
use crate::node::{InMemoryNode, NodeMetadata};

#[derive(Debug, PartialEq)]
enum State {
    Initial,
    Block,
}

struct MiniJsParser {
    pub root: Rc<RefCell<InMemoryNode>>,
    pub cursor: Rc<RefCell<InMemoryNode>>,
    pub unrecognized: Option<Rc<RefCell<InMemoryNode>>>,
    pub state: State,
    pub char_counter: usize,
}

impl MiniJsParser {
    pub fn new() -> Self {
        let root = InMemoryNode::new_empty();
        Self {
            cursor: root.clone(),
            root,
            unrecognized: None,
            state: State::Initial,
            char_counter: 0,
        }
    }
    pub fn parse_string(input: &str) -> Rc<RefCell<InMemoryNode>> {
        let mut parser = Self::new();
        parser.send(input);
        parser.root
    }

    pub fn send(self: &mut Self, input: &str) {
        let _ = input.chars().map(|char| {
            // println!("CHAR: {char}");
            if char == '{' {
                // Open block
                let block = InMemoryNode::new_from_literal("BLOCK");
                InMemoryNode::append_child(&self.cursor, block.clone());
                self.state = State::Block;
                self.cursor = block;
                self.unrecognized = None;

            } else if char == '}' {
                // Close block
                let parent = self.cursor.borrow().clone().parent.map(|p| p.upgrade());
                if let Some(Some(parent)) = parent {
                    self.cursor = parent;
                }
                self.unrecognized = None;

            } else if char.is_whitespace() {
                if let Some(unrecognized) = &self.unrecognized {
                    let mut unrecognized_mut = unrecognized.borrow_mut();
                    if let NodeMetadata::Whitespace(literal) = unrecognized_mut.metadata.clone() {
                        println!("A: '{literal}' '{char}' {unrecognized_mut:?}");
                        (*unrecognized_mut).metadata = NodeMetadata::Literal(format!("{}{}", literal, char));
                    }
                } else {
                    println!("B: '{char}'");
                    let whitespace = InMemoryNode::new_with_metadata(NodeMetadata::Whitespace(format!("{char}")));
                    InMemoryNode::append_child(&self.cursor, whitespace.clone());
                    self.unrecognized = Some(whitespace);
                }

            } else {
                if let Some(unrecognized) = self.unrecognized.clone() {
                    let mut unrecognized_mut = unrecognized.borrow_mut();
                    if let NodeMetadata::Literal(literal) = unrecognized_mut.metadata.clone() {
                        unrecognized_mut.metadata = NodeMetadata::Literal(format!("{}{}", literal, char));
                    }
                } else {
                    let literal = format!("{char}");
                    let unrecognized = InMemoryNode::new_from_literal(&literal);
                    self.unrecognized = Some(unrecognized.clone());
                    InMemoryNode::append_child(&self.cursor, unrecognized);
                }
            }
            self.char_counter += 1;
        }).collect::<()>();
    }
}

pub fn parse_string(input: &str) -> Rc<RefCell<InMemoryNode>> {
    MiniJsParser::parse_string(input)
}
