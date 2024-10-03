use crate::node_tree::{
    cursor::CursorSeek,
    node::{InMemoryNode, NodeSeek},
    utils::{Direction, Inclusivity},
};
use std::{cell::RefCell, rc::Rc, fmt::Debug};

/// A cursor represents a position in a node tree - ie, a node and an offset in characters from the
/// start of that node. A cursor can be seeked forwards and backwards through the node tree to get
/// its contents or to perform operations on the node tree.
#[derive(Clone)]
pub struct Cursor {
    node: Rc<RefCell<InMemoryNode>>,
    offset: usize,
}

impl Debug for Cursor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Cursor")
         .field(&self.node.borrow().metadata)
         .field(&self.offset)
         .finish()
    }
}

impl Cursor {
    pub fn new(node: Rc<RefCell<InMemoryNode>>) -> Self {
        Self::new_at(node, 0)
    }
    pub fn new_at(node: Rc<RefCell<InMemoryNode>>, offset: usize) -> Self {
        Self { node, offset }
    }

    /// When called, create a new Selection out of this cursor.
    ///
    /// A Selection is a "double ended" cursor that can be used to define text ranges to perform
    /// operations on.
    pub fn selection(self: &Self) -> Selection {
        Selection::new_from_cursor(self.clone())
    }

    /// When called, seeks starting at the cursor position character by character through the node
    /// structure in the giren `direction` until the given `until_fn` returns either `Stop` or `Done`.
    pub fn seek_until<UntilFn>(self: &Self, direction: Direction, until_fn: UntilFn) -> (Self, String)
    where
        UntilFn: Fn(char, usize) -> CursorSeek,
    {
        let mut global_char_counter = 0; // Store a global count of characters processed

        // The final node and offset values:
        let mut new_offset = self.offset;
        let mut new_node = self.node.clone();

        // To handle CursorSeek::AdvanceByCharCount(n), keep a counter of characters to skip:
        let mut cached_char_until_count = 0;

        // To handle CursorSeek::AdvanceUntil(...), keep a stack of `until_fn`s and their
        // corresponding counts - these should always have the same length:
        let mut advance_until_fn_stack: Vec<Rc<RefCell<dyn FnMut(char, usize) -> CursorSeek>>> =
            vec![];
        let mut advance_until_char_counter_stack: Vec<usize> = vec![];

        let resulting_char_vectors = InMemoryNode::seek_until(&self.node, direction, Inclusivity::Inclusive, |node, ct| {
            new_node = node.clone();
            let mut result = vec![];

            let node_literal = InMemoryNode::literal(node);
            let mut iterator = node_literal.chars();
            if ct == 0 {
                // If this is the first node, skip forward / backward `self.offset` times.
                match direction {
                    Direction::Forwards => {
                        // Seek from the start to the offset
                        for _ in 0..self.offset {
                            iterator.next();
                        };
                    },
                    Direction::Backwards => {
                        // Seek from the end to the offset from the start
                        for _ in 0..(node_literal.len()-self.offset) {
                            iterator.next_back();
                        };
                    },
                };
            } else {
                // If this is not the first node, then make sure to reset the offset to either the
                // start or end of the node so that increments / decrements later are operating on
                // the right value.
                new_offset = match direction {
                    Direction::Forwards => 0,
                    Direction::Backwards => node_literal.len(),
                };
            };

            // Iterate over all characters within the node, one by one, until a match occurs:
            while let Some(character) = match direction {
                Direction::Forwards => iterator.next(),
                Direction::Backwards => iterator.next_back(),
            } {
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
                        CursorSeek::AdvanceByCharCount(n) => {
                            result.push(character);
                            global_char_counter += 1;
                            new_offset = match direction {
                                Direction::Forwards => new_offset + 1,
                                Direction::Backwards => new_offset - 1,
                            };

                            // NOTE: n-1 to take into account the implicit "Continue" on the first
                            // character
                            cached_char_until_count += n-1;
                            continue;
                        }
                        CursorSeek::AdvanceUntil {
                            until_fn: char_until_fn,
                        } => {
                            result.push(character);
                            global_char_counter += 1;
                            new_offset = match direction {
                                Direction::Forwards => new_offset + 1,
                                Direction::Backwards => new_offset - 1,
                            };

                            advance_until_fn_stack.push(char_until_fn);
                            // NOTE: 1 to take into account the implicit "Continue" on the first
                            // character
                            advance_until_char_counter_stack.push(1);
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
                        result.push(character);
                        global_char_counter += 1;
                        new_offset = match direction {
                            Direction::Forwards => new_offset + 1,
                            Direction::Backwards => new_offset - 1,
                        };

                        // NOTE: n-1 to take into account the implicit "Continue" on the first
                        // character
                        cached_char_until_count += n-1;
                        continue;
                    }
                    CursorSeek::AdvanceUntil {
                        until_fn: char_until_fn,
                    } => {
                        result.push(character);
                        global_char_counter += 1;
                        new_offset = match direction {
                            Direction::Forwards => new_offset + 1,
                            Direction::Backwards => new_offset - 1,
                        };

                        advance_until_fn_stack.push(char_until_fn);
                        // NOTE: 1 to take into account the implicit "Continue" on the first
                        // character
                        advance_until_char_counter_stack.push(1);
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
        UntilFn: Fn(char, usize) -> CursorSeek,
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
        UntilFn: Fn(char, usize) -> CursorSeek,
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
pub struct Selection {
    pub primary: Cursor,
    pub secondary: Cursor,
}

impl Debug for Selection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Selection({:?}, primary={:?} secondary={:?})", self.literal(), self.primary, self.secondary)
    }
}

impl Selection {
    pub fn new(node: Rc<RefCell<InMemoryNode>>) -> Self {
        Self::new_at(node, 0)
    }
    pub fn new_at(node: Rc<RefCell<InMemoryNode>>, offset: usize) -> Self {
        let cursor = Cursor::new_at(node, offset);
        Self::new_from_cursor(cursor)
    }
    pub fn new_from_cursor(cursor: Cursor) -> Self {
        Self { secondary: cursor.clone(), primary: cursor }
    }

    pub fn set_primary(self: &mut Self, input: (Cursor, String)) {
        self.secondary = input.0;
    }
    pub fn set_secondary(self: &mut Self, input: (Cursor, String)) {
        self.secondary = input.0;
    }

    /// When called, computes the underlying literal text that the selection has covered.
    pub fn literal(self: &Self) -> String {
        // If the node selection spans within a single node, then take a substring of the common
        // literal value based on the offsets.
        if self.primary.node == self.secondary.node {
            let literal_start_offset = if self.primary.offset < self.secondary.offset {
                self.primary.offset
            } else {
                self.secondary.offset
            };
            let literal_length = self.secondary.offset.abs_diff(self.primary.offset);
            return InMemoryNode::literal_substring(
                &self.primary.node,
                literal_start_offset,
                literal_length,
            );
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
                NodeSeek::Continue(literal)
            }
        });

        // 4. Combine it all together!
        format!("{earlier_suffix}{}{later_prefix}", in_between_node_literals.collect::<String>())
    }

    /// When called, deletes the character span referred to by the selection.
    pub fn delete(self: &Self) -> Result<(), String> {
        // If the node selection spans within a single node, then to delete that data, just update
        // the string literal value on the node
        if self.primary.node == self.secondary.node {
            let new_literal_start_offset = if self.primary.offset < self.secondary.offset {
                self.primary.offset
            } else {
                self.secondary.offset
            };

            // Construct a string, taking all the characters before the selection and the
            // characters after the selection, and sticking them together (omitting the selection
            // chars)
            let new_literal_length = self.secondary.offset.abs_diff(self.primary.offset);
            let new_literal_prefix = InMemoryNode::literal_substring(
                &self.primary.node,
                0,
                new_literal_start_offset,
            );
            let new_literal_suffix = InMemoryNode::literal_substring(
                &self.primary.node,
                new_literal_start_offset + new_literal_length,
                InMemoryNode::literal(&self.primary.node).len() - new_literal_start_offset,
            );
            let new_literal = format!("{new_literal_prefix}{new_literal_suffix}");

            // NOTE: should all nodes under the parent be combined and reparsed if
            // new_literal.len() == 0?
            InMemoryNode::set_literal(&self.primary.node, &new_literal);

            return Ok(());
        };

        // If the node selection spans multiple nodes, then:
        //
        // 1. Find the earlier node, and store the first part which should be kept
        let earlier_cursor = if self.primary.node < self.secondary.node { &self.primary } else { &self.secondary };
        let later_cursor = if self.primary.node < self.secondary.node { &self.secondary } else { &self.primary };
        let literal_prefix_to_keep = InMemoryNode::literal_substring(&earlier_cursor.node, 0, earlier_cursor.offset);

        // 2. Store the last part of the later node which should be kept
        let literal_suffix_to_keep = InMemoryNode::literal_substring(
            &later_cursor.node,
            later_cursor.offset,
            InMemoryNode::literal(&later_cursor.node).len() - later_cursor.offset,
        );

        // 3. Delete all nodes starting at after the earlier node up to and including the later node
        InMemoryNode::remove_nodes_sequentially_until(&earlier_cursor.node, Inclusivity::Exclusive, |node, _ct| {
            if node == &later_cursor.node {
                NodeSeek::Done(())
            } else {
                NodeSeek::Continue(())
            }
        });

        // 4. Keep going, storing literal text until back up at the same depth level as the
        //    earlier node. Swap the earlier node with a new node containing literal text of all
        //    the accumulated text.
        let earlier_node_depth = InMemoryNode::depth(&earlier_cursor.node);
        let resulting_literal_vectors = InMemoryNode::remove_nodes_sequentially_until(&later_cursor.node, Inclusivity::Exclusive, |node, _ct| {
            let literal = InMemoryNode::literal(node);

            let depth = InMemoryNode::depth(node);
            if depth > earlier_node_depth {
                // The node that was found was below `earlier_cursor.node` in the hierarchy, so
                // keep going
                NodeSeek::Continue(literal)
            } else {
                // The node was at or above `earlier_cursor.node`, so bail out
                NodeSeek::Stop
            }
        });

        let resulting_literal = format!(
            "{literal_prefix_to_keep}{}{literal_suffix_to_keep}",
            resulting_literal_vectors.collect::<String>(),
        );

        // Swap the earlier node with a new node containing literal text of all
        // the accumulated text.
        InMemoryNode::set_literal(&earlier_cursor.node, &resulting_literal);
        InMemoryNode::remove_all_children(&earlier_cursor.node);

        // 5. Reparse the newly created literal text node
        // TODO

        Ok(())
    }
}
