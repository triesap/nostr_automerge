//! Seeded small-model algebraic properties.
use nostr_automerge::checkpoint::{leaf_hash, merkle_root};
#[test]
fn expand_property_test_model() {
    let mut seed = 0x5eed_u64;
    for count in 1..=64 {
        let leaves = (0..count)
            .map(|index| {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                leaf_hash(
                    index,
                    count,
                    (seed.to_be_bytes().repeat(4)).try_into().unwrap_or([0; 32]),
                )
            })
            .collect::<Vec<_>>();
        let first = merkle_root(&leaves);
        let second = merkle_root(&leaves);
        assert_eq!(first, second);
        assert!(first.is_ok());
    }
}
