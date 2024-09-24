use std::{rc::Rc, cell::RefCell};
use crate::node::{InMemoryNode, NodeSeek};

pub enum CursorSeekAdvanceUntil {
    Continue,
    Stop,
    Done,
}

// An enum used by seek_forwards_until to control how seeking should commence.
pub enum CursorSeek {
    Continue, // Seek to the next character
    Stop, // Finish and don't include this character
    Done, // Finish and do include this character
    AdvanceByCharCount(usize), // Advance by N chars before checking again
    AdvanceUntil(fn(char) -> CursorSeekAdvanceUntil), // Advance until the given `until_fn` check passes
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

    // TODO: seek_forwards_until (Continue, AdvanceByCharCount(...), AdvanceUntilNextChar(...), Stop, Done)
    // Inclusive / exclusive?

    pub fn seek_forwards_until<UntilFn>(
        self: &mut Self,
        until_fn: UntilFn,
    ) -> String where UntilFn: Fn(char, usize) -> CursorSeek {
        let mut global_char_counter = 0;
        let mut new_offset = self.offset;
        let mut new_node = self.node.clone();

        let resulting_chars = InMemoryNode::seek_forwards_until(&self.node, |node, _ct| {
            new_node = node.clone();
            new_offset = 0;
            let mut result = vec![];

            // Iterate over all characters within the node, one by one, until a match occurs:
            let node_literal = InMemoryNode::literal(node);
            let mut iterator = node_literal.chars();
            while let Some(character) = iterator.next() {
                global_char_counter += 1;
                new_offset += 1;

                match until_fn(character, global_char_counter-1) {
                    CursorSeek::Continue => {
                        result.push(character);
                        continue;
                    },
                    CursorSeek::AdvanceByCharCount(n) => {
                        // FIXME: this doesn't work when `n` crosses the border from one node to
                        // another node! Track instead this value as a mut outside the closure
                        for _ in 0..n {
                            global_char_counter += 1;
                            iterator.next();
                        }
                        continue;
                    },
                    CursorSeek::AdvanceUntil(until_fn) => {
                        while let Some(character) = iterator.next() {
                            match until_fn(character) {
                                CursorSeekAdvanceUntil::Continue => {
                                    global_char_counter += 1;
                                    continue;
                                },
                                CursorSeekAdvanceUntil::Stop => {
                                    break;
                                },
                                CursorSeekAdvanceUntil::Done => {
                                    global_char_counter += 1;
                                    break;
                                },
                            }
                        }
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
}
