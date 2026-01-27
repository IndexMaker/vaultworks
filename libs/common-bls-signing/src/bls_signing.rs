use common_bls::{affine::Signature, bls::hash_to_curve};
use pairing::group::Curve;

use crate::signing_key::SigningKey;

pub fn sign_message(message: &[u8], signing_key: SigningKey) -> Signature {
    let mut proj = hash_to_curve(message);
    proj *= signing_key;
    proj.to_affine()
}
