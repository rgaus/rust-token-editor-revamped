#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Inclusivity {
    Inclusive,
    Exclusive,
}

#[derive(Debug, Clone, Copy)]
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
