//! Sealed-limit resource benchmark smoke.

use nostr_automerge::checkpoint::{leaf_hash, merkle_root};
fn main() {
    let leaves = (0..4096)
        .map(|index| leaf_hash(index, 4096, [index as u8; 32]))
        .collect::<Vec<_>>();
    assert!(merkle_root(&leaves).is_ok());
}
