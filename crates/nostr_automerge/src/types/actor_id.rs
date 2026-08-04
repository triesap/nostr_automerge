semantic_id!(
    ActorId,
    "The derived 32-byte Automerge actor identity of a device."
);

impl ActorId {
    /// Derives the sealed actor identity from a document coordinate and device key.
    #[must_use]
    pub fn derive(coordinate: crate::DocumentCoordinate, device: crate::DevicePublicKey) -> Self {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(b"nostr-crdt/automerge/actor/v1");
        hasher.update([0]);
        hasher.update(coordinate.controller().as_bytes());
        hasher.update(coordinate.document_id().as_bytes());
        hasher.update(device.as_bytes());
        Self::from_bytes(hasher.finalize().into())
    }
}
