use std::{rc::Rc, cell::RefCell};
use crate::node_tree::node::{InMemoryNode, NodeSeek};

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
    AdvanceUntil { // Advance until the given `until_fn` check passes
        until_fn: Rc<RefCell<dyn FnMut(char, usize) -> CursorSeek>>,
    },
}

impl CursorSeek {
    pub fn advance_until<T>(until_fn: T) -> Self where T: FnMut(char, usize) -> CursorSeek + 'static {
        CursorSeek::AdvanceUntil {
            until_fn: Rc::new(RefCell::new(until_fn)),
        }
    }
    pub fn advance_until_char_then_done(character: char) -> Self {
        CursorSeek::advance_until(move |c, _i| {
            if c == character {
                CursorSeek::Done
            } else {
                CursorSeek::Continue
            }
        })
    }
    pub fn advance_until_char_then_stop(character: char) -> Self {
        CursorSeek::advance_until(move |c, _i| {
            if c == character {
                CursorSeek::Stop
            } else {
                CursorSeek::Continue
            }
        })
    }

    pub fn advance_lower_word(inclusive: CursorInclusivity) -> Self {
        let char_of_value_255 = char::from_u32(255).unwrap();

        let final_seek = match inclusive {
            CursorInclusivity::Inclusive => CursorSeek::Done,
            CursorInclusivity::Exclusive => CursorSeek::Stop,
        };

        let mut hit_word_char = false;

        // From :h word -
        // A word consists of a sequence of letters, digits and underscores, or a
        // sequence of other non-blank characters, separated with white space (spaces,
        // tabs, <EOL>).  This can be changed with the 'iskeyword' option.  An empty line
        // is also considered to be a word.
        CursorSeek::advance_until(move |c, _i| {
            let final_seek = final_seek.clone();

            // set iskeyword? @,48-57,_,192-255
            if c > char_of_value_255 || (c >= '0' && c <= '9') || c == '_' || (c >= 'A' && c <= char_of_value_255)  {
                // If a word character, keep going
                hit_word_char = true;
                CursorSeek::Continue
            } else if !hit_word_char && c == '\n' {
                // If a newling, then advance until whitespace after that new line stops
                CursorSeek::advance_until(move |c, _i| if c.is_whitespace() {
                    CursorSeek::Continue
                } else {
                    CursorSeek::Stop
                })
            } else if !hit_word_char && c.is_whitespace() {
                // If whitespace, then advance until the whitespace finishes, then resume the word
                // checking logic
                CursorSeek::advance_until(move |c, _i| if c.is_whitespace() {
                    CursorSeek::Continue
                } else {
                    CursorSeek::Stop
                })
            } else {
                final_seek.clone()
            }
        })
    }

    pub fn advance_upper_word(inclusive: CursorInclusivity) -> Self {
        let final_seek = match inclusive {
            CursorInclusivity::Inclusive => CursorSeek::Done,
            CursorInclusivity::Exclusive => CursorSeek::Stop,
        };

        let mut hit_word_char = false;

        // From :h WORD -
        // A WORD consists of a sequence of non-blank characters, separated with white
        // space.  An empty line is also considered to be a WORD.
        CursorSeek::advance_until(move |c, _i| {
            let final_seek = final_seek.clone();

            if !c.is_whitespace() {
                // If a word character, keep going
                hit_word_char = true;
                CursorSeek::Continue
            } else if !hit_word_char && c == '\n' {
                CursorSeek::advance_until(move |c, _i| if c.is_whitespace() {
                    CursorSeek::Continue
                } else {
                    CursorSeek::Stop
                })
            } else if !hit_word_char && c.is_whitespace() {
                // If whitespace, then advance until the whitespace finishes, then resume the word
                // checking logic
                CursorSeek::advance_until(move |c, _i| if c.is_whitespace() {
                    CursorSeek::Continue
                } else {
                    CursorSeek::Stop
                })
            } else {
                final_seek.clone()
            }
        })
    }

    // Note that `e` is always inclusive, ie `cw`, `de`, and `e` all end up with the cursor in
    // the same end spot
    // TODO: pub fn advance_lower_end() -> Self {
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
        let mut global_char_counter = 0; // Store a global count of characters processed

        // The final node and offset values:
        let mut new_offset = self.offset;
        let mut new_node = self.node.clone();

        // To handle CursorSeek::AdvanceByCharCount(n), keep a counter of characters to ekip:
        let mut cached_char_until_count = 0;

        // To handle CursorSeek::AdvanceUntil(...), keep a stack of `until_fn`s and their
        // corresponding counts - these should always have the same length:
        let mut advance_until_fn_stack: Vec<Rc<RefCell<dyn FnMut(char, usize) -> CursorSeek>>> = vec![];
        let mut advance_until_char_counter_stack: Vec<usize> = vec![];

        let resulting_chars = InMemoryNode::seek_forwards_until(&self.node, |node, _ct| {
            new_node = node.clone();
            new_offset = 0;
            let mut result = vec![];

            // Iterate over all characters within the node, one by one, until a match occurs:
            let node_literal = InMemoryNode::literal(node);
            let mut iterator = node_literal.chars();
            while let Some(character) = iterator.next() {
                println!("CHAR: {}", character);
                // If there's a char_until_count, then run until that exhausts iself
                if cached_char_until_count > 0 {
                    result.push(character);
                    cached_char_until_count -= 1;
                    if cached_char_until_count > 0 {
                        continue;
                    }
                }

                // If there's a char_until_fn, then run until that passes
                if let (
                    Some(advance_until_fn),
                    Some(advance_until_char_counter),
                ) = (&advance_until_fn_stack.last(), advance_until_char_counter_stack.last()) {
                    let value = {
                        let mut until_fn_borrowed_mut = advance_until_fn.borrow_mut();
                        until_fn_borrowed_mut(character, *advance_until_char_counter)
                    };

                    match value {
                        CursorSeek::Continue => {
                            result.push(character);
                            global_char_counter += 1;
                            new_offset += 1;
                            *advance_until_char_counter_stack.last_mut().unwrap() += 1;
                            println!("... CONTINUE? {:?}", advance_until_char_counter_stack);
                            continue;
                        },
                        CursorSeek::AdvanceByCharCount(n) => {
                            result.push(character);
                            cached_char_until_count += n;
                            continue;
                        },
                        CursorSeek::AdvanceUntil{ until_fn: char_until_fn } => {
                            result.push(character);
                            advance_until_fn_stack.push(char_until_fn);
                            advance_until_char_counter_stack.push(0);
                            println!("... PUSH! {:?}", advance_until_char_counter_stack);
                            continue;
                        },
                        CursorSeek::Stop => {
                            advance_until_fn_stack.pop();
                            advance_until_char_counter_stack.pop();
                            println!("... STOP? {:?}", advance_until_char_counter_stack);
                        },
                        CursorSeek::Done => {
                            result.push(character);
                            global_char_counter += 1;
                            new_offset += 1;
                            advance_until_fn_stack.pop();
                            advance_until_char_counter_stack.pop();
                            println!("... DONE? {:?}", advance_until_char_counter_stack);
                        },
                    }
                    if !advance_until_fn_stack.is_empty() || !advance_until_char_counter_stack.is_empty() {
                        continue;
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
                        cached_char_until_count += n;
                        continue;
                    },
                    CursorSeek::AdvanceUntil{ until_fn: char_until_fn } => {
                        result.push(character);
                        advance_until_fn_stack.push(char_until_fn);
                        advance_until_char_counter_stack.push(0);
                        println!("PUSH!");
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
