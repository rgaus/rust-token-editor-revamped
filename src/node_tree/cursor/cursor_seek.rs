use crate::node_tree::utils::{
    Inclusivity,
    is_lower_word_char,
    is_upper_word_char,
    Newline,
    NEWLINE,
    is_delimiter,
    Delimiter,
    Direction, DELIMITER_LOOKBACK_BUFFER_LENGTH_CHARS,
};
use std::{cell::RefCell, rc::Rc};

// An enum used by seek_forwards_until to control how seeking should commence.
#[derive(Clone)]
pub enum CursorSeek {
    Continue,                   // Seek to the next character
    Stop,                       // Finish and don't include this character
    Done,                       // Finish and do include this character
    AdvanceByCharCount(usize),  // Advance by N chars before checking again
    AdvanceByLines(usize),      // Advance by N lines before checking again
    AdvanceUntil {
        // Advance until the given `until_fn` check passes
        until_fn: Rc<RefCell<dyn FnMut(char, usize) -> CursorSeek>>,
    },
    ChangeDirection(Direction), // Change to seeking in the given direction
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

    /// When called, advances up until the given character, also including that character.
    pub fn advance_until_char_then_done(character: char, newline: Newline) -> Self {
        CursorSeek::advance_until(move |c, _i| {
            if c == character {
                CursorSeek::Done
            } else if newline == Newline::ShouldTerminate && c == *NEWLINE {
                CursorSeek::Done
            } else {
                CursorSeek::Continue
            }
        })
    }

    /// When called, advances up until the given character, but NOT including that character.
    pub fn advance_until_char_then_stop(character: char, newline: Newline) -> Self {
        CursorSeek::advance_until(move |c, _i| {
            if c == character {
                CursorSeek::Stop
            } else if newline == Newline::ShouldTerminate && c == *NEWLINE {
                CursorSeek::Done
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

    /// When called, advances forwards until the end of the lower word.
    ///
    /// Note that `e` is always inclusive, ie `ce`, `de`, and `e` all end up with the cursor in
    /// the same end spot
    ///
    /// NOTE: ONLY WORKS WHEN SEEKING FORWARDS!
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

    /// When called, advances forwards until the end of the upper word.
    ///
    /// Note that `E` is always inclusive, ie `ce`, `de`, and `e` all end up with the cursor in
    /// the same end spot
    ///
    /// NOTE: ONLY WORKS WHEN SEEKING FORWARDS!
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

    /// When called, advance forwards to the start of the line.
    ///
    /// NOTE: ONLY WORKS WHEN SEEKING FORWARDS!
    pub fn advance_until_line_end(inclusive: Inclusivity) -> Self {
        match inclusive {
            Inclusivity::Inclusive => CursorSeek::advance_until_char_then_done(*NEWLINE, Newline::ShouldTerminate),
            Inclusivity::Exclusive => CursorSeek::advance_until_char_then_stop(*NEWLINE, Newline::ShouldTerminate),
        }
    }

    /// When called, advance backwards to the start of the line.
    ///
    /// NOTE: ONLY WORKS WHEN SEEKING BACKWARDS!
    pub fn advance_until_line_start() -> Self {
        CursorSeek::advance_until(move |c, _i| {
            if c == *NEWLINE {
                CursorSeek::Stop
            } else {
                CursorSeek::Continue
            }
        })
    }

    /// When called, advanced to the start or end of a document
    pub fn advance_until_start_end() -> Self {
        CursorSeek::advance_until(|_c, _i| CursorSeek::Continue)
    }

    // %
    pub fn advance_until_matching_delimiter(inclusive: Inclusivity) -> Self {
        #[derive(Debug)]
        enum Mode {
            FindingInitialDelimeter(Vec<char>),
            FindingDelimiterSeekingForwards(Vec<char>, Delimiter),
            FindingDelimiterSeekingBackwards(Vec<char>, Delimiter),
            StopOnNextChar,
        }
        let mut mode = Mode::FindingInitialDelimeter(vec![]);

        CursorSeek::advance_until(move |c, _i| {
            match &mode {
                Mode::FindingInitialDelimeter(buffer) => {
                    // No delimiter found, add to buffer and keep going
                    let mut buffer_copy = buffer.clone();
                    buffer_copy.push(c);
                    if buffer_copy.len() > DELIMITER_LOOKBACK_BUFFER_LENGTH_CHARS {
                        buffer_copy.remove(0);
                    };

                    let Some(found_delimeter) = is_delimiter(&buffer_copy[..]) else {
                        // No delimiter found, add to buffer and keep going
                        mode = Mode::FindingInitialDelimeter(buffer_copy);
                        return CursorSeek::Continue;
                    };

                    if matches!(found_delimeter, Delimiter::End(..)) {
                        mode = Mode::FindingDelimiterSeekingBackwards(buffer_copy, found_delimeter);
                        CursorSeek::ChangeDirection(Direction::Backwards)
                    } else {
                        mode = Mode::FindingDelimiterSeekingForwards(buffer_copy, found_delimeter);
                        CursorSeek::Continue
                    }
                },
                Mode::FindingDelimiterSeekingForwards(buffer, initial_delimiter) => {
                    // No delimiter found, add to buffer and keep going
                    let mut buffer_copy = buffer.clone();
                    buffer_copy.push(c);
                    if buffer_copy.len() > DELIMITER_LOOKBACK_BUFFER_LENGTH_CHARS {
                        buffer_copy.remove(0);
                    };

                    let Some(found_delimeter) = is_delimiter(&buffer_copy[..]) else {
                        // No delimiter found, add to buffer and keep going
                        mode = Mode::FindingDelimiterSeekingForwards(buffer_copy, initial_delimiter.clone());
                        return CursorSeek::Continue;
                    };

                    match (initial_delimiter, found_delimeter.clone()) {
                        (Delimiter::Start(initial_type, _), Delimiter::Midpoint(found_type, found_length))
                        | (Delimiter::Start(initial_type, _), Delimiter::End(found_type, found_length))
                        | (Delimiter::Start(initial_type, _), Delimiter::EitherStartOrEnd(found_type, found_length))
                        | (Delimiter::EitherStartOrEnd(initial_type, _), Delimiter::Midpoint(found_type, found_length))
                        | (Delimiter::EitherStartOrEnd(initial_type, _), Delimiter::End(found_type, found_length))
                        | (Delimiter::EitherStartOrEnd(initial_type, _), Delimiter::EitherStartOrEnd(found_type, found_length))
                        | (Delimiter::Midpoint(initial_type, _), Delimiter::End(found_type, found_length))
                        | (Delimiter::Midpoint(initial_type, _), Delimiter::EitherStartOrEnd(found_type, found_length))
                            if found_type == *initial_type => {

                            dbg!(&initial_delimiter, found_delimeter);
                            // found_delimeter is a valid match! This is the end of the match
                            // process. If inclusive though, advance to the end of the found match.
                            if inclusive == Inclusivity::Inclusive {
                                mode = Mode::StopOnNextChar;
                                CursorSeek::AdvanceByCharCount(found_length)
                            } else {
                                CursorSeek::Stop
                            }
                        },
                        (Delimiter::End(_, _), _) => {
                            unimplemented!("mode if Mode::FindingDelimiterSeekingForwards when initial_delimiter is Delimiter::End! This should be impossible.");
                        },

                        // The found delimeter doesn't fit with the given traversal, so skip it.
                        _ => CursorSeek::Continue,
                    }
                },
                Mode::FindingDelimiterSeekingBackwards(buffer, initial_delimiter) => {
                    // No delimiter found, add to buffer and keep going
                    let mut buffer_copy = buffer.clone();
                    buffer_copy.push(c);
                    if buffer_copy.len() > DELIMITER_LOOKBACK_BUFFER_LENGTH_CHARS {
                        buffer_copy.remove(0);
                    };

                    let Some(found_delimeter) = is_delimiter(&buffer_copy[..]) else {
                        // No delimiter found, add to buffer and keep going
                        mode = Mode::FindingDelimiterSeekingBackwards(buffer_copy, initial_delimiter.clone());
                        return CursorSeek::Continue;
                    };

                    match (initial_delimiter, found_delimeter) {
                        (
                            Delimiter::End(initial_type, _),
                            Delimiter::Start(found_type, found_length)
                        ) if found_type == *initial_type => {

                            // found_delimeter is a valid match! This is the end of the match
                            // process. If inclusive though, advance to the start of the found match.
                            if inclusive == Inclusivity::Inclusive {
                                mode = Mode::StopOnNextChar;
                                CursorSeek::AdvanceByCharCount(found_length)
                            } else {
                                CursorSeek::Stop
                            }
                        },
                        (Delimiter::End(_, _), _) => {
                            unimplemented!("mode if Mode::FindingDelimiterSeekingForwards when initial_delimiter is Delimiter::End! This should be impossible.");
                        },
                        (Delimiter::End(_, _), _) => {
                            // The Seek needs to happen in the other direction to go back to start? hmm
                            CursorSeek::ChangeDirection(Direction::Backwards)
                        },

                        // The found delimeter doesn't fit with the given traversal, so skip it.
                        _ => CursorSeek::Continue,
                    }
                },
                Mode::StopOnNextChar => CursorSeek::Stop,
            }
        })
    }
}
