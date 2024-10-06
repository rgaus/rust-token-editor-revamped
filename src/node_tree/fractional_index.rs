use std::fmt::Display;

/// A FractionalIndex is a mechanism for defining order of elements that allows multiple concurrent
/// or conflicting edits to come in and the system to self heal.
///
/// More info: https://gist.github.com/wolever/3c3fa1f23a7e2e19dcb39e74af3d9282
///
/// Note that currently these are backed by a `usize`. In the future, backing this with a string
/// that can vary in length would probably be a better idea, because there's only sqrt(usize::MAX)
/// (~3B) possible insertions that can be made before precision issues could potentially arrise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FractionalIndex(usize);

impl Display for FractionalIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // let places = usize::MAX.checked_ilog(8).unwrap();
        write!(f, "FractionalIndex({:0>20})", self.0)
    }
}

impl FractionalIndex {
    pub fn start() -> Self {
        Self(usize::MIN)
    }
    pub fn end() -> Self {
        Self(usize::MAX)
    }

    pub fn generate(previous: Self, next: Self) -> Self {
        let new_value = (previous.0 / 2) + (next.0 / 2);
        assert_ne!(previous.0, new_value, "FractionalIndex: ran out of precision to represent new entry!");
        assert_ne!(next.0, new_value, "FractionalIndex: ran out of precision to represent new entry!");
        Self(new_value)
    }

    /// Given node values for a next and previous that may or may not exist, generate a midpoint
    /// node value to assign to a node in this new position.
    pub fn generate_or_fallback(previous: Option<Self>, next: Option<Self>) -> Self {
        match (previous, next) {
            (None, None) => Self::start(),
            (None, Some(next)) => Self::generate(Self::start(), next),
            (Some(previous), None) => Self::generate(previous, Self::end()),
            (Some(previous), Some(next)) => Self::generate(previous, next),
        }
    }
}
