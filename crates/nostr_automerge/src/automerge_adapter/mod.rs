//! Private anti-corruption boundary for every Automerge upstream interaction.

// No upstream type may leave this module tree.

pub(crate) mod counters;
pub(crate) mod decode;
pub(crate) mod document;
pub(crate) mod encode;
#[cfg(test)]
mod fixture;
pub(crate) mod framing;
pub(crate) mod leb128;
pub(crate) mod types;
