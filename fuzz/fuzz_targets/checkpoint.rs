#![no_main]
use libfuzzer_sys::fuzz_target;
use nostr_automerge::{SnapshotHash,checkpoint::{CheckpointDescriptor,leaf_hash,merkle_root}};
fuzz_target!(|data:&[u8]|{if let Ok(text)=core::str::from_utf8(data){let _=CheckpointDescriptor::parse_content(text,SnapshotHash::from_bytes([0;32]));}let leaves=data.chunks(32).take(64).enumerate().map(|(i,c)|{let mut h=[0;32];h[..c.len()].copy_from_slice(c);leaf_hash(i as u32,data.len().div_ceil(32).min(64) as u32,h)}).collect::<Vec<_>>();let _=merkle_root(&leaves);});
