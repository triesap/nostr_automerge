#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DeliveryPermutation<T> {
    pub(crate) name: String,
    pub(crate) items: Vec<T>,
}

pub(crate) fn delivery_permutations<T: Clone>(
    canonical: &[T],
    seeds: &[u64],
) -> Vec<DeliveryPermutation<T>> {
    let mut variants = vec![DeliveryPermutation {
        name: "canonical".to_owned(),
        items: canonical.to_vec(),
    }];
    let mut reverse = canonical.to_vec();
    reverse.reverse();
    variants.push(DeliveryPermutation {
        name: "reverse".to_owned(),
        items: reverse,
    });
    for seed in seeds {
        let mut items = canonical.to_vec();
        shuffle(&mut items, *seed);
        variants.push(DeliveryPermutation {
            name: format!("seed_{seed}"),
            items,
        });
    }
    variants
}

fn shuffle<T>(items: &mut [T], seed: u64) {
    let mut state = seed;
    for upper in (1..items.len()).rev() {
        let random = splitmix64(&mut state);
        let modulus = u64::try_from(upper + 1).unwrap_or(u64::MAX);
        let index = usize::try_from(random % modulus).unwrap_or(0);
        items.swap(upper, index);
    }
}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut value = *state;
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::delivery_permutations;

    #[test]
    fn add_seeded_permutation_runner() {
        let events = vec![1_u8, 2, 3, 4, 5, 6];
        let first = delivery_permutations(&events, &[0, 1, 0x5eed]);
        let second = delivery_permutations(&events, &[0, 1, 0x5eed]);
        assert_eq!(first, second);
        assert_eq!(first[0].items, events);
        assert_eq!(first[1].items, vec![6, 5, 4, 3, 2, 1]);
        assert!(first.iter().all(|variant| {
            variant.items.iter().copied().collect::<BTreeSet<_>>()
                == events.iter().copied().collect::<BTreeSet<_>>()
        }));

        let canonical_report = |delivery: &[u8]| delivery.iter().copied().collect::<BTreeSet<_>>();
        assert!(
            first
                .iter()
                .all(|variant| canonical_report(&variant.items) == canonical_report(&events))
        );
    }
}
