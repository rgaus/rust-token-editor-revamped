#[derive(Debug, Clone, Copy)]
pub enum Inclusivity {
    Inclusive,
    Exclusive,
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Forwards,
    Backwards,
}

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
