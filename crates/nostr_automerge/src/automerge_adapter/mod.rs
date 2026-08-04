//! Private anti-corruption boundary for every Automerge upstream interaction.

// No upstream type may leave this module tree.

pub(crate) mod document;
pub(crate) mod framing;
pub(crate) mod leb128;
