use crate::node_tree::{
    cursor::CursorSeek,
    node::{InMemoryNode, NodeSeek},
};
use std::{cell::RefCell, rc::Rc};

/// A cursor represents a position in a node tree - ie, a node and an offset in characters from the
/// start of that node. A cursor can be seeked forwards and backwards through the node tree to get
/// its contents or to perform operations on the node tree.
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
    pub fn seek_forwards_until<UntilFn>(self: &mut Self, until_fn: UntilFn) -> String
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
                            new_offset += 1;
                            *advance_until_char_counter_stack.last_mut().unwrap() += 1;
                            println!("... CONTINUE? {:?}", advance_until_char_counter_stack);
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
                            println!("... PUSH! {:?}", advance_until_char_counter_stack);
                            continue;
                        }
                        CursorSeek::Stop => {
                            advance_until_fn_stack.pop();
                            advance_until_char_counter_stack.pop();
                            println!("... STOP? {:?}", advance_until_char_counter_stack);
                        }
                        CursorSeek::Done => {
                            result.push(character);
                            global_char_counter += 1;
                            new_offset += 1;
                            advance_until_fn_stack.pop();
                            advance_until_char_counter_stack.pop();
                            println!("... DONE? {:?}", advance_until_char_counter_stack);
                        }
                    }
                    if !advance_until_fn_stack.is_empty()
                        || !advance_until_char_counter_stack.is_empty()
                    {
                        continue;
                    }
                }

                global_char_counter += 1;
                new_offset += 1;

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
                        println!("PUSH!");
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

        self.node = new_node;
        self.offset = new_offset;

        resulting_chars
            .flat_map(|vector| vector.into_iter())
            .collect::<String>()
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
