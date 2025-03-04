use crate::node_tree::utils::{
    is_delimiter, is_lower_word_char, is_upper_word_char, vim_cls, Delimiter, Direction,
    Inclusivity, Newline, VimClass, DELIMITER_LOOKBACK_BUFFER_LENGTH_CHARS, NEWLINE,
};
use std::{cell::RefCell, rc::Rc};

// An enum used by seek_forwards_until to control how seeking should commence.
#[derive(Clone)]
pub enum CursorSeek {
    Continue,                  // Seek to the next character
    Stop,                      // Finish and don't include this character
    Done,                      // Finish and do include this character
    Fail(&'static str),        // The seek didn't occur successfully, throw an error
    AdvanceByCharCount(usize), // Advance by N chars before checking again
    AdvanceByLines(usize),     // Advance by N lines before checking again
    AdvanceUntil {
        // Advance until the given `until_fn` check passes
        until_fn: Rc<RefCell<dyn FnMut(char, CursorSeekContext) -> CursorSeek>>,
        // Optionally limit the operation to only run in a given direction
        only_in_direction: Option<Direction>,
    },
    ChangeDirection(Direction), // Change to seeking in the given direction
}

#[derive(Debug, Clone)]
pub struct CursorSeekContext {
    pub direction: Direction,
    pub index: usize,
    pub is_at_end_of_line: bool,
}

impl CursorSeek {
    pub fn advance_until<T>(until_fn: T) -> Self
    where
        T: FnMut(char, CursorSeekContext) -> CursorSeek + 'static,
    {
        CursorSeek::AdvanceUntil {
            only_in_direction: None,
            until_fn: Rc::new(RefCell::new(until_fn)),
        }
    }

    pub fn advance_until_only<T>(direction: Direction, until_fn: T) -> Self
    where
        T: FnMut(char, CursorSeekContext) -> CursorSeek + 'static,
    {
        CursorSeek::AdvanceUntil {
            only_in_direction: Some(direction),
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

    /// forwards_word(count, is_big_word, is_eol) - move forward `count` words
    ///
    /// If is_eol is TRUE, last word stops at end of line (for operators).
    ///
    /// ref: https://github.com/JimZhouZZY/vim/blob/20df5aa89983c5c89a99c83e5837275d7a8c7137/src/textobject.c#L361
    pub fn forwards_word(count: usize, is_big_word: bool, _is_eol: bool) -> Self {
        #[derive(Debug)]
        enum Mode {
            Initial,
            One(VimClass),
            Two,
        }
        let mut mode = Mode::Initial;
        let mut loop_count = count;

        // From :h word -
        // A word consists of a sequence of letters, digits and underscores, or a
        // sequence of other non-blank characters, separated with white space (spaces,
        // tabs, <EOL>).  This can be changed with the 'iskeyword' option.  An empty line
        // is also considered to be a word.
        CursorSeek::advance_until(move |c, _ctx| {
            match &mode {
                Mode::Initial => {
                    let starting_class = vim_cls(c, is_big_word);
                    mode = Mode::One(starting_class);

                    /*
                     * We always move at least one character, unless on the last
                     * character in the buffer.
                     */
                    // last_line = (curwin->w_cursor.lnum == curbuf->b_ml.ml_line_count);
                    CursorSeek::Continue
                    // i = inc_cursor();
                    // if (i == -1 || (i >= 1 && last_line)) // started at last char in file
                    //     return FAIL;
                    // if (i >= 1 && eol && count == 0)      // started at last char in line
                    //     return OK;
                }

                /*
                 * Go one char past end of current word (if any)
                 */
                Mode::One(starting_class) => {
                    let current_class = vim_cls(c, is_big_word);
                    println!(
                        "1. c={c} starting_class={starting_class} current_class={current_class}"
                    );

                    if *starting_class != 0 && *starting_class == current_class {
                        mode = Mode::Two;
                        CursorSeek::Continue
                    } else {
                        CursorSeek::Continue
                        // if (i == -1 || (i >= 1 && eol && count == 0))
                        //     return OK;
                    }
                }

                /*
                 * go to next non-white
                 */
                Mode::Two => {
                    let current_class = vim_cls(c, is_big_word);

                    /*
                     * We'll stop if we land on a blank line
                     */
                    // if (curwin->w_cursor.col == 0 && *ml_get_curline() == NUL)
                    // break;

                    if current_class == 0 {
                        println!("space char!");

                        // Loop around again if there are still iterations to go
                        loop_count -= 1;
                        if loop_count > 0 {
                            mode = Mode::Initial;
                            CursorSeek::Continue
                        } else {
                            CursorSeek::Done
                        }
                    } else {
                        CursorSeek::Continue
                    }
                }
            }
        })
    }

    /// back_word() - move backward 1 word
    /// If stop is TRUE and we are already on the start of a word, move one less.
    ///
    /// ref: https://github.com/JimZhouZZY/vim/blob/20df5aa89983c5c89a99c83e5837275d7a8c7137/src/textobject.c#L431
    pub fn back_word(count: usize, is_big_word: bool, stop: bool) -> Self {
        #[derive(Debug)]
        enum Mode {
            Initial,
            One(VimClass),
            Two,
            Three(VimClass),
            Four,
            Five,
        }
        let mut mode = Mode::Initial;
        let mut loop_count = count;
        let mut stop_value = stop;

        // From :h word -
        // A word consists of a sequence of letters, digits and underscores, or a
        // sequence of other non-blank characters, separated with white space (spaces,
        // tabs, <EOL>).  This can be changed with the 'iskeyword' option.  An empty line
        // is also considered to be a word.
        // CursorSeek::advance_until_only(Direction::Forwards, move |c, _i| {
        CursorSeek::advance_until(move |c, _i| {
            match &mode {
                Mode::Initial => {
                    let starting_class = vim_cls(c, is_big_word);

                    // Always advance at least one char
                    mode = Mode::One(starting_class);
                    CursorSeek::Continue
                }
                Mode::One(starting_class) => {
                    let current_class = vim_cls(c, is_big_word);

                    println!("1. c={c} starting_class={starting_class:?} current_class={current_class:?}");
                    if !stop_value || *starting_class == current_class || *starting_class == 0 {
                        // if (curwin->w_cursor.col == 0
                        //               && LINEEMPTY(curwin->w_cursor.lnum))
                        //     goto finished;
                        mode = Mode::Two;
                        CursorSeek::Continue
                    } else {
                        // overshot - forward one
                        mode = Mode::Four;
                        CursorSeek::Continue
                    }
                }

                /*
                 * Skip white space before the word.
                 * Stop on an empty line.
                 */
                Mode::Two => {
                    let current_class = vim_cls(c, is_big_word);
                    println!("2. c={c} current_class={current_class:?}");

                    if current_class == 0 {
                        // if (curwin->w_cursor.col == 0
                        //               && LINEEMPTY(curwin->w_cursor.lnum))
                        //     goto finished;
                        CursorSeek::Continue
                        // if (dec_cursor() == -1) // hit start of file, stop here
                        //     return OK;
                        // }

                        /*
                         * Move backward to start of this word.
                         */
                        // if (skip_chars(cls(), BACKWARD))
                        // return OK;
                    } else {
                        mode = Mode::Three(current_class);
                        CursorSeek::Continue
                    }
                }

                /*
                 * Move backward to start of this word.
                 */
                Mode::Three(previous_class) => {
                    // if (skip_chars(cls(), BACKWARD))
                    // return OK;
                    let current_class = vim_cls(c, is_big_word);
                    println!("3. c={c} current_class={current_class:?}");

                    if current_class == *previous_class {
                        CursorSeek::Continue
                    } else {
                        mode = Mode::Four;
                        CursorSeek::Continue
                    }
                }

                // overshot - forward one
                Mode::Four => {
                    println!("4. c={c}");
                    mode = Mode::Five;
                    CursorSeek::ChangeDirection(Direction::Forwards)
                }
                Mode::Five => {
                    println!("5. c={c}");

                    stop_value = false;

                    // Loop around again if there are still iterations to go
                    loop_count -= 1;
                    if loop_count > 0 {
                        mode = Mode::Initial;
                        CursorSeek::AdvanceByCharCount(1)
                    } else {
                        CursorSeek::Done
                    }
                }
            }
        })
    }

    pub fn advance_upper_word() -> Self {
        let mut hit_word_char = false;

        // From :h WORD -
        // A WORD consists of a sequence of non-blank characters, separated with white
        // space.  An empty line is also considered to be a WORD.
        CursorSeek::advance_until(move |c, _i| {
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
                CursorSeek::Done
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
        CursorSeek::advance_until_only(Direction::Forwards, move |c, _i| {
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
        CursorSeek::advance_until_only(Direction::Forwards, move |c, _i| {
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

    /// When called, advance forwards to the start of the line - this implements '$'!
    ///
    /// NOTE: ONLY WORKS WHEN SEEKING FORWARDS!
    pub fn advance_until_line_end() -> Self {
        CursorSeek::advance_until_only(Direction::Forwards, move |c, _i| {
            if c == *NEWLINE {
                CursorSeek::Done
            } else {
                CursorSeek::Continue
            }
        })
    }

    /// When called, advance backwards to the start of the line - this implements '0'!
    ///
    /// NOTE: ONLY WORKS WHEN SEEKING BACKWARDS!
    pub fn advance_until_line_start() -> Self {
        CursorSeek::advance_until_only(Direction::Backwards, move |c, _i| {
            if c == *NEWLINE {
                CursorSeek::Stop
            } else {
                CursorSeek::Continue
            }
        })
    }

    /// When called, advance backwards to the start of the line, not taking into account leading
    /// whitespace - this implements '^'!
    ///
    /// NOTE: ONLY WORKS WHEN SEEKING BACKWARDS!
    pub fn advance_until_line_start_after_leading_whitespace() -> Self {
        let mut hit_start_of_line = false;
        CursorSeek::advance_until_only(Direction::Backwards, move |c, _i| {
            if !hit_start_of_line {
                if c != *NEWLINE {
                    // 1. Seek backwards until the newline boundary is hit
                    CursorSeek::Continue
                } else {
                    // 2. Newline was hit, now start seeking forward
                    hit_start_of_line = true;
                    CursorSeek::ChangeDirection(Direction::Forwards)
                }
            } else {
                // 3. Stop seeking forward once the first non-whitespace char is hit
                if c.is_whitespace() {
                    CursorSeek::Continue
                } else {
                    CursorSeek::Stop
                }
            }
        })
    }

    /// When called, advanced to the start or end of a document
    pub fn advance_until_start_end() -> Self {
        CursorSeek::advance_until(|_c, _i| CursorSeek::Continue)
    }

    /// When called, advance to the next delimeter. See :help % for an outline of the behavior
    ///
    /// NOTE: ONLY WORKS WHEN SEEKING FORWARDS!
    pub fn advance_until_matching_delimiter(inclusive: Inclusivity) -> Self {
        #[derive(Debug)]
        enum Mode {
            FindingInitialDelimeter(Vec<char>),
            FindingDelimiterSeekingForwards(Vec<char>, Delimiter),
            FindingDelimiterSeekingBackwards(Vec<char>, Delimiter),
            StopOnNextChar,
        }
        let mut mode = Mode::FindingInitialDelimeter(vec![]);

        CursorSeek::advance_until_only(Direction::Forwards, move |c, _i| {
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
                }
                Mode::FindingDelimiterSeekingForwards(buffer, initial_delimiter) => {
                    // No delimiter found, add to buffer and keep going
                    let mut buffer_copy = buffer.clone();
                    buffer_copy.push(c);
                    if buffer_copy.len() > DELIMITER_LOOKBACK_BUFFER_LENGTH_CHARS {
                        buffer_copy.remove(0);
                    };

                    let Some(found_delimeter) = is_delimiter(&buffer_copy[..]) else {
                        // No delimiter found, add to buffer and keep going
                        mode = Mode::FindingDelimiterSeekingForwards(
                            buffer_copy,
                            initial_delimiter.clone(),
                        );
                        return CursorSeek::Continue;
                    };

                    match (initial_delimiter, found_delimeter.clone()) {
                        (
                            Delimiter::Start(initial_type, _),
                            Delimiter::Midpoint(found_type, found_length),
                        )
                        | (
                            Delimiter::Start(initial_type, _),
                            Delimiter::End(found_type, found_length),
                        )
                        | (
                            Delimiter::Start(initial_type, _),
                            Delimiter::EitherStartOrEnd(found_type, found_length),
                        )
                        | (
                            Delimiter::EitherStartOrEnd(initial_type, _),
                            Delimiter::Midpoint(found_type, found_length),
                        )
                        | (
                            Delimiter::EitherStartOrEnd(initial_type, _),
                            Delimiter::End(found_type, found_length),
                        )
                        | (
                            Delimiter::EitherStartOrEnd(initial_type, _),
                            Delimiter::EitherStartOrEnd(found_type, found_length),
                        )
                        | (
                            Delimiter::Midpoint(initial_type, _),
                            Delimiter::End(found_type, found_length),
                        )
                        | (
                            Delimiter::Midpoint(initial_type, _),
                            Delimiter::EitherStartOrEnd(found_type, found_length),
                        ) if found_type == *initial_type => {
                            // found_delimeter is a valid match! This is the end of the match
                            // process. If inclusive though, advance to the end of the found match.
                            if inclusive == Inclusivity::Inclusive {
                                mode = Mode::StopOnNextChar;
                                CursorSeek::AdvanceByCharCount(
                                    found_length - 1, /* the cursor width */
                                )
                            } else {
                                CursorSeek::Stop
                            }
                        }
                        (Delimiter::End(_, _), _) => {
                            unimplemented!("mode if Mode::FindingDelimiterSeekingForwards when initial_delimiter is Delimiter::End! This should be impossible.");
                        }

                        // The found delimeter doesn't fit with the given traversal, so skip it.
                        _ => CursorSeek::Continue,
                    }
                }
                Mode::FindingDelimiterSeekingBackwards(buffer, initial_delimiter) => {
                    // No delimiter found, add to buffer and keep going
                    let mut buffer_copy = buffer.clone();
                    buffer_copy.push(c);
                    if buffer_copy.len() > DELIMITER_LOOKBACK_BUFFER_LENGTH_CHARS {
                        buffer_copy.remove(0);
                    };

                    let Some(found_delimeter) = is_delimiter(&buffer_copy[..]) else {
                        // No delimiter found, add to buffer and keep going
                        mode = Mode::FindingDelimiterSeekingBackwards(
                            buffer_copy,
                            initial_delimiter.clone(),
                        );
                        return CursorSeek::Continue;
                    };

                    match (initial_delimiter, found_delimeter) {
                        (
                            Delimiter::End(initial_type, _),
                            Delimiter::Start(found_type, found_length),
                        )
                        | (
                            Delimiter::End(initial_type, _),
                            Delimiter::EitherStartOrEnd(found_type, found_length),
                        ) if found_type == *initial_type => {
                            // found_delimeter is a valid match! This is the end of the match
                            // process. If inclusive though, advance to the start of the found match.
                            if inclusive == Inclusivity::Inclusive {
                                mode = Mode::StopOnNextChar;
                                CursorSeek::AdvanceByCharCount(found_length)
                            } else {
                                CursorSeek::Stop
                            }
                        }
                        (initial_delimeter, _)
                            if !matches!(initial_delimeter, Delimiter::End(_, _)) =>
                        {
                            unimplemented!("mode if Mode::FindingDelimiterSeekingForwards when initial_delimiter is NOT Delimiter::End! This should be impossible.");
                        }

                        // The found delimeter doesn't fit with the given traversal, so skip it.
                        _ => CursorSeek::Continue,
                    }
                }
                Mode::StopOnNextChar => CursorSeek::Stop,
            }
        })
    }
}
