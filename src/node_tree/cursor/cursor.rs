use colored::{ColoredString, Colorize};

use crate::node_tree::{
    cursor::CursorSeek,
    node::{InMemoryNode, NodeSeek, TokenKindTrait, NodeMetadata},
    utils::{Direction, Inclusivity, NEWLINE},
};
use std::{cell::RefCell, rc::Rc, fmt::Debug, collections::VecDeque};

/// A cursor represents a position in a node tree - ie, a node and an offset in characters from the
/// start of that node. A cursor can be seeked forwards and backwards through the node tree to get
/// its contents or to perform operations on the node tree.
#[derive(Clone)]
pub struct Cursor<TokenKind: TokenKindTrait> {
    node: Rc<RefCell<InMemoryNode<TokenKind>>>,
    offset: usize,
}

impl<TokenKind: TokenKindTrait> Debug for Cursor<TokenKind> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Cursor")
         .field(&self.node.borrow().metadata)
         .field(&self.node.borrow().index)
         .field(&self.offset)
         .finish()
    }
}

impl<TokenKind: TokenKindTrait> Cursor<TokenKind> {
    pub fn new(node: Rc<RefCell<InMemoryNode<TokenKind>>>) -> Self {
        Self::new_at(node, 0)
    }
    pub fn new_at(node: Rc<RefCell<InMemoryNode<TokenKind>>>, offset: usize) -> Self {
        Self { node, offset }
    }
    pub fn new_at_rows_cols(root: Rc<RefCell<InMemoryNode<TokenKind>>>, rows_cols: (usize, usize)) -> Self {
        let (rows, cols) = rows_cols;
        let mut row_counter = 1;
        let mut col_counter = 1;

        let (cursor, _) = Self::new(root).seek_forwards_until(|c, _i| {
            if row_counter == rows {
                // Before reaching the first newline, count the col chars
                if col_counter < cols {
                    col_counter += 1;
                    CursorSeek::Continue
                } else {
                    CursorSeek::Done
                }
            } else if c == *NEWLINE {
                // From that point on count each newline
                row_counter += 1;
                CursorSeek::Continue
            } else {
                CursorSeek::Continue
            }
        });

        cursor
    }

    /// When called, create a new Selection out of this cursor.
    ///
    /// A Selection is a "double ended" cursor that can be used to define text ranges to perform
    /// operations on.
    pub fn selection(self: &Self) -> Selection<TokenKind> {
        Selection::new_from_cursor(self.clone())
    }

    pub fn to_rows_cols(self: &Self) -> (usize, usize) {
        let mut row_counter = 1;
        let mut col_counter = 1;

        let _ = self.seek_backwards_until(|c, _i| {
            if row_counter == 0 {
                // Before reaching the first newline, count the col chars
                col_counter += 1;
            } else if c == *NEWLINE {
                // From that point on count each newline
                row_counter += 1;
            };
            CursorSeek::Continue
        });

        (row_counter, col_counter)
    }

    /// When called, seeks starting at the cursor position character by character through the node
    /// structure in the giren `direction` until the given `until_fn` returns either `Stop` or `Done`.
    pub fn seek_until<UntilFn>(self: &Self, direction: Direction, mut until_fn: UntilFn) -> (Self, String)
    where
        UntilFn: FnMut(char, usize) -> CursorSeek,
    {
        let mut global_char_counter = 0; // Store a global count of characters processed

        // The final node and offset values:
        let mut new_offset = self.offset;
        let mut new_node = self.node.clone();

        // To handle CursorSeek::AdvanceByCharCount(n), keep a counter of characters to skip:
        let mut cached_char_until_count = 0;

        // To handle CursorSeek::AdvanceByLineCount(n), keep a counter of lines to skip:
        let mut cached_line_until_count = 0;
        let mut cached_line_reached_newline = false;
        let mut cached_line_current_cols: Option<usize> = None;

        // To handle CursorSeek::AdvanceUntil(...), keep a stack of `until_fn`s and their
        // corresponding counts - these should always have the same length:
        let mut advance_until_fn_stack: Vec<Rc<RefCell<dyn FnMut(char, usize) -> CursorSeek>>> =
            vec![];
        let mut advance_until_char_counter_stack: Vec<usize> = vec![];

        let resulting_char_vectors = InMemoryNode::seek_until(&self.node, direction, Inclusivity::Inclusive, |node, ct| {
            new_node = node.clone();
            let mut result = vec![];

            let node_literal = InMemoryNode::literal(node);
            let mut characters = if ct == 0 {
                // If this is the first node, skip forward / backward `self.offset` times.
                match direction {
                    Direction::Forwards => {
                        // Seek from the start to the offset
                        node_literal.chars().skip(self.offset).collect::<VecDeque<char>>()
                    },
                    Direction::Backwards => {
                        // Seek from the end to the offset from the start
                        let mut iterator = node_literal.chars();
                        for _ in 0..(node_literal.len()-self.offset) {
                            iterator.next_back();
                        };
                        iterator.collect::<VecDeque<char>>()
                    },
                }
            } else {
                // If this is not the first node, then make sure to reset the offset to either the
                // start or end of the node so that increments / decrements later are operating on
                // the right value.
                new_offset = match direction {
                    Direction::Forwards => 0,
                    Direction::Backwards => node_literal.len(),
                };

                node_literal.chars().collect::<VecDeque<char>>()
            };

            // Iterate over all characters within the node, one by one, until a match occurs:
            while let Some(character) = match direction {
                Direction::Forwards => characters.pop_front(),
                Direction::Backwards => characters.pop_back(),
            } {
                // println!("INITIAL NEW_OFFSET: {new_offset} ({global_char_counter}, {character})");
                // If there's a char_until_count, then run until that exhausts iself
                if cached_char_until_count > 0 {
                    cached_char_until_count -= 1;

                    if cached_char_until_count > 0 {
                        result.push(character);
                        global_char_counter += 1;
                        new_offset = match direction {
                            Direction::Forwards => new_offset + 1,
                            Direction::Backwards => new_offset - 1,
                        };
                        continue;
                    }
                }

                // If there's a line_until_count, then run until that exhausts iself
                if cached_line_until_count > 0 {
                    dbg!(cached_line_until_count);
                    dbg!(cached_line_reached_newline);

                    let cols = if let Some(cols) = cached_line_current_cols {
                        cols
                    } else {
                        println!("--- LINE START! ---");
                        // 1. Figure out how many characters are before the current cursor in the
                        //    line
                        let mut current_cols = self.offset + 1;
                        let _ = InMemoryNode::seek_until(&self.node, Direction::Backwards, Inclusivity::Exclusive, |inner_node, _ct| {
                            for inner_character in InMemoryNode::literal(inner_node).chars().rev() {
                                if inner_character == *NEWLINE {
                                    return NodeSeek::Stop;
                                }
                                current_cols += 1;
                            }
                            return NodeSeek::Continue(());
                        });
                        cached_line_current_cols = Some(current_cols+1);

                        current_cols
                    };
                    dbg!(cols);

                    if cached_line_reached_newline {
                        let mut cols_copy = cols;
                        if cols_copy > 0 {
                            // 4. Advance cached_line_current_cols (in this context, cols) characters 
                            //    to get to the next line
                            cols_copy -= 1;
                            cached_line_current_cols = Some(cols_copy);
                        } else {
                            // 3. A newline has been reached going the seek direction, so after
                            //    this, advance by the number of cols before the cursor to get back
                            //    to the same place
                            cols_copy = cols + 1;
                            cached_line_current_cols = Some(cols_copy);
                        }

                        if cols_copy > 0 {
                            result.push(character);
                            global_char_counter += 1;
                            new_offset = match direction {
                                Direction::Forwards => new_offset + 1,
                                Direction::Backwards => new_offset - 1,
                            };
                            continue;
                        }

                        cached_line_until_count -= 1;
                        cached_line_reached_newline = false;
                        cached_line_current_cols = None;
                        println!("--- LINE DONE! --- cached_line_until_count={}", cached_line_until_count);

                        if cached_line_until_count > 0 {
                            continue;
                        }
                    } else {
                        result.push(character);
                        global_char_counter += 1;
                        new_offset = match direction {
                            Direction::Forwards => new_offset + 1,
                            Direction::Backwards => new_offset - 1,
                        };

                        // 2. If the first newline hasn't been reached, then keep going until it is
                        // reached
                        if character == *NEWLINE {
                            cached_line_reached_newline = true;
                        };
                        continue;
                    }
                }

                // If there's a char_until_fn, then run until that passes
                if let (Some(advance_until_fn), Some(advance_until_char_counter)) = (
                    &advance_until_fn_stack.last(),
                    advance_until_char_counter_stack.last(),
                ) {
                    let value = {
                        let mut until_fn_borrowed_mut = advance_until_fn.borrow_mut();
                        until_fn_borrowed_mut(character, *advance_until_char_counter)
                    };

                    match value {
                        CursorSeek::Continue => {
                            result.push(character);
                            global_char_counter += 1;
                            new_offset = match direction {
                                Direction::Forwards => new_offset + 1,
                                Direction::Backwards => new_offset - 1,
                            };
                            *advance_until_char_counter_stack.last_mut().unwrap() += 1;
                            continue;
                        }
                        CursorSeek::AdvanceByLines(n) => {
                            cached_line_until_count += n;

                            // NOTE: re-add the character back to the characters vec, so that it
                            // can be skipped with the AdvanceByCharCount skip code
                            match direction {
                                Direction::Forwards => characters.push_front(character),
                                Direction::Backwards => characters.push_back(character),
                            };
                            continue;
                        }
                        CursorSeek::AdvanceByCharCount(n) => {
                            cached_char_until_count += n+1;

                            // NOTE: re-add the character back to the characters vec, so that it
                            // can be skipped with the AdvanceByCharCount skip code
                            match direction {
                                Direction::Forwards => characters.push_front(character),
                                Direction::Backwards => characters.push_back(character),
                            };
                            continue;
                        }
                        CursorSeek::AdvanceUntil {
                            until_fn: char_until_fn,
                        } => {
                            advance_until_fn_stack.push(char_until_fn);
                            advance_until_char_counter_stack.push(0);

                            // NOTE: re-add the character back to the characters vec, so that it
                            // can be skipped with the AdvanceByCharCount skip code
                            match direction {
                                Direction::Forwards => characters.push_front(character),
                                Direction::Backwards => characters.push_back(character),
                            };
                            continue;
                        }
                        CursorSeek::Stop => {
                            advance_until_fn_stack.pop();
                            advance_until_char_counter_stack.pop();
                        }
                        CursorSeek::Done => {
                            result.push(character);
                            global_char_counter += 1;
                            new_offset = match direction {
                                Direction::Forwards => new_offset + 1,
                                Direction::Backwards => new_offset - 1,
                            };

                            advance_until_fn_stack.pop();
                            advance_until_char_counter_stack.pop();
                        }
                    }
                    if !advance_until_fn_stack.is_empty()
                        || !advance_until_char_counter_stack.is_empty()
                    {
                        continue;
                    }
                }

                match until_fn(character, global_char_counter) {
                    CursorSeek::Continue => {
                        result.push(character);
                        global_char_counter += 1;
                        new_offset = match direction {
                            Direction::Forwards => new_offset + 1,
                            Direction::Backwards => new_offset - 1,
                        };
                        continue;
                    }
                    CursorSeek::AdvanceByCharCount(n) => {
                        cached_char_until_count += n+1;

                        // NOTE: re-add the character back to the characters vec, so that it
                        // can be skipped with the AdvanceByCharCount skip code
                        match direction {
                            Direction::Forwards => characters.push_front(character),
                            Direction::Backwards => characters.push_back(character),
                        };
                        continue;
                    }
                    CursorSeek::AdvanceByLines(n) => {
                        cached_line_until_count += n;

                        // NOTE: re-add the character back to the characters vec, so that it
                        // can be skipped with the AdvanceByCharCount skip code
                        match direction {
                            Direction::Forwards => characters.push_front(character),
                            Direction::Backwards => characters.push_back(character),
                        };
                        continue;
                    }
                    CursorSeek::AdvanceUntil {
                        until_fn: char_until_fn,
                    } => {
                        advance_until_fn_stack.push(char_until_fn);
                        advance_until_char_counter_stack.push(0);

                        // NOTE: re-add the character back to the characters vec, so that it
                        // can be skipped with the AdvanceByCharCount skip code
                        match direction {
                            Direction::Forwards => characters.push_front(character),
                            Direction::Backwards => characters.push_back(character),
                        };
                        continue;
                    }
                    CursorSeek::Stop => {
                        return NodeSeek::Done(result);
                    }
                    CursorSeek::Done => {
                        result.push(character);
                        global_char_counter += 1;
                        new_offset = match direction {
                            Direction::Forwards => new_offset + 1,
                            Direction::Backwards => new_offset - 1,
                        };
                        return NodeSeek::Done(result);
                    }
                }
            }

            NodeSeek::Continue(result)
        });

        // Once all the seeks have been performed, take the vectors of caracters seeked through
        // from each node and flatten them all together into a string.
        let resulting_chars = resulting_char_vectors.flat_map(|vector| vector.into_iter());
        let output_string = match direction {
            Direction::Forwards => resulting_chars.collect::<String>(),
            Direction::Backwards => resulting_chars.rev().collect::<String>(),
        };

        (Self::new_at(new_node, new_offset), output_string)
    }

    /// When called, seeks forward starting at the cursor position character by character through
    /// the node structure until the given `until_fn` returns either `Stop` or `Done`.
    pub fn seek_forwards_until<UntilFn>(self: &Self, until_fn: UntilFn) -> (Self, String)
    where
        UntilFn: FnMut(char, usize) -> CursorSeek,
    {
        self.seek_until(Direction::Forwards, until_fn)
    }

    /// When called, performs the given `seek` operation once, causing the cursor to seek forwards
    /// by the given amount
    pub fn seek_forwards(self: &Self, seek: CursorSeek) -> (Self, String) {
        self.seek_forwards_until(|_character, index| {
            if index == 0 {
                seek.clone()
            } else {
                CursorSeek::Stop
            }
        })
    }

    /// When called, seeks backward starting at the cursor position character by character through
    /// the node structure until the given `until_fn` returns either `Stop` or `Done`.
    pub fn seek_backwards_until<UntilFn>(self: &Self, until_fn: UntilFn) -> (Self, String)
    where
        UntilFn: FnMut(char, usize) -> CursorSeek,
    {
        self.seek_until(Direction::Backwards, until_fn)
    }

    /// When called, performs the given `seek` operation once, causing the cursor to seek backwards
    /// by the given amount
    pub fn seek_backwards(self: &Self, seek: CursorSeek) -> (Self, String) {
        self.seek_backwards_until(|_character, index| {
            if index == 0 {
                seek.clone()
            } else {
                CursorSeek::Stop
            }
        })
    }
}





#[derive(Clone)]
pub struct Selection<TokenKind: TokenKindTrait> {
    pub primary: Cursor<TokenKind>,
    pub secondary: Cursor<TokenKind>,
}

impl<TokenKind: TokenKindTrait> Debug for Selection<TokenKind> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let literal_colors = self.literal_colors();
        let literal_length = self.literal().len();
        write!(f, "Selection(literal=\"{}\", len={}, primary={:?} secondary={:?})", literal_colors, literal_length, self.primary, self.secondary)
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
        Self { secondary: cursor.clone(), primary: cursor }
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

    pub fn set_primary(self: &mut Self, input: (Cursor<TokenKind>, String)) -> &mut Self {
        self.primary = input.0;
        self
    }
    pub fn set_secondary(self: &mut Self, input: (Cursor<TokenKind>, String)) -> &mut Self {
        self.secondary = input.0;
        self
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
        let earlier_cursor = if self.primary.node < self.secondary.node { &self.primary } else { &self.secondary };
        let later_cursor = if self.primary.node < self.secondary.node { &self.secondary } else { &self.primary };
        let earlier_suffix = InMemoryNode::literal_substring(
            &earlier_cursor.node,
            earlier_cursor.offset,
            InMemoryNode::literal(&earlier_cursor.node).len() - earlier_cursor.offset,
        );

        // 2. Store the first part of the later node which should be kept
        let later_prefix = InMemoryNode::literal_substring(
            &later_cursor.node,
            0,
            later_cursor.offset,
        );

        // 3. Accumulate the text in the in between nodes
        let in_between_node_literals = InMemoryNode::seek_forwards_until(&earlier_cursor.node, Inclusivity::Exclusive, |node, ct| {
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
        });

        // 4. Combine it all together!
        let in_between = in_between_node_literals.fold::<ColoredString, _>("".into(), |acc, colored_str| {
            format!("{}{}", acc, colored_str).into()
        });
        format!("{earlier_suffix}{in_between}{later_prefix}").into()
    }

    /// When called, deletes the character span referred to by the selection.
    fn splice(self: &Self, new_literal: Option<String>, perform_reparse: bool) -> Result<(), String> {
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
            while earlier_cursor.offset == 0 && InMemoryNode::literal(&earlier_cursor.node).is_empty() {
                let Some(next) = earlier_cursor.node.borrow().next.as_ref().map(|n| n.upgrade()).flatten() else {
                    break;
                };
                earlier_cursor = Cursor::new(next);
            }
            earlier_cursor
        };
        let later_cursor = if self.primary.node < self.secondary.node { &self.secondary } else { &self.primary };

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
            let new_literal_prefix = InMemoryNode::literal_substring(
                &earlier_cursor.node,
                0,
                new_literal_start_offset,
            );
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
        let literal_prefix_to_keep = InMemoryNode::literal_substring(&earlier_cursor.node, 0, earlier_cursor.offset);

        // 2. Store the last part of the later node which should be kept
        let later_cursor_substring_outside_selection = InMemoryNode::literal_substring(
            &later_cursor.node,
            later_cursor.offset,
            InMemoryNode::literal(&later_cursor.node).len() - later_cursor.offset,
        );

        let earlier_node_depth = InMemoryNode::depth(&earlier_cursor.node);

        // 3. Delete all nodes starting at after the earlier node up to and including the later node
        let mut reached_later_cursor_node = false;
        let resulting_literal_vectors = InMemoryNode::remove_nodes_sequentially_until(&earlier_cursor.node, Inclusivity::Exclusive, |node, _ct| {
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
                return NodeSeek::Continue(Some(later_cursor_substring_outside_selection.clone()));
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
        });

        let collected = resulting_literal_vectors.filter_map(|n| n).collect::<String>();
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
            if let (
                Some(Some(parent)),
                Some(child_index),
            ) = (child.parent.as_ref().map(|n| n.upgrade()), child.child_index) {
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
