use colored::ColoredString;
use std::{cell::RefCell, rc::Rc};

use crate::node_tree::node::{InMemoryNode, TokenKindTrait};

/// The size of characters from the literal stored in each ast node.
const RAW_LITERAL_NODE_CHUNK_SIZE_CHARS: usize = 32;

/// The Raw language definition performs no syntax parsing at all, and purely treats the whole text
/// document as unparsable text.
#[derive(Debug, Clone, PartialEq)]
pub enum SyntaxKind {}

impl TokenKindTrait for SyntaxKind {
    fn apply_debug_syntax_color(
        text: String,
        _ancestry: std::vec::IntoIter<SyntaxKind>,
    ) -> ColoredString {
        text.into()
    }

    // Any node can be reparsed, since a reparse is effectively a noop at the moment
    fn is_reparsable(&self) -> bool {
        true
    }

    fn parse(
        literal: &str,
        _parent: Option<Rc<RefCell<InMemoryNode<Self>>>>,
    ) -> Rc<RefCell<InMemoryNode<Self>>> {
        InMemoryNode::new_tree_from_literal_in_chunks(literal, RAW_LITERAL_NODE_CHUNK_SIZE_CHARS)
    }
}
