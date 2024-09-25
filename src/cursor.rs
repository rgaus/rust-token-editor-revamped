use std::{rc::Rc, cell::RefCell};
use crate::node::{InMemoryNode, NodeSeek};

pub enum CursorSeekAdvanceUntil {
    Continue,
    Stop,
    Done,
}

pub enum CursorInclusivity {
    Inclusive,
    Exclusive,
}

// An enum used by seek_forwards_until to control how seeking should commence.
#[derive(Clone)]
pub enum CursorSeek {
    Continue, // Seek to the next character
    Stop, // Finish and don't include this character
    Done, // Finish and do include this character
    AdvanceByCharCount(usize), // Advance by N chars before checking again
    AdvanceUntil(Rc<dyn Fn(char) -> CursorSeekAdvanceUntil>), // Advance until the given `until_fn` check passes
}

impl CursorSeek {
    pub fn advance_until<T>(until_fn: T) -> Self where T: Fn(char) -> CursorSeekAdvanceUntil + 'static {
        CursorSeek::AdvanceUntil(Rc::new(until_fn))
    }
    pub fn advance_until_char_then_done(character: char) -> Self {
        CursorSeek::advance_until(move |c| {
            if c == character {
                CursorSeekAdvanceUntil::Done
            } else {
                CursorSeekAdvanceUntil::Continue
            }
        })
    }
    pub fn advance_until_char_then_stop(character: char) -> Self {
        CursorSeek::advance_until(move |c| {
            if c == character {
                CursorSeekAdvanceUntil::Stop
            } else {
                CursorSeekAdvanceUntil::Continue
            }
        })
    }
    pub fn advance_lower_word(inclusive: CursorInclusivity) -> Self {
        let char_of_value_255 = char::from_u32(255).unwrap();

        // letters / digits / underscores
        //
        CursorSeek::advance_until(move |c| {
            // set iskeyword? @,48-57,_,192-255
            if c > char_of_value_255 || (c >= '0' && c <= '9') || c == '_' || (c >= 'A' && c <= char_of_value_255)  {
                CursorSeekAdvanceUntil::Continue
            // } else if c === 10 {
            //     CursorSeekAdvanceUntil::Continue
            } else {
                match inclusive {
                    CursorInclusivity::Inclusive => CursorSeekAdvanceUntil::Done,
                    CursorInclusivity::Exclusive => CursorSeekAdvanceUntil::Stop,
                }
            }
        })
    }
}

pub struct Cursor {
    node: Rc<RefCell<InMemoryNode>>,
    offset: usize,
}

impl Cursor {
    pub fn new(node: Rc<RefCell<InMemoryNode>>) -> Self {
        Self::new_at(node, 0)
    }
    pub fn new_at(node: Rc<RefCell<InMemoryNode>>, offset: usize) -> Self {
        Self { node, offset }
    }

    /// When called, seeks forward starting at the cursor position character by character through
    /// the node structure until the given `until_fn` returns either `Stop` or `Done`.
    pub fn seek_forwards_until<UntilFn>(
        self: &mut Self,
        until_fn: UntilFn,
    ) -> String where UntilFn: Fn(char, usize) -> CursorSeek {
        let mut global_char_counter = 0;
        let mut new_offset = self.offset;
        let mut new_node = self.node.clone();

        let mut cached_char_until_count: Option<usize> = None;
        let mut cached_char_until_fn: Option<Rc<dyn Fn(char) -> CursorSeekAdvanceUntil>> = None;

        let resulting_chars = InMemoryNode::seek_forwards_until(&self.node, |node, _ct| {
            new_node = node.clone();
            new_offset = 0;
            let mut result = vec![];

            // Iterate over all characters within the node, one by one, until a match occurs:
            let node_literal = InMemoryNode::literal(node);
            let mut iterator = node_literal.chars();
            while let Some(character) = iterator.next() {
                // If there's a char_until_count, then run until that exhausts iself
                if let Some(char_until_count) = cached_char_until_count {
                    if char_until_count > 1 {
                        result.push(character);
                        cached_char_until_count = Some(char_until_count-1);
                        continue;
                    } else {
                        cached_char_until_count = None;
                    }
                }

                // If there's a char_until_fn, then run until that passes
                if let Some(char_until_fn) = &cached_char_until_fn {
                    match char_until_fn(character) {
                        CursorSeekAdvanceUntil::Continue => {
                            result.push(character);
                            global_char_counter += 1;
                            new_offset += 1;
                            continue;
                        },
                        CursorSeekAdvanceUntil::Stop => {
                            cached_char_until_fn = None;
                        },
                        CursorSeekAdvanceUntil::Done => {
                            result.push(character);
                            global_char_counter += 1;
                            new_offset += 1;
                            cached_char_until_fn = None;
                        },
                    }
                }

                global_char_counter += 1;
                new_offset += 1;

                match until_fn(character, global_char_counter-1) {
                    CursorSeek::Continue => {
                        result.push(character);
                        continue;
                    },
                    CursorSeek::AdvanceByCharCount(n) => {
                        result.push(character);
                        cached_char_until_count = Some(n);
                        continue;
                    },
                    CursorSeek::AdvanceUntil(char_until_fn) => {
                        result.push(character);
                        cached_char_until_fn = Some(char_until_fn);
                        continue;
                    },
                    CursorSeek::Stop => {
                        return NodeSeek::Done(result);
                    },
                    CursorSeek::Done => {
                        result.push(character);
                        return NodeSeek::Done(result);
                    },
                }
            }

            NodeSeek::Continue(result)
        });

        self.node = new_node;
        self.offset = new_offset;

        resulting_chars.flat_map(|vector| vector.into_iter()).collect::<String>()
    }

    /// When called, performs the given `seek` operation once, causing the cursor to seek forwards
    /// by the given amount
    pub fn seek_forwards(self: &mut Self, seek: CursorSeek) -> String {
        self.seek_forwards_until(|_character, index| {
            if index == 0 {
                seek.clone()
            } else {
                CursorSeek::Stop
            }
        })
    }
}
