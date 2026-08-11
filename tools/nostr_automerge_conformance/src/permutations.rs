#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SignedPermutation<T> {
    pub(crate) name: &'static str,
    pub(crate) events: Vec<T>,
}

pub(crate) fn required_delivery_permutations<T: Clone>(
    canonical: &[T],
    is_dependency: impl Fn(&T) -> bool,
    is_control: impl Fn(&T) -> bool,
    is_invalid: impl Fn(&T) -> bool,
) -> Vec<SignedPermutation<T>> {
    let mut reverse = canonical.to_vec();
    reverse.reverse();
    let mut seed_zero = canonical.to_vec();
    shuffle(&mut seed_zero, 0);
    let mut seed_fixed = canonical.to_vec();
    shuffle(&mut seed_fixed, 0x5eed);
    let mut duplicate_heavy = Vec::with_capacity(canonical.len().saturating_mul(3));
    for event in canonical {
        duplicate_heavy.extend([event.clone(), event.clone(), event.clone()]);
    }
    vec![
        SignedPermutation {
            name: "canonical",
            events: canonical.to_vec(),
        },
        SignedPermutation {
            name: "reverse",
            events: reverse,
        },
        SignedPermutation {
            name: "seed_0",
            events: seed_zero,
        },
        SignedPermutation {
            name: "seed_24301",
            events: seed_fixed,
        },
        SignedPermutation {
            name: "duplicate_heavy",
            events: duplicate_heavy,
        },
        SignedPermutation {
            name: "dependencies_last",
            events: delay(canonical, &is_dependency),
        },
        SignedPermutation {
            name: "controls_last",
            events: delay(canonical, &is_control),
        },
        SignedPermutation {
            name: "invalid_before_valid",
            events: prioritize(canonical, &is_invalid),
        },
    ]
}

fn delay<T: Clone>(canonical: &[T], predicate: &impl Fn(&T) -> bool) -> Vec<T> {
    canonical
        .iter()
        .filter(|event| !predicate(event))
        .cloned()
        .chain(canonical.iter().filter(|event| predicate(event)).cloned())
        .collect()
}

fn prioritize<T: Clone>(canonical: &[T], predicate: &impl Fn(&T) -> bool) -> Vec<T> {
    canonical
        .iter()
        .filter(|event| predicate(event))
        .cloned()
        .chain(canonical.iter().filter(|event| !predicate(event)).cloned())
        .collect()
}

fn shuffle<T>(items: &mut [T], seed: u64) {
    let mut state = seed;
    for upper in (1..items.len()).rev() {
        state = splitmix64(state);
        let modulus = u64::try_from(upper + 1).unwrap_or(u64::MAX);
        let index = usize::try_from(state % modulus).unwrap_or(0);
        items.swap(upper, index);
    }
}

fn splitmix64(mut state: u64) -> u64 {
    state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut value = state;
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::required_delivery_permutations;

    #[test]
    fn permutation_generator_is_deterministic_and_complete() {
        let events = vec![1_u8, 2, 3, 4, 5, 6];
        let generate = || {
            required_delivery_permutations(
                &events,
                |event| *event == 3,
                |event| matches!(*event, 1 | 2),
                |event| *event == 5,
            )
        };
        let first = generate();
        assert_eq!(first, generate());
        assert_eq!(first.len(), 8);
        assert_eq!(first[0].events, events);
        assert_eq!(first[1].events, vec![6, 5, 4, 3, 2, 1]);
        assert_eq!(first[4].events.len(), events.len() * 3);
        assert_eq!(first[5].events.last(), Some(&3));
        assert!(first[6].events.ends_with(&[1, 2]));
        assert_eq!(first[7].events.first(), Some(&5));
        assert!(first.iter().enumerate().all(|(index, variant)| {
            index == 4
                || variant.events.iter().copied().collect::<BTreeSet<_>>()
                    == events.iter().copied().collect()
        }));
    }
}
