use crate::node_tree::utils::{Inclusivity, is_lower_word_char, is_upper_word_char};
use std::{cell::RefCell, rc::Rc};

// An enum used by seek_forwards_until to control how seeking should commence.
#[derive(Clone)]
pub enum CursorSeek {
    Continue,                  // Seek to the next character
    Stop,                      // Finish and don't include this character
    Done,                      // Finish and do include this character
    AdvanceByCharCount(usize), // Advance by N chars before checking again
    AdvanceUntil {
        // Advance until the given `until_fn` check passes
        until_fn: Rc<RefCell<dyn FnMut(char, usize) -> CursorSeek>>,
    },
}

impl CursorSeek {
    pub fn advance_until<T>(until_fn: T) -> Self
    where
        T: FnMut(char, usize) -> CursorSeek + 'static,
    {
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

    pub fn advance_lower_word(inclusive: Inclusivity) -> Self {
        let final_seek = match inclusive {
            Inclusivity::Inclusive => CursorSeek::Done,
            Inclusivity::Exclusive => CursorSeek::Stop,
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
            if is_lower_word_char(c) {
                // If a word character, keep going
                hit_word_char = true;
                CursorSeek::Continue
            } else if !hit_word_char && c == '\n' {
                // If a newling, then advance until whitespace after that new line stops
                CursorSeek::advance_until(move |c, _i| {
                    if c.is_whitespace() {
                        CursorSeek::Continue
                    } else {
                        CursorSeek::Stop
                    }
                })
            } else if !hit_word_char && c.is_whitespace() {
                // If whitespace, then advance until the whitespace finishes, then resume the word
                // checking logic
                CursorSeek::advance_until(move |c, _i| {
                    if c.is_whitespace() {
                        CursorSeek::Continue
                    } else {
                        CursorSeek::Stop
                    }
                })
            } else {
                final_seek.clone()
            }
        })
    }

    pub fn advance_upper_word(inclusive: Inclusivity) -> Self {
        let final_seek = match inclusive {
            Inclusivity::Inclusive => CursorSeek::Done,
            Inclusivity::Exclusive => CursorSeek::Stop,
        };

        let mut hit_word_char = false;

        // From :h WORD -
        // A WORD consists of a sequence of non-blank characters, separated with white
        // space.  An empty line is also considered to be a WORD.
        CursorSeek::advance_until(move |c, _i| {
            let final_seek = final_seek.clone();

            if is_upper_word_char(c) {
                // If a word character, keep going
                hit_word_char = true;
                CursorSeek::Continue
            } else if !hit_word_char && c == '\n' {
                CursorSeek::advance_until(move |c, _i| {
                    if c.is_whitespace() {
                        CursorSeek::Continue
                    } else {
                        CursorSeek::Stop
                    }
                })
            } else if !hit_word_char && c.is_whitespace() {
                // If whitespace, then advance until the whitespace finishes, then resume the word
                // checking logic
                CursorSeek::advance_until(move |c, _i| {
                    if c.is_whitespace() {
                        CursorSeek::Continue
                    } else {
                        CursorSeek::Stop
                    }
                })
            } else {
                final_seek.clone()
            }
        })
    }

    // Note that `e` is always inclusive, ie `ce`, `de`, and `e` all end up with the cursor in
    // the same end spot
    pub fn advance_lower_end() -> Self {
        let mut hit_word_char = false;

        // From :h word -
        // A word consists of a sequence of letters, digits and underscores, or a
        // sequence of other non-blank characters, separated with white space (spaces,
        // tabs, <EOL>).  This can be changed with the 'iskeyword' option.  An empty line
        // is also considered to be a word.
        CursorSeek::advance_until(move |c, _i| {
            if is_lower_word_char(c) {
                // If a word character, keep going
                hit_word_char = true;
                CursorSeek::Continue
            } else if !hit_word_char && !is_lower_word_char(c) {
                // If not word character and a word character hasn't been encountered yet, keep
                // going until a word character is hit.
                CursorSeek::Continue
            } else if !hit_word_char && c == '\n' {
                // If a newline, then advance until whitespace after that new line stops, and then
                // start looking for word chars
                CursorSeek::advance_until(move |c, _i| {
                    if c.is_whitespace() {
                        CursorSeek::Continue
                    } else {
                        CursorSeek::Stop
                    }
                })
            } else if !hit_word_char && c.is_whitespace() {
                // If whitespace, then advance until the whitespace finishes, then resume the word
                // checking logic
                CursorSeek::advance_until(move |c, _i| {
                    if c.is_whitespace() {
                        CursorSeek::Continue
                    } else {
                        CursorSeek::Stop
                    }
                })
            } else {
                CursorSeek::Stop
            }
        })
    }

    // Note that `e` is always inclusive, ie `ce`, `de`, and `e` all end up with the cursor in
    // the same end spot
    pub fn advance_upper_end() -> Self {
        let mut hit_word_char = false;

        // From :h WORD -
        // A WORD consists of a sequence of non-blank characters, separated with white
        // space.  An empty line is also considered to be a WORD.
        CursorSeek::advance_until(move |c, _i| {
            if is_upper_word_char(c) {
                // If a word character, keep going
                hit_word_char = true;
                CursorSeek::Continue
            } else if !hit_word_char && !is_upper_word_char(c) {
                // If not word character and a word character hasn't been encountered yet, keep
                // going until a word character is hit.
                CursorSeek::Continue
            } else if !hit_word_char && c == '\n' {
                // If a newline, then advance until whitespace after that new line stops, and then
                // start looking for word chars
                CursorSeek::advance_until(move |c, _i| {
                    if c.is_whitespace() {
                        CursorSeek::Continue
                    } else {
                        CursorSeek::Stop
                    }
                })
            } else if !hit_word_char && c.is_whitespace() {
                // If whitespace, then advance until the whitespace finishes, then resume the word
                // checking logic
                CursorSeek::advance_until(move |c, _i| {
                    if c.is_whitespace() {
                        CursorSeek::Continue
                    } else {
                        CursorSeek::Stop
                    }
                })
            } else {
                CursorSeek::Stop
            }
        })
    }
}
