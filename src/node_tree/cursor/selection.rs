use colored::{ColoredString, Colorize};

use crate::node_tree::{
    cursor::Cursor,
    node::{InMemoryNode, NodeSeek, TokenKindTrait},
    utils::Inclusivity,
};
use std::{cell::RefCell, fmt::Debug, rc::Rc};

use super::CursorSeek;

#[derive(Clone)]
pub struct Selection<TokenKind: TokenKindTrait> {
    pub primary: Cursor<TokenKind>,
    pub secondary: Cursor<TokenKind>,
}

impl<TokenKind: TokenKindTrait> Debug for Selection<TokenKind> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let literal_colors = self.literal_colors();
        let literal_length = self.literal().len();
        write!(
            f,
            "Selection(literal=\"{}\", len={}, primary={:?} secondary={:?})",
            literal_colors, literal_length, self.primary, self.secondary
        )
    }
}

impl<TokenKind: TokenKindTrait> Selection<TokenKind> {
    pub fn new(node: Rc<RefCell<InMemoryNode<TokenKind>>>) -> Self {
        Self::new_at(node, 0)
    }
    pub fn new_at(node: Rc<RefCell<InMemoryNode<TokenKind>>>, offset: usize) -> Self {
        let cursor = Cursor::new_at(node, offset);
        Self::new_from_cursor(cursor)
    }
    pub fn new_from_cursor(cursor: Cursor<TokenKind>) -> Self {
        Self::new_from_cursor_pair(cursor.clone(), cursor.clone())
    }
    pub fn new_from_cursor_pair(primary: Cursor<TokenKind>, secondary: Cursor<TokenKind>) -> Self {
        Self { primary, secondary }
    }

    /// When called with a node, creates a new Selection that starts at the node and spans across
    /// all of its children, ending at the end of the final child.
    ///
    /// ie: calling this function on the root node would select the entire token tree
    pub fn new_across_subtree(node: &Rc<RefCell<InMemoryNode<TokenKind>>>) -> Self {
        let deep_last_child = InMemoryNode::deep_last_child(node).unwrap_or_else(|| node.clone());
        let deep_last_child_length = InMemoryNode::literal(&deep_last_child).len();
        Self {
            primary: Cursor::new(node.clone()),
            secondary: Cursor::new_at(deep_last_child, deep_last_child_length),
        }
    }

    pub fn perform<ClosureFn>(
        self: &Self,
        inclusivity: Inclusivity,
        mut closure_fn: ClosureFn,
    ) -> Self
    where
        ClosureFn:
            FnMut(Cursor<TokenKind>, Cursor<TokenKind>) -> (Cursor<TokenKind>, Cursor<TokenKind>),
    {
        let (primary, secondary) = closure_fn(self.primary.clone(), self.secondary.clone());
        let mut result = Self::new_from_cursor_pair(primary, secondary);

        // From :help exclusive -
        //
        // A character motion is either inclusive or exclusive.  When inclusive, the
        // start and end position of the motion are included in the operation.  When
        // exclusive, the last character towards the end of the buffer is not included.
        // Linewise motions always include the start and end position.
        //
        if inclusivity == Inclusivity::Exclusive {
            let end = result.end_mut();
            *end = end.seek_forwards(CursorSeek::AdvanceByCharCount(1));
        }

        result
    }

    pub fn perform_inclusive<ClosureFn>(self: &mut Self, closure_fn: ClosureFn) -> Self
    where
        ClosureFn:
            FnMut(Cursor<TokenKind>, Cursor<TokenKind>) -> (Cursor<TokenKind>, Cursor<TokenKind>),
    {
        self.perform(Inclusivity::Inclusive, closure_fn)
    }

    pub fn perform_exclusive<ClosureFn>(self: &mut Self, closure_fn: ClosureFn) -> Self
    where
        ClosureFn:
            FnMut(Cursor<TokenKind>, Cursor<TokenKind>) -> (Cursor<TokenKind>, Cursor<TokenKind>),
    {
        self.perform(Inclusivity::Exclusive, closure_fn)
    }

    pub fn set_primary(self: &mut Self, input: Cursor<TokenKind>) -> &mut Self {
        self.primary = input;
        self
    }
    pub fn set_secondary(self: &mut Self, input: Cursor<TokenKind>) -> &mut Self {
        self.secondary = input;
        self
    }

    /// Returns the cursor furthest to the end of the selection - either `self.primary` or `self.secondary`
    pub fn end_mut(self: &mut Self) -> &mut Cursor<TokenKind> {
        if self.primary > self.secondary {
            &mut self.primary
        } else {
            &mut self.secondary
        }
    }

    /// When called, computes the underlying literal text that the selection has covered.
    pub fn literal(self: &Self) -> String {
        let colored_result = self.generate_literal(false);
        format!("{}", colored_result.clear())
    }
    /// When called, computes the underlying literal text that the selection has covered. Returns
    /// the output with terminal syntax colors injected for pretty printing.
    pub fn literal_colors(self: &Self) -> ColoredString {
        self.generate_literal(true)
    }

    /// When called, computes the underlying literal text that the selection has covered.
    fn generate_literal(self: &Self, include_terminal_colors: bool) -> ColoredString {
        // If the node selection spans within a single node, then take a substring of the common
        // literal value based on the offsets.
        if self.primary.node == self.secondary.node {
            let literal_start_offset = if self.primary.offset < self.secondary.offset {
                self.primary.offset
            } else {
                self.secondary.offset
            };
            let literal_length = self.secondary.offset.abs_diff(self.primary.offset);
            let literal_section = InMemoryNode::literal_substring(
                &self.primary.node,
                literal_start_offset,
                literal_length,
            );

            // Apply the proper colors to the string, if required
            return if include_terminal_colors {
                InMemoryNode::literal_colored(&self.primary.node, &literal_section)
            } else {
                literal_section.into()
            };
        };

        // If the node selection spans multiple nodes, then:
        //
        // 1. Find the earlier node, and store the part which is within the selection
        let earlier_cursor = if self.primary.node < self.secondary.node {
            &self.primary
        } else {
            &self.secondary
        };
        let later_cursor = if self.primary.node < self.secondary.node {
            &self.secondary
        } else {
            &self.primary
        };
        let earlier_suffix = InMemoryNode::literal_substring(
            &earlier_cursor.node,
            earlier_cursor.offset,
            InMemoryNode::literal(&earlier_cursor.node).len() - earlier_cursor.offset,
        );

        // 2. Store the first part of the later node which should be kept
        let later_prefix =
            InMemoryNode::literal_substring(&later_cursor.node, 0, later_cursor.offset);

        // 3. Accumulate the text in the in between nodes
        let in_between_node_literals = InMemoryNode::seek_forwards_until(
            &earlier_cursor.node,
            Inclusivity::Exclusive,
            |node, _ct| {
                if node == &later_cursor.node {
                    NodeSeek::Stop
                } else {
                    let literal = InMemoryNode::literal(node);
                    let literal_colored = if include_terminal_colors {
                        InMemoryNode::literal_colored(&node, &literal)
                    } else {
                        literal.into()
                    };
                    NodeSeek::Continue(literal_colored)
                }
            },
        );

        // 4. Combine it all together!
        let in_between = in_between_node_literals
            .expect("generate_literal seek_forwards_until failed")
            .fold::<ColoredString, _>("".into(), |acc, colored_str| {
                format!("{}{}", acc, colored_str).into()
            });
        format!("{earlier_suffix}{in_between}{later_prefix}").into()
    }

    /// When called, deletes the character span referred to by the selection.
    fn splice(
        self: &Self,
        new_literal: Option<String>,
        perform_reparse: bool,
    ) -> Result<(), String> {
        // Find the earlier and later pointers out of self.primary and self.secondary
        let earlier_cursor = &{
            // NOTE: advance earlier_cursor forward, skipping empty nodes at the start of the selection
            //
            // This ensures that because there's always an empty node at the top of the token tree,
            // that the full tree won't be deleted.
            let mut earlier_cursor = if self.primary.node < self.secondary.node {
                self.primary.clone()
            } else {
                self.secondary.clone()
            };
            while earlier_cursor.offset == 0
                && InMemoryNode::literal(&earlier_cursor.node).is_empty()
            {
                let Some(next) = earlier_cursor
                    .node
                    .borrow()
                    .next
                    .as_ref()
                    .map(|n| n.upgrade())
                    .flatten()
                else {
                    break;
                };
                earlier_cursor = Cursor::new(next);
            }
            earlier_cursor
        };
        let later_cursor = if self.primary.node < self.secondary.node {
            &self.secondary
        } else {
            &self.primary
        };

        // println!("earlier={:?} later={:?}", earlier_cursor.node.borrow().metadata, later_cursor.node.borrow().metadata);

        // If the node selection spans within a single node, then to delete that data, just update
        // the string literal value on the node
        if earlier_cursor.node == later_cursor.node {
            if earlier_cursor.offset == later_cursor.offset {
                // A zero length selection - do nothing!
                return Ok(());
            };

            let new_literal_start_offset = if earlier_cursor.offset < later_cursor.offset {
                earlier_cursor.offset
            } else {
                later_cursor.offset
            };

            // Construct a string, taking all the characters before the selection and the
            // characters after the selection, and sticking them together (omitting the selection
            // chars)
            let new_literal_length = later_cursor.offset.abs_diff(earlier_cursor.offset);
            let new_literal_prefix =
                InMemoryNode::literal_substring(&earlier_cursor.node, 0, new_literal_start_offset);
            let new_literal_suffix = InMemoryNode::literal_substring(
                &earlier_cursor.node,
                new_literal_start_offset + new_literal_length,
                InMemoryNode::literal(&earlier_cursor.node).len() - new_literal_start_offset,
            );
            let new_literal = format!(
                "{new_literal_prefix}{}{new_literal_suffix}",
                if let Some(new_literal) = new_literal {
                    new_literal
                } else {
                    "".into()
                },
            );

            // NOTE: should all nodes under the parent be combined and reparsed if
            // new_literal.len() == 0?
            InMemoryNode::set_literal(&earlier_cursor.node, &new_literal);

            return Ok(());
        };

        // If the node selection spans multiple nodes, then:
        //
        // 1. Find the earlier node (done above), and store the first part which should be kept
        let literal_prefix_to_keep =
            InMemoryNode::literal_substring(&earlier_cursor.node, 0, earlier_cursor.offset);

        // 2. Store the last part of the later node which should be kept
        let later_cursor_substring_outside_selection = InMemoryNode::literal_substring(
            &later_cursor.node,
            later_cursor.offset,
            InMemoryNode::literal(&later_cursor.node).len() - later_cursor.offset,
        );

        let earlier_node_depth = InMemoryNode::depth(&earlier_cursor.node);

        // 3. Delete all nodes starting at after the earlier node up to and including the later node
        let mut reached_later_cursor_node = false;
        let resulting_literal_vectors = InMemoryNode::remove_nodes_sequentially_until(
            &earlier_cursor.node,
            Inclusivity::Exclusive,
            |node, _ct| {
                // 3. Delete all nodes starting at after the earlier node up to and including the later node
                if !reached_later_cursor_node && node == &later_cursor.node {
                    reached_later_cursor_node = true;
                }
                if !reached_later_cursor_node {
                    // println!("DELETE: {} {:?}", InMemoryNode::depth(node), node.borrow().metadata);
                    return NodeSeek::Continue(None);
                }

                if node == &later_cursor.node {
                    // The node that was found was `later_cursor.node`, so use the part of the
                    // later node that is outside the selection.
                    //
                    // This is where the loop transitions from "deleting stuff in the selection" to
                    // "collecting stuff after the selection into a literal"
                    return NodeSeek::Continue(Some(
                        later_cursor_substring_outside_selection.clone(),
                    ));
                };

                // 4. Keep going, storing literal text until back up at the same depth level as the
                //    earlier node. Swap the earlier node with a new node containing literal text of all
                //    the accumulated text.
                let literal = InMemoryNode::literal(node);

                let depth = InMemoryNode::depth(node);
                // println!("NODE: {} {:?}", depth, node.borrow().metadata);
                if depth > earlier_node_depth {
                    // The node that was found was below `earlier_cursor.node` in the hierarchy, so
                    // keep going
                    NodeSeek::Continue(Some(literal))
                } else {
                    // The node was at or above `earlier_cursor.node`, so bail out
                    NodeSeek::Done(Some(literal))
                }
            },
        )?;

        let collected = resulting_literal_vectors
            .filter_map(|n| n)
            .collect::<String>();
        // println!("RESULT: {:?} {:?} {:?}", literal_prefix_to_keep, collected, later_cursor_substring_outside_selection);
        let resulting_literal = format!(
            "{literal_prefix_to_keep}{}{collected}",
            if let Some(new_literal) = new_literal {
                new_literal
            } else {
                "".into()
            },
        );

        // Swap the earlier node with a new node containing literal text of all
        // the accumulated text.
        InMemoryNode::set_literal(&earlier_cursor.node, &resulting_literal);
        InMemoryNode::remove_all_children(&earlier_cursor.node);

        // 5. Reparse the newly created literal text node
        // NOTE: consider making this an async job that can run when free cycles are available
        if perform_reparse {
            let child = earlier_cursor.node.borrow();
            if let (Some(Some(parent)), Some(child_index)) = (
                child.parent.as_ref().map(|n| n.upgrade()),
                child.child_index,
            ) {
                InMemoryNode::reparse_child_at_index(parent, child_index)?;
            } else {
                // The node that needs to be reparsed doesn't have a parent!
                //
                // This should be impossible, since the ROOT node at the top of the document has no
                // length, and should therefore never be part of a selection
                unreachable!("Selection::delete: tried to reparse a node that has no parent ({:?}), this is impossible!", child.metadata);
            }
        }

        Ok(())
    }

    /// When called, deletes the character span referred to by the selection, and reparses the
    /// result
    pub fn delete(&self) -> Result<(), String> {
        self.splice(None, true)
    }
    /// When called, deletes the character span referred to by the selection. NO REPARSE OCCURS.
    pub fn delete_raw(&self) -> Result<(), String> {
        self.splice(None, false)
    }

    /// When called, replaces the character span referred to by the selection with the given
    /// literal, and reparses the result
    pub fn replace(&self, literal: &str) -> Result<(), String> {
        self.splice(Some(literal.into()), true)
    }
    /// When called, replaces the character span referred to by the selection with the given
    /// literal. NO REPARSE OCCURS.
    pub fn replace_raw(&self, literal: &str) -> Result<(), String> {
        self.splice(Some(literal.into()), false)
    }
}

#[cfg(test)]
mod tests {
    use super::Selection;
    use crate::{
        languages,
        node_tree::{
            cursor::{Cursor, CursorSeek},
            node::InMemoryNode,
            utils::{Direction, Inclusivity},
        },
    };

    const LOREM_IPSUM: &'static str = concat!(
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum dignissim ",
        "augue ac arcu convallis, a eleifend mauris blandit. Pellentesque molestie erat ",
        "ex, et scelerisque magna ultrices a. Morbi porta mauris a nisl cursus luctus. ",
        "Suspendisse dapibus accumsan dui, quis bibendum eros facilisis a. Orci varius ",
        "natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. ",
        "Suspendisse suscipit rutrum lobortis. Suspendisse suscipit ultrices gravida. Aenean",
    );

    #[test]
    fn seek_forward_back_by_char() {
        let root = InMemoryNode::<languages::raw::SyntaxKind>::new_from_parsed(LOREM_IPSUM);
        // InMemoryNode::dump(&root);

        let mut selection = Selection::new(root);
        assert_eq!(selection.primary.to_rows_cols(), (1, 1));
        assert_eq!(selection.secondary.to_rows_cols(), (1, 1));

        // Forwards one char
        let mut selection = selection.set_primary(
            selection
                .primary
                .seek_forwards(CursorSeek::AdvanceByCharCount(1)),
        );
        assert_eq!(selection.primary.to_rows_cols(), (1, 2));
        assert_eq!(selection.secondary.to_rows_cols(), (1, 1));
        assert_eq!(selection.literal(), "L");

        // Forwards 10 chars
        let mut selection = selection.set_primary(
            selection
                .primary
                .seek_forwards(CursorSeek::AdvanceByCharCount(10)),
        );
        assert_eq!(selection.primary.to_rows_cols(), (1, 12));
        assert_eq!(selection.secondary.to_rows_cols(), (1, 1));
        assert_eq!(selection.literal(), "Lorem ipsum");

        // Backwards 5 chars
        let mut selection = selection.set_primary(
            selection
                .primary
                .seek_backwards(CursorSeek::AdvanceByCharCount(5)),
        );
        assert_eq!(selection.primary.to_rows_cols(), (1, 7));
        assert_eq!(selection.secondary.to_rows_cols(), (1, 1));
        assert_eq!(selection.literal(), "Lorem ");

        // Back to start
        let mut selection = selection.set_primary(
            selection
                .primary
                .seek_backwards(CursorSeek::advance_until_start_end()),
        );
        assert_eq!(selection.primary.to_rows_cols(), (1, 1));
        assert_eq!(selection.secondary.to_rows_cols(), (1, 1));
        assert_eq!(selection.literal(), "");
    }

    #[test]
    fn seek_forward_by_lower_word() {
        let root = InMemoryNode::<languages::raw::SyntaxKind>::new_from_parsed(LOREM_IPSUM);
        // InMemoryNode::dump(&root);

        let mut selection = Selection::new(root);
        assert_eq!(selection.primary.to_rows_cols(), (1, 1));
        assert_eq!(selection.secondary.to_rows_cols(), (1, 1));

        // Forwards one word INCLUSIVE
        let mut selection = selection.perform_inclusive(move |primary, secondary| {
            let primary = primary.seek_forwards(CursorSeek::forwards_word(1, false, false));
            (primary, secondary)
        });
        assert_eq!(selection.primary.to_rows_cols(), (1, 7));
        assert_eq!(selection.secondary.to_rows_cols(), (1, 1));
        assert_eq!(selection.literal(), "Lorem ");

        // Forwards one word EXCLUSIVE
        let mut selection = selection.perform_exclusive(move |primary, secondary| {
            let primary = primary.seek_forwards(CursorSeek::forwards_word(1, false, false));
            (primary, secondary)
        });
        assert_eq!(selection.primary.to_rows_cols(), (1, 14));
        assert_eq!(selection.secondary.to_rows_cols(), (1, 1));
        assert_eq!(selection.literal(), "Lorem ipsum d");

        // Forwards three words INCLUSIVE
        println!("------");
        let mut selection = selection.perform_inclusive(move |primary, secondary| {
            let primary = primary
                .seek_forwards(CursorSeek::forwards_word(1, false, false))
                .seek_forwards(CursorSeek::forwards_word(1, false, false))
                .seek_forwards(CursorSeek::forwards_word(1, false, false));
            (primary, secondary)
        });
        assert_eq!(selection.primary.to_rows_cols(), (1, 29));
        assert_eq!(selection.secondary.to_rows_cols(), (1, 1));
        assert_eq!(selection.literal(), "Lorem ipsum dolor sit amet, ");

        // Forwards two words EXCLUSIVE
        println!("------");
        let mut selection = selection.perform_exclusive(move |primary, secondary| {
            let primary = primary
                .seek_forwards(CursorSeek::forwards_word(1, false, false))
                .seek_forwards(CursorSeek::forwards_word(1, false, false));
            (primary, secondary)
        });
        assert_eq!(selection.primary.to_rows_cols(), (1, 53));
        assert_eq!(selection.secondary.to_rows_cols(), (1, 1));
        assert_eq!(
            selection.literal(),
            "Lorem ipsum dolor sit amet, consectetur adipiscing e"
        );
    }

    #[test]
    fn seek_backward_by_lower_word() {
        let root = InMemoryNode::<languages::raw::SyntaxKind>::new_from_parsed(LOREM_IPSUM);
        // InMemoryNode::dump(&root);

        // Start by seeking to a middle point in the input 52 chars in:
        //
        // > Lorem ipsum dolor sit amet, consectetur adipiscing elit
        //                                                       ^ (right here!)
        let mut selection = Selection::new(root);
        let mut selection = selection.perform_inclusive(move |primary, secondary| {
            let primary = primary.seek_forwards(CursorSeek::AdvanceByCharCount(52));
            let secondary = secondary.seek_forwards(CursorSeek::AdvanceByCharCount(52));
            (primary, secondary)
        });
        assert_eq!(selection.primary.to_rows_cols(), (1, 53));

        println!("------");
        // Backwards one word INCLUSIVE
        let mut selection = selection.perform_inclusive(move |primary, secondary| {
            let primary = primary.seek_backwards(CursorSeek::back_word(1, false, false));
            (primary, secondary)
        });
        assert_eq!(selection.literal(), "adipiscing e");
        assert_eq!(selection.primary.to_rows_cols(), (1, 41));
        assert_eq!(selection.secondary.to_rows_cols(), (1, 53));

        // Backwards one word EXCLUSIVE
        let mut selection = selection.perform_exclusive(move |primary, secondary| {
            let primary = primary.seek_backwards(CursorSeek::back_word(1, false, false));
            (primary, secondary)
        });
        assert_eq!(selection.primary.to_rows_cols(), (1, 29));
        assert_eq!(selection.secondary.to_rows_cols(), (1, 54));
        assert_eq!(selection.literal(), "consectetur adipiscing el");
    }
}
