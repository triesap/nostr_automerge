//! Sealed-limit resource benchmark smoke.

use nostr_automerge::checkpoint::{leaf_hash, merkle_root};
fn main() {
    for count in [2048, 4096] {
        let leaves = (0..count)
            .map(|index| leaf_hash(index, count, [index as u8; 32]))
            .collect::<Vec<_>>();
        assert!(merkle_root(&leaves).is_ok());
    }
}
