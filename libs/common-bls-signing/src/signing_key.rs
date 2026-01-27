use bls12_381::{G1Projective, Scalar};

use common_bls::affine::PublicKey;
use pairing::group::Curve;

pub type SigningKey = Scalar;

pub const SIGNING_KEY_LEN: usize = 32;

pub fn signing_key_from_data(data: &[u8]) -> Result<SigningKey, Vec<u8>> {
    if data.len() != SIGNING_KEY_LEN {
        Err(b"Invalid signing key length")?;
    }
    let mut bytes = [0u8; SIGNING_KEY_LEN];
    bytes.copy_from_slice(data);
    let res = SigningKey::from_bytes(&bytes)
        .into_option()
        .ok_or_else(|| b"Failed to deserialize signing key")?;
    Ok(res)
}

pub fn public_key_from_signing_key(signing_key: &SigningKey) -> PublicKey {
    let mut g = G1Projective::generator();
    g *= signing_key;
    g.to_affine()
}
