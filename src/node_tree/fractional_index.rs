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


#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct VariableSizeFractionalIndex(Vec<u8>);

impl Display for VariableSizeFractionalIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // let places = u8::MAX.checked_ilog(8).unwrap();
        let entries = self.0.iter()
            .map(|entry| format!("{:0>8}", entry))
            .collect::<Vec<String>>()
            .join(",");
        write!(f, "VariableSizeFractionalIndex({entries})")
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
        dbg!(shorter_length);

        for index in (0..shorter_length+1).rev() {
            let previous_ancestry = &previous.0[..index];
            let next_ancestry = &next.0[..index];
            println!("ancestry: {index} {previous_ancestry:?} {next_ancestry:?}");
            if previous_ancestry != next_ancestry {
                continue;
            }

            let previous_lead = previous.0.get(index).unwrap_or(&0u8);
            let next_lead = next.0.get(index).unwrap_or(&0u8);

            let previous_tail = previous.0[index..].last().unwrap_or(&0u8);
            let next_tail = next.0[index..].last().unwrap_or(&0u8);
            println!("prev_lead={previous_lead} next_lead={next_lead} previous_tail={previous_tail} next_tail={next_tail}");

            // The new lead should be equidistant between previous_lead and next_lead
            let new_lead = average_u8(*previous_lead, *next_lead);
            dbg!(new_lead);

            // And the new tail should be 0 ...
            let mut before_tail = vec![];
            let mut new_tail = 0;
            if new_lead == *previous_lead {
                // ... unless the new_lead is the same as previous_lead, which in practice only
                // happens if the previous_lead is (for example) 5 and the next_lead is (for
                // example) 6, so there's no space in between to fit in another number.
                //
                // So, in this case, push the tail forward to offset it into its midpoint ...
                new_tail = u8::MAX / 2;

                let new_tail_before_previous_tail = *previous_tail >= new_tail;
                let new_tail_after_next_tail = next_lead == previous_lead && new_tail >= *next_tail;
                if new_tail_before_previous_tail || new_tail_after_next_tail {
                    // ... unless this hasn't pushed it forward enough, because the previous_tail is
                    // already beyond this new_tail point. In this case, find the midpoint between 
                    new_tail = average_u8(*previous_tail, *next_tail);

                    if new_tail == *previous_tail {
                        before_tail.push(new_tail);
                        new_tail = u8::MAX / 2;
                    }
                }
            }

            println!("NEW LEAD: {} NEW TAIL: {}", new_lead, new_tail);
            // [..., new_lead, new_tail]
            let mut result = previous_ancestry.to_vec();
            result.push(new_lead);
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
