use super::cursor::CursorSeek;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Inclusivity {
    Inclusive,
    Exclusive,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Forwards,
    Backwards,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Newline {
    Ignore,
    ShouldTerminate,
}

pub const NEWLINE: &'static char = &'\n';

const CHAR_OF_VALUE_255: &'static char = &(255 as char);

/// Returns true if the given char is a lower word char according to stock vim definitions
/// See :h word for more info.
/// In my vim instance, `:set iskeyword?` is `@,48-57,_,192-255` - so this follows those rules.
pub fn is_lower_word_char(c: char) -> bool {
    c > *CHAR_OF_VALUE_255
        || (c >= '0' && c <= '9')
        || c == '_'
        || (c >= 'A' && c <= *CHAR_OF_VALUE_255)
}

/// Returns true if the given char is a upper word char according to stock vim definitions
/// See :h WORD for more info.
/// In my vim instance, `:set iskeyword?` is `@,48-57,_,192-255` - so this follows those rules.
pub fn is_upper_word_char(c: char) -> bool {
    !c.is_whitespace()
}

// ------------------------------------------------------------------------------------------------
// This delimiter related stuff is used to implement %-type actions
// Delimeters are chars like ([{}])
#[derive(Debug, Clone, PartialEq)]
pub enum DelimiterType {
    SingleQuote,
    DoubleQuote,
    Parenthesis,
    Square,
    Curly,
    CMultiLineComment,
    CPreprocesserConditional,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Delimiter {
    Start(DelimiterType, usize),
    End(DelimiterType, usize),
    EitherStartOrEnd(DelimiterType, usize),
    Midpoint(DelimiterType, usize), // ie, like #elif
}
pub const DELIMITER_LOOKBACK_BUFFER_LENGTH_CHARS: usize = 6;
pub fn is_delimiter(buffer: &[char]) -> Option<Delimiter> {
    match buffer {
        // Single char delimeters
        [.., '\''] => Some(Delimiter::EitherStartOrEnd(DelimiterType::SingleQuote, 1)),
        [.., '"'] => Some(Delimiter::EitherStartOrEnd(DelimiterType::DoubleQuote, 1)),
        [.., '('] => Some(Delimiter::Start(DelimiterType::Parenthesis, 1)),
        [.., '['] => Some(Delimiter::Start(DelimiterType::Square, 1)),
        [.., '{'] => Some(Delimiter::Start(DelimiterType::Curly, 1)),
        [.., ')'] => Some(Delimiter::End(DelimiterType::Parenthesis, 1)),
        [.., ']'] => Some(Delimiter::End(DelimiterType::Square, 1)),
        [.., '}'] => Some(Delimiter::End(DelimiterType::Curly, 1)),

        // Multi char delimiters
        [.., '/', '*'] => Some(Delimiter::Start(DelimiterType::CMultiLineComment, 2)),
        [.., '*', '/'] => Some(Delimiter::End(DelimiterType::CMultiLineComment, 2)),

        [.., '#', 'i', 'f'] => Some(Delimiter::Start(DelimiterType::CPreprocesserConditional, 3)),
        [.., '#', 'i', 'f', 'd', 'e', 'f'] => Some(Delimiter::Start(DelimiterType::CPreprocesserConditional, 6)),
        [.., '#', 'e', 'l', 's', 'e'] => Some(Delimiter::Midpoint(DelimiterType::CPreprocesserConditional, 5)),
        [.., '#', 'e', 'l', 'i', 'f'] => Some(Delimiter::Midpoint(DelimiterType::CPreprocesserConditional, 5)),
        [.., '#', 'e', 'n', 'd', 'i', 'f'] => Some(Delimiter::End(DelimiterType::CPreprocesserConditional, 6)),

        _ => None,
    }
}

pub type VimClass = usize;
/*
 * cls() - returns the class of character at curwin->w_cursor
 *
 * If a 'W', 'B', or 'E' motion is being done (cls_bigword == TRUE), chars
 * from class 2 and higher are reported as class 1 since only white space
 * boundaries are of interest.
 */
pub fn vim_cls(c: char, cls_bigword: bool) -> VimClass {
    if c == ' ' || c == '\t' || c == '\0' {
        0
    } else if c.len_utf8() > 1 {
        // If cls_bigword, report multi-byte chars as class 1.
        if cls_bigword {
            1
        } else {
            // process code leading/trailing bytes
            // TODO: https://github.com/vim/vim/blob/659cb28c25b756e59c712c337f8b4650e85f8bcd/src/mbyte.c#L863
            // return dbcs_class(
            //     ((unsigned)c >> 8), (c & 0xFF)
            // );
            3
        }
    // } else if (enc_utf8) {
    //     // TODO: https://github.com/vim/vim/blob/659cb28c25b756e59c712c337f8b4650e85f8bcd/src/mbyte.c#L2901
    //     let c = 2 // utf_class(c);
    //     if c != 0 && cls_bigword {
    //         1
    //     } else {
    //         c
    //     }
    } else if (cls_bigword) {
        // If cls_bigword is TRUE, report all non-blanks as class 1.
        1
    } else if (is_lower_word_char(c)) {
        2
    } else {
        1
    }
}
