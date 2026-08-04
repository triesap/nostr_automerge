use nostr_automerge::authoring::PreparedEvent;
use nostr_automerge::{DevicePublicKey, ProtocolRevision, RawEventBytes};
use secp256k1::{Keypair, Secp256k1, SecretKey};

pub(crate) struct TestSigner {
    keypair: Keypair,
    public_key: DevicePublicKey,
}

impl TestSigner {
    #[allow(clippy::expect_used)]
    pub(crate) fn from_byte(byte: u8) -> Self {
        let secret = SecretKey::from_byte_array([byte; 32]).expect("fixed test secret");
        let keypair = Keypair::from_secret_key(&Secp256k1::new(), &secret);
        let (public_key, _) = keypair.x_only_public_key();
        Self {
            keypair,
            public_key: DevicePublicKey::from_bytes(public_key.serialize()),
        }
    }

    pub(crate) const fn public_key(&self) -> DevicePublicKey {
        self.public_key
    }

    #[allow(clippy::expect_used)]
    pub(crate) fn sign(&self, prepared: &PreparedEvent) -> RawEventBytes {
        let signature = Secp256k1::new()
            .sign_schnorr_no_aux_rand(prepared.event_id().as_bytes(), &self.keypair);
        let raw = serde_json::to_vec(&serde_json::json!({
            "id": prepared.event_id().to_hex(),
            "pubkey": prepared.public_key().to_hex(),
            "created_at": prepared.created_at(),
            "kind": prepared.kind(),
            "tags": prepared.tags(),
            "content": prepared.content(),
            "sig": signature.to_string(),
        }))
        .expect("test event serialization");
        RawEventBytes::new(&raw, ProtocolRevision::draft_v1()).expect("bounded test event")
    }
}
