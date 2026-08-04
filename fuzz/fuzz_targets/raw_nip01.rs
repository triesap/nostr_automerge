#![no_main]
use libfuzzer_sys::fuzz_target;
use nostr_automerge::{ProtocolRevision,RawEventBytes,VerifiedNip01Event};
fuzz_target!(|data:&[u8]|{if let Ok(raw)=RawEventBytes::new(data,ProtocolRevision::draft_v1()){let _=VerifiedNip01Event::verify(raw);}});
