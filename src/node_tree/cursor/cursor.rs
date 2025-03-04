use crate::node_tree::{
    cursor::{cursor_seek::CursorSeekContext, CursorSeek, Selection},
    node::{InMemoryNode, NodeSeek, TokenKindTrait},
    utils::{Direction, Inclusivity, NEWLINE},
};
use std::{cell::RefCell, collections::VecDeque, fmt::Debug, rc::Rc};

/// A cursor represents a position in a node tree - ie, a node and an offset in characters from the
/// start of that node. A cursor can be seeked forwards and backwards through the node tree to get
/// its contents or to perform operations on the node tree.
#[derive(Clone)]
pub struct Cursor<TokenKind: TokenKindTrait> {
    pub node: Rc<RefCell<InMemoryNode<TokenKind>>>,
    pub offset: usize,
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

impl<TokenKind: TokenKindTrait> PartialEq for Cursor<TokenKind> {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node && self.offset == other.offset
    }
}

impl<TokenKind: TokenKindTrait> PartialOrd for Cursor<TokenKind> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.node < other.node {
            Some(std::cmp::Ordering::Less)
        } else if self.node > other.node {
            Some(std::cmp::Ordering::Greater)
        } else {
            // Nodes must be the same! So, compare offsets:
            if self.offset < other.offset {
                Some(std::cmp::Ordering::Less)
            } else if self.offset > other.offset {
                Some(std::cmp::Ordering::Greater)
            } else {
                Some(std::cmp::Ordering::Equal)
            }
        }
    }
}

impl<TokenKind: TokenKindTrait> Cursor<TokenKind> {
    pub fn new(node: Rc<RefCell<InMemoryNode<TokenKind>>>) -> Self {
        Self::new_at(node, 0)
    }
    pub fn new_at(node: Rc<RefCell<InMemoryNode<TokenKind>>>, offset: usize) -> Self {
        Self { node, offset }
    }
    pub fn new_at_rows_cols(
        root: Rc<RefCell<InMemoryNode<TokenKind>>>,
        rows_cols: (usize, usize),
    ) -> Self {
        let (rows, cols) = rows_cols;
        let mut row_counter = 1;
        let mut col_counter = 1;

        let cursor = Self::new(root).seek_forwards_until(|c, _i| {
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

        println!("--------");
        let _ = self.seek_backwards_until(|c, _i| {
            if c == *NEWLINE {
                // From that point on count each newline
                row_counter += 1;
            } else if row_counter == 1 {
                // Before reaching the first newline, count the col chars
                col_counter += 1;
            };
            CursorSeek::Continue
        });

        (row_counter, col_counter)
    }

    pub fn to_cols(self: &Self) -> usize {
        let mut col_counter = 1;

        let _ = self.seek_backwards_until(|c, _i| {
            if c == *NEWLINE {
                CursorSeek::Stop
            } else {
                // Before reaching the first newline, count the col chars
                col_counter += 1;
                CursorSeek::Continue
            }
        });

        col_counter
    }

    pub fn to_rows(self: &Self) -> usize {
        self.to_rows_cols().0
    }

    /// When called, seeks starting at the cursor position character by character through the node
    /// structure in the giren `direction` until the given `until_fn` returns either `Stop` or `Done`.
    pub fn seek_until<UntilFn>(
        self: &Self,
        initial_direction: Direction,
        mut until_fn: UntilFn,
    ) -> Self
    where
        UntilFn: FnMut(char, usize) -> CursorSeek,
    {
        let mut direction = initial_direction;
        let mut global_char_counter = 0; // Store a global count of characters processed

        // The final node and offset values:
        let mut new_offset = self.offset;
        let mut new_node = self.node.clone();

        // To handle CursorSeek::AdvanceByCharCount(n), keep a counter of characters to skip:
        let mut cached_char_until_count = 0;

        // To handle CursorSeek::AdvanceByLineCount(n), keep a counter of lines to skip:
        #[derive(Debug, PartialEq)]
        enum AdvanceByLineCountState {
            Inactive,
            ScanningForwardTowardsNewline { chars_before_start: usize },
            AdvancingCharactersOnNextLine { remaining: usize },
        }
        let mut cached_line_until_count = 0;
        let mut cached_line_state = AdvanceByLineCountState::Inactive;

        // To handle CursorSeek::AdvanceUntil(...), keep a stack of `until_fn`s and their
        // corresponding counts - these should always have the same length:
        let mut advance_until_fn_stack: Vec<
            Rc<RefCell<dyn FnMut(char, CursorSeekContext) -> CursorSeek>>,
        > = vec![];
        let mut advance_until_char_counter_stack: Vec<usize> = vec![];

        let _ = InMemoryNode::seek_until(
            &self.node,
            direction,
            Inclusivity::Inclusive,
            |node, ct| {
                new_node = node.clone();

                let direction_at_start_of_node = direction.clone();

                let node_literal = InMemoryNode::literal(node);
                let mut characters = if ct == 0 {
                    // If this is the first node, skip forward / backward `self.offset` times.
                    match direction {
                        Direction::Forwards => {
                            // Seek from the start to the offset
                            node_literal
                                .chars()
                                .skip(self.offset)
                                .collect::<VecDeque<char>>()
                        }
                        Direction::Backwards => {
                            // Seek from the end to the offset from the start
                            let mut iterator = node_literal.chars();
                            for _ in 0..(node_literal.len() - self.offset) {
                                iterator.next_back();
                            }
                            iterator.collect::<VecDeque<char>>()
                        }
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
                        dbg!(cached_line_until_count, &cached_line_state);

                        if cached_line_state == AdvanceByLineCountState::Inactive {
                            println!("--- LINE START! ---");
                            // 1. Figure out how many characters are before the current cursor in the
                            //    line
                            let mut current_cols = self.offset + 1;
                            let _ = InMemoryNode::seek_until(
                                &self.node,
                                Direction::Backwards,
                                Inclusivity::Exclusive,
                                |inner_node, _ct| {
                                    for inner_character in
                                        InMemoryNode::literal(inner_node).chars().rev()
                                    {
                                        if inner_character == *NEWLINE {
                                            return NodeSeek::Stop;
                                        }
                                        current_cols += 1;
                                    }
                                    return NodeSeek::Continue(());
                                },
                            );

                            cached_line_state =
                                AdvanceByLineCountState::ScanningForwardTowardsNewline {
                                    chars_before_start: current_cols + 1,
                                };
                        }

                        match cached_line_state {
                            AdvanceByLineCountState::Inactive => {
                                unreachable!("AdvanceByLineCountState::Inactive should have been handled earlier in the control flow!");
                            }
                            AdvanceByLineCountState::ScanningForwardTowardsNewline {
                                chars_before_start,
                            } => {
                                global_char_counter += 1;
                                new_offset = match direction {
                                    Direction::Forwards => new_offset + 1,
                                    Direction::Backwards => new_offset - 1,
                                };

                                // 2. If the first newline hasn't been reached, then keep going until it is
                                // reached
                                if character == *NEWLINE {
                                    cached_line_state =
                                        AdvanceByLineCountState::AdvancingCharactersOnNextLine {
                                            remaining: chars_before_start,
                                        };
                                };
                                continue;
                            }
                            AdvanceByLineCountState::AdvancingCharactersOnNextLine {
                                remaining,
                            } => {
                                let mut remaining_copy = remaining;
                                if remaining_copy > 0 {
                                    // 3. Advance cached_line_current_cols (in this context, chars_before_start) characters
                                    //    to get to the next line
                                    remaining_copy -= 1;
                                    cached_line_state =
                                        AdvanceByLineCountState::AdvancingCharactersOnNextLine {
                                            remaining: remaining_copy,
                                        };
                                }

                                if remaining_copy > 0 {
                                    global_char_counter += 1;
                                    new_offset = match direction {
                                        Direction::Forwards => new_offset + 1,
                                        Direction::Backwards => new_offset - 1,
                                    };
                                    continue;
                                }

                                cached_line_until_count -= 1;
                                cached_line_state = AdvanceByLineCountState::Inactive;
                                println!(
                                    "--- LINE DONE! --- cached_line_until_count={}",
                                    cached_line_until_count
                                );

                                if cached_line_until_count > 0 {
                                    continue;
                                }
                            }
                        }
                    }

                    // If there's a char_until_fn, then run until that passes
                    if let (Some(advance_until_fn), Some(advance_until_char_counter)) = (
                        &advance_until_fn_stack.last(),
                        advance_until_char_counter_stack.last(),
                    ) {
                        let value = {
                            let mut until_fn_borrowed_mut = advance_until_fn.borrow_mut();
                            until_fn_borrowed_mut(
                                character,
                                CursorSeekContext {
                                    direction,
                                    index: *advance_until_char_counter,
                                    is_at_end_of_line: {
                                        let next_char = match direction {
                                            Direction::Forwards => characters.front(),
                                            Direction::Backwards => characters.back(),
                                        };
                                        next_char.map_or(false, |next_char| next_char == NEWLINE)
                                    }
                                },
                            )
                        };

                        match value {
                            CursorSeek::Continue => {
                                global_char_counter += 1;
                                new_offset = match direction {
                                    Direction::Forwards => new_offset + 1,
                                    Direction::Backwards => new_offset - 1,
                                };
                                *advance_until_char_counter_stack.last_mut().unwrap() += 1;
                                continue;
                            }
                            CursorSeek::AdvanceByLines(n) => {
                                if n == 0 {
                                    continue;
                                };
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
                                cached_char_until_count += n + 1;

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
                                only_in_direction,
                            } => {
                                if only_in_direction.is_some_and(|d| d != direction) {
                                    panic!("CursorSeek::AdvanceUntil only_in_direction was {only_in_direction:?}, but direction was {direction:?}. This is not allowed!");
                                };
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
                                global_char_counter += 1;
                                new_offset = match direction {
                                    Direction::Forwards => new_offset + 1,
                                    Direction::Backwards => new_offset - 1,
                                };

                                advance_until_fn_stack.pop();
                                advance_until_char_counter_stack.pop();
                            }
                            CursorSeek::ChangeDirection(new_direction) => {
                                direction = new_direction;

                                new_offset = match direction {
                                    Direction::Forwards => new_offset - 1,
                                    Direction::Backwards => new_offset + 1,
                                };

                                characters = match new_direction {
                                    Direction::Forwards => {
                                        // Seek from the start to the offset
                                        node_literal
                                            .chars()
                                            .skip(new_offset)
                                            .collect::<VecDeque<char>>()
                                        }
                                    Direction::Backwards => {
                                        // Seek from the end to the offset from the start
                                        let mut iterator = node_literal.chars();
                                        for _ in 0..(node_literal.len() - new_offset) {
                                            iterator.next_back();
                                        }
                                        iterator.collect::<VecDeque<char>>()
                                    }
                                };

                                continue;
                            }
                            CursorSeek::Fail(message) => {
                                return NodeSeek::Fail(message);
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
                            global_char_counter += 1;
                            new_offset = match direction {
                                Direction::Forwards => new_offset + 1,
                                Direction::Backwards => new_offset - 1,
                            };
                            continue;
                        }
                        CursorSeek::AdvanceByCharCount(n) => {
                            cached_char_until_count += n + 1;

                            // NOTE: re-add the character back to the characters vec, so that it
                            // can be skipped with the AdvanceByCharCount skip code
                            match direction {
                                Direction::Forwards => characters.push_front(character),
                                Direction::Backwards => characters.push_back(character),
                            };
                            continue;
                        }
                        CursorSeek::AdvanceByLines(n) => {
                            if n == 0 {
                                continue;
                            };
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
                            only_in_direction,
                        } => {
                            if only_in_direction.is_some_and(|d| d != direction) {
                                panic!("CursorSeek::AdvanceUntil only_in_direction was {only_in_direction:?}, but direction was {direction:?}. This is not allowed!");
                            };
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
                            return NodeSeek::Done(());
                        }
                        CursorSeek::Done => {
                            global_char_counter += 1;
                            new_offset = match direction {
                                Direction::Forwards => new_offset + 1,
                                Direction::Backwards => new_offset - 1,
                            };
                            return NodeSeek::Done(());
                        }
                        CursorSeek::ChangeDirection(new_direction) => {
                            direction = new_direction;

                            new_offset = match direction {
                                Direction::Forwards => new_offset - 1,
                                Direction::Backwards => new_offset + 1,
                            };

                            characters = match new_direction {
                                Direction::Forwards => {
                                    // Seek from the start to the offset
                                    node_literal
                                        .chars()
                                        .skip(new_offset)
                                        .collect::<VecDeque<char>>()
                                    }
                                Direction::Backwards => {
                                    // Seek from the end to the offset from the start
                                    let mut iterator = node_literal.chars();
                                    for _ in 0..(node_literal.len() - new_offset) {
                                        iterator.next_back();
                                    }
                                    iterator.collect::<VecDeque<char>>()
                                }
                            };
                            continue;
                        }
                        CursorSeek::Fail(message) => {
                            return NodeSeek::Fail(message);
                        }
                    }
                }

                // The whole node has been parsed! If the direction changed half way through though
                // then change the node seek direction so that the correct node in the proper direction
                // is selected next.
                if direction_at_start_of_node != direction {
                    NodeSeek::ChangeDirection((), direction)
                } else {
                    NodeSeek::Continue(())
                }
            },
        );

        Self::new_at(new_node, new_offset)
    }

    /// When called, seeks forward starting at the cursor position character by character through
    /// the node structure until the given `until_fn` returns either `Stop` or `Done`.
    pub fn seek_forwards_until<UntilFn>(self: &Self, until_fn: UntilFn) -> Self
    where
        UntilFn: FnMut(char, usize) -> CursorSeek,
    {
        self.seek_until(Direction::Forwards, until_fn)
    }

    /// When called, performs the given `seek` operation once, causing the cursor to seek forwards
    /// by the given amount
    pub fn seek_forwards(self: &Self, seek: CursorSeek) -> Self {
        let mut is_first = true;
        self.seek_forwards_until(|_character, _index| {
            if is_first {
                is_first = false;
                seek.clone()
            } else {
                CursorSeek::Stop
            }
        })
    }

    /// When called, seeks backward starting at the cursor position character by character through
    /// the node structure until the given `until_fn` returns either `Stop` or `Done`.
    pub fn seek_backwards_until<UntilFn>(self: &Self, until_fn: UntilFn) -> Self
    where
        UntilFn: FnMut(char, usize) -> CursorSeek,
    {
        self.seek_until(Direction::Backwards, until_fn)
    }

    /// When called, performs the given `seek` operation once, causing the cursor to seek backwards
    /// by the given amount
    pub fn seek_backwards(self: &Self, seek: CursorSeek) -> Self {
        let mut is_first = true;
        self.seek_backwards_until(|_character, _index| {
            if is_first {
                is_first = false;
                seek.clone()
            } else {
                CursorSeek::Stop
            }
        })
    }
}
