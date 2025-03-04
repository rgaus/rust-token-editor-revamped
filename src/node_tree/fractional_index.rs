use std::{
    collections::VecDeque,
    fmt::{Debug, Display},
    iter::zip,
};

fn get_midpoint_u8(smaller: u8, larger: u8) -> u8 {
    if smaller == larger || smaller + 1 == larger {
        // 5 and 5, or 5 and 6
        smaller
    } else if smaller + 2 == larger {
        // 5 and 7
        smaller + 1
    } else if smaller == u8::MIN && larger == u8::MAX {
        u8::MAX / 8
    } else {
        (smaller / 2) + (larger / 2)
    }
}

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
    pub fn of(n: usize) -> Self {
        Self(n)
    }

    pub fn generate(previous: Self, next: Self) -> Self {
        let new_value = (previous.0 / 2) + (next.0 / 2);
        assert_ne!(
            previous.0, new_value,
            "FractionalIndex: ran out of precision to represent new entry!"
        );
        assert_ne!(
            next.0, new_value,
            "FractionalIndex: ran out of precision to represent new entry!"
        );
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

    /// Generates a roughly equally distributed sequence of `sequence_length` indexes
    /// between `start` and `end`.
    ///
    /// This is "roughly equally distributed" because the index is backed by a usize, which means
    /// rounding may result in off by one errors in spacings.
    ///
    /// Note that the output sequence does not include `start` or `end`, just the in between values.
    pub fn distributed_sequence(
        start: &Self,
        end: &Self,
        sequence_length: usize,
    ) -> impl std::iter::Iterator<Item = Self> {
        if sequence_length == 0 {
            return vec![].into_iter();
        };

        let multiplier = (end.0 - start.0) / sequence_length;

        let result = (0..sequence_length).map(|index| Self::of(start.0 + (index * multiplier)));

        for (a, b) in zip(result.clone(), result.clone().skip(1)) {
            assert_ne!(a, b, "FractionalIndex::distributed_sequence: ran out of precision to represent new entry!");
        }

        result.collect::<Vec<Self>>().into_iter()
    }

    /// Given node values for a next and previous that may or may not exist, generate a midpoint
    /// node value to assign to a node in this new position.
    pub fn distributed_sequence_or_fallback(
        start: Option<Self>,
        end: Option<Self>,
        sequence_length: usize,
    ) -> impl std::iter::Iterator<Item = Self> {
        match (start, end) {
            (None, None) => {
                Self::distributed_sequence(&Self::start(), &Self::end(), sequence_length)
            }
            (None, Some(end)) => Self::distributed_sequence(&Self::start(), &end, sequence_length),
            (Some(start), None) => {
                Self::distributed_sequence(&start, &Self::end(), sequence_length)
            }
            (Some(start), Some(end)) => Self::distributed_sequence(&start, &end, sequence_length),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Ord)]
pub struct VariableSizeFractionalIndex(Vec<u8>);

impl Display for VariableSizeFractionalIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // let places = u8::MAX.checked_ilog(8).unwrap();
        let entries = self
            .0
            .iter()
            .map(|entry| format!("{}", entry))
            .collect::<Vec<String>>()
            .join(",");
        write!(f, "{entries}")
    }
}

impl Debug for VariableSizeFractionalIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // let places = u8::MAX.checked_ilog(8).unwrap();
        let entries = self
            .0
            .iter()
            .map(|entry| format!("{}", entry))
            .collect::<Vec<String>>()
            .join(",");
        write!(f, "VariableSizeFractionalIndex({entries})")
    }
}

impl PartialOrd for VariableSizeFractionalIndex {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let longer_length = if self.0.len() < other.0.len() {
            other.0.len()
        } else {
            self.0.len()
        };

        for index in 0..longer_length {
            let a = self.0.get(index).unwrap_or(&0);
            let b = other.0.get(index).unwrap_or(&0);
            if a == b {
                continue;
            } else if a < b {
                return Some(std::cmp::Ordering::Less);
            } else if a > b {
                return Some(std::cmp::Ordering::Greater);
            }
        }

        Some(std::cmp::Ordering::Equal)
    }
}

impl VariableSizeFractionalIndex {
    pub fn start() -> Self {
        Self::of(vec![0])
    }
    pub fn end() -> Self {
        Self::of(vec![255])
    }
    pub fn of(raw: Vec<u8>) -> Self {
        Self(raw)
    }

    pub fn generate(previous: &Self, next: &Self) -> Self {
        let (shorter_length, longer_length) = if previous.0.len() < next.0.len() {
            (previous.0.len(), next.0.len())
        } else {
            (next.0.len(), previous.0.len())
        };

        for index in (0..shorter_length + 1).rev() {
            let previous_ancestry = &previous.0[..index];
            let next_ancestry = &next.0[..index];
            if previous_ancestry != next_ancestry {
                continue;
            }

            let mut result = previous_ancestry.to_vec();
            for secondary_index in 0..longer_length {
                let previous_tail = previous.0.get(index + secondary_index).unwrap_or(&u8::MIN);
                let next_tail = next.0.get(index + secondary_index).unwrap_or(&u8::MAX);

                let new_tail = get_midpoint_u8(*previous_tail, *next_tail);
                result.push(new_tail);
            }
            // If the new generated value is the same as the previous (ie, maybe trying to pick a
            // number between 5 and 6), then add a 127 as another place on the end.
            if result.len() == previous.0.len()
                && previous
                    .0
                    .iter()
                    .enumerate()
                    .all(|(index, n)| *n == result[index])
            {
                result.push(get_midpoint_u8(u8::MIN, u8::MAX));
            }
            return Self(result);
        }

        unreachable!("VariableSizeFractionalIndex: could not find a common ansestry between previous and next, even a zero length common ancestry? This should not be possible.");
    }

    /// Given node values for a next and previous that may or may not exist, generate a midpoint
    /// node value to assign to a node in this new position.
    pub fn generate_or_fallback(previous: Option<Self>, next: Option<Self>) -> Self {
        match (previous, next) {
            (None, None) => Self::start(),
            (None, Some(next)) => Self::generate(&Self::start(), &next),
            (Some(previous), None) => Self::generate(&previous, &Self::end()),
            (Some(previous), Some(next)) => Self::generate(&previous, &next),
        }
    }

    /// Generates a roughly equally distributed sequence of `sequence_length` indexes
    /// between `start` and `end`.
    ///
    /// This is "roughly equally distributed" because it is implemented in a binary search type
    /// fashion, where each division between existing nodes is divided repeatedly to acheive the
    /// output. This means some nodes may be further apart because a division between them was not
    /// required to get to the reqested sequence length.
    ///
    /// Note that the output sequence does not include `start` or `end`, just the in between values.
    pub fn distributed_sequence(
        start: &Self,
        end: &Self,
        sequence_length: usize,
    ) -> impl std::iter::Iterator<Item = Self> {
        let mut sequence = VecDeque::from(vec![start.clone(), end.clone()]);
        while sequence.len() < sequence_length {
            // Generate a new element between each existing element
            let mut start_index = 0;
            let mut end_index = 1;
            while end_index < sequence.len() {
                // println!("{}, {}", start_index, end_index);
                let start = &sequence[start_index];
                let end = &sequence[end_index];
                let midpoint = Self::generate(start, end);
                sequence.insert(start_index + 1, midpoint);
                start_index += 2 /* each element of pair */ + 1 /* newly added element */;
                end_index += 2 /* each element of pair */ + 1 /* newly added element */;
            }
            // println!("SEQ: {:?}", sequence);
        }

        // Remove the original start and end elements
        sequence.pop_front();
        sequence.pop_back();

        sequence.into_iter()
    }

    /// Given node values for a next and previous that may or may not exist, generate a midpoint
    /// node value to assign to a node in this new position.
    pub fn distributed_sequence_or_fallback(
        start: Option<Self>,
        end: Option<Self>,
        sequence_length: usize,
    ) -> impl std::iter::Iterator<Item = Self> {
        match (start, end) {
            (None, None) => {
                Self::distributed_sequence(&Self::start(), &Self::end(), sequence_length)
            }
            (None, Some(end)) => Self::distributed_sequence(&Self::start(), &end, sequence_length),
            (Some(start), None) => {
                Self::distributed_sequence(&start, &Self::end(), sequence_length)
            }
            (Some(start), Some(end)) => Self::distributed_sequence(&start, &end, sequence_length),
        }
    }
}
