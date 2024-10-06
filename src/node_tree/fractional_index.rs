use std::fmt::Display;

fn average_u8(smaller: u8, larger: u8) -> u8 {
    if smaller == larger || smaller+1 == larger {
        // 5 and 5, or 5 and 6
        smaller
    } else if smaller+2 == larger {
        // 5 and 7
        smaller+1
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


#[derive(Debug, Clone, PartialEq, Eq, Ord)]
pub struct VariableSizeFractionalIndex(Vec<u8>);

impl Display for VariableSizeFractionalIndex {
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

    pub fn generate(previous: Self, next: Self) -> Self {
        let (shorter_length, longer_length) = if previous.0.len() < next.0.len() {
            (previous.0.len(), next.0.len())
        } else {
            (next.0.len(), previous.0.len())
        };
        // dbg!(shorter_length);

        for index in (0..shorter_length+1).rev() {
            let previous_ancestry = &previous.0[..index];
            let next_ancestry = &next.0[..index];
            // println!("ancestry: {index} {previous_ancestry:?} {next_ancestry:?}");
            if previous_ancestry != next_ancestry {
                continue;
            }

            // SPECIAL CASE: if the first values after the shared ancestry are not equal, then
            // make the generated index in between these unequal values. This isn't the _exact_
            // center but doing this optimizes for smaller indexes at the expense of them being a
            // little less well distributed among the entire range.
            let previous_head = previous.0.get(index);
            let next_head = next.0.get(index);
            if let (Some(previous_head), Some(next_head)) = (previous_head, next_head) {
                if previous_head != next_head {
                    let mut result = previous.0.clone();
                    let value = average_u8(*previous_head, *next_head);
                    result.pop();
                    result.push(if value == *previous_head {
                        u8::MAX / 2
                    } else {
                        value
                    });
                    return Self(result);
                };
            };

            let previous_tail = previous.0[index..].last().unwrap_or(&0u8);
            let next_tail = next.0[index..].last().unwrap_or(&0u8);

            let mut before_tail = vec![];
            // The new tail should be equidistant between previous_tail and next_tail
            let mut new_tail = average_u8(*previous_tail, *next_tail);
            loop {
                // println!("---");
                // println!("previous_tail={previous_tail} next_tail={next_tail}");
                // println!("BEFORE: {:?} NEW TAIL: {}", before_tail, new_tail);
                if before_tail.len() >= longer_length-index {
                    break;
                }

                let previous_tail = previous.0.get(index + before_tail.len()).unwrap_or(&0u8);
                let next_tail = next.0.get(index + before_tail.len()).unwrap_or(&0u8);

                // ... unless the new_tail is the same as previous_tail, which in practice only
                // happens if the previous_tail is (for example) 5 and the next_lead is (for
                // example) 6, so there's no space in between to fit in another number.
                //
                // So, in this case, push the tail forward to offset it into its midpoint ...
                before_tail.push(average_u8(*previous_tail, *next_tail));
                new_tail = u8::MAX / 2;

                let new_tail_before_previous_tail = *previous_tail >= new_tail;
                let new_tail_after_next_tail = next_tail == previous_tail && new_tail >= *next_tail;
                if new_tail_before_previous_tail || new_tail_after_next_tail {
                    // ... unless this hasn't pushed it forward enough, because the previous_tail is
                    // already beyond this new_tail point. In this case, find the midpoint between 
                    new_tail = average_u8(*previous_tail, *next_tail);

                    if new_tail == *previous_tail {
                        continue;
                    }
                }
            };

            // [..., new_lead, new_tail]
            let mut result = previous_ancestry.to_vec();
            // println!("RESULT: {:?} BEFORE: {:?} NEW TAIL: {}", result, before_tail, new_tail);
            for n in before_tail.into_iter() {
                result.push(n);
            }
            if new_tail > 0 {
                result.push(new_tail);
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
            (None, Some(next)) => Self::generate(Self::start(), next),
            (Some(previous), None) => Self::generate(previous, Self::end()),
            (Some(previous), Some(next)) => Self::generate(previous, next),
        }
    }
}
