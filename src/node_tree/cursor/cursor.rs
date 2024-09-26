use crate::node_tree::{
    cursor::CursorSeek,
    node::{InMemoryNode, NodeSeek},
    utils::{Direction, Inclusivity},
};
use std::{cell::RefCell, rc::Rc, fmt::Debug};

/// A cursor represents a position in a node tree - ie, a node and an offset in characters from the
/// start of that node. A cursor can be seeked forwards and backwards through the node tree to get
/// its contents or to perform operations on the node tree.
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

        // To handle CursorSeek::AdvanceByCharCount(n), keep a counter of characters to ekip:
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
                            cached_char_until_count += n;
                            continue;
                        }
                        CursorSeek::AdvanceUntil {
                            until_fn: char_until_fn,
                        } => {
                            result.push(character);
                            advance_until_fn_stack.push(char_until_fn);
                            advance_until_char_counter_stack.push(0);
                            continue;
                        }
                        CursorSeek::Stop => {
                            advance_until_fn_stack.pop();
                            advance_until_char_counter_stack.pop();
                        }
                        CursorSeek::Done => {
                            result.push(character);
                            advance_until_fn_stack.pop();
                            advance_until_char_counter_stack.pop();

                            let other_until_fns_are_in_the_stack = (
                                !advance_until_fn_stack.is_empty() || !advance_until_char_counter_stack.is_empty()
                            );

                            // NOTE: these values will get incremented after this as part of
                            // the main while loop if this was the final until_fn, so skip the
                            // increments on the final stack item to avoid doing them twice.
                            if other_until_fns_are_in_the_stack {
                                global_char_counter += 1;
                                new_offset = match direction {
                                    Direction::Forwards => new_offset + 1,
                                    Direction::Backwards => new_offset - 1,
                                };
                            }
                        }
                    }
                    if !advance_until_fn_stack.is_empty()
                        || !advance_until_char_counter_stack.is_empty()
                    {
                        continue;
                    }
                }

                global_char_counter += 1;
                new_offset = match direction {
                    Direction::Forwards => new_offset + 1,
                    Direction::Backwards => new_offset - 1,
                };

                match until_fn(character, global_char_counter - 1) {
                    CursorSeek::Continue => {
                        result.push(character);
                        continue;
                    }
                    CursorSeek::AdvanceByCharCount(n) => {
                        result.push(character);
                        cached_char_until_count += n;
                        continue;
                    }
                    CursorSeek::AdvanceUntil {
                        until_fn: char_until_fn,
                    } => {
                        result.push(character);
                        advance_until_fn_stack.push(char_until_fn);
                        advance_until_char_counter_stack.push(0);
                        continue;
                    }
                    CursorSeek::Stop => {
                        return NodeSeek::Done(result);
                    }
                    CursorSeek::Done => {
                        result.push(character);
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
