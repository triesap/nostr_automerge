use secp256k1::{Secp256k1, XOnlyPublicKey, schnorr::Signature};

use crate::types::public_key::VerifiedPublicKey;
use crate::{EventId, Nip01Signature};

pub(crate) fn verify(
    public_key: VerifiedPublicKey,
    event_id: EventId,
    signature: Nip01Signature,
) -> Result<(), Bip340Error> {
    let public_key = XOnlyPublicKey::from_byte_array(*public_key.as_bytes())
        .map_err(|_| Bip340Error::InvalidPublicKey)?;
    let signature = Signature::from_byte_array(*signature.as_bytes());
    Secp256k1::verification_only()
        .verify_schnorr(&signature, event_id.as_bytes(), &public_key)
        .map_err(|_| Bip340Error::InvalidSignature)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Bip340Error {
    InvalidPublicKey,
    InvalidSignature,
}

#[cfg(test)]
mod tests {
    use super::{Bip340Error, verify};
    use crate::types::public_key::VerifiedPublicKey;
    use crate::{EventId, Nip01Signature};
    use core::str::FromStr;

    #[test]
    #[allow(clippy::expect_used)]
    fn verifies_official_bip340_vector_zero() {
        let key = VerifiedPublicKey::parse(
            "f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9",
        )
        .expect("official key");
        let signature = Nip01Signature::from_str("e907831f80848d1069a5371b402410364bdf1c5f8307b0084c55f1ce2dca821525f66a4a85ea8b71e482a74f382d2ce5ebeee8fdb2172f477df4900d310536c0").expect("official signature");
        assert_eq!(verify(key, EventId::from_bytes([0; 32]), signature), Ok(()));
        let mut invalid = *signature.as_bytes();
        invalid[63] ^= 1;
        assert_eq!(
            verify(
                key,
                EventId::from_bytes([0; 32]),
                Nip01Signature::from_bytes(invalid)
            ),
            Err(Bip340Error::InvalidSignature)
        );
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn rejects_x_coordinate_that_is_not_a_curve_point() {
        let key = VerifiedPublicKey::parse(&"ff".repeat(32)).expect("fixed-width key");
        assert_eq!(
            verify(
                key,
                EventId::from_bytes([0; 32]),
                Nip01Signature::from_bytes([0; 64])
            ),
            Err(Bip340Error::InvalidPublicKey)
        );
    }
}
