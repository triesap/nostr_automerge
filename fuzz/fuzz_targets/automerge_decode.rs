#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    nostr_automerge::qualification_probe_automerge_decode(data);
});
