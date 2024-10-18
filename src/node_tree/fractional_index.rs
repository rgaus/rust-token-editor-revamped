use std::fmt::{Display, Debug};

fn get_midpoint_u8(smaller: u8, larger: u8) -> u8 {
    if smaller == larger || smaller+1 == larger {
        // 5 and 5, or 5 and 6
        smaller
    } else if smaller+2 == larger {
        // 5 and 7
        smaller+1
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


#[derive(Clone, PartialEq, Eq, Ord)]
pub struct VariableSizeFractionalIndex(Vec<u8>);

impl Display for VariableSizeFractionalIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // let places = u8::MAX.checked_ilog(8).unwrap();
        let entries = self.0.iter()
            .map(|entry| format!("{}", entry))
            .collect::<Vec<String>>()
            .join(",");
        write!(f, "{entries}")
    }
}

impl Debug for VariableSizeFractionalIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // let places = u8::MAX.checked_ilog(8).unwrap();
        let entries = self.0.iter()
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

        for index in (0..shorter_length+1).rev() {
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
            };
            // If the new generated value is the same as the previous (ie, maybe trying to pick a
            // number between 5 and 6), then add a 127 as another place on the end.
            if result.len() == previous.0.len() && previous.0.iter().enumerate().all(|(index, n)| *n == result[index]) {
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
}
