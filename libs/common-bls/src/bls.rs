use ff_zeroize::Field;
use pairing_plus::{
    bls12_381::{Bls12, Fq12, G1},
    hash_to_curve::HashToCurve,
    hash_to_field::ExpandMsgXmd,
    CurveAffine, CurveProjective, Engine,
};

#[cfg(not(feature = "stylus"))]
use crate::affine::SigningKey;

use crate::{
    affine::{PublicKey, Signature},
    keccak_hash::Keccak256Hash,
};

pub fn hash_to_curve(message: &[u8]) -> G1 {
    let cs = b"BLS_SIG_BLS12381G1_XMD:KECCAK-256_SSWU_RO_";
    let point = <G1 as HashToCurve<ExpandMsgXmd<Keccak256Hash>>>::hash_to_curve(message, cs);
    point
}

#[cfg(not(feature = "stylus"))]
pub fn sign_message(message: &[u8], signing_key: SigningKey) -> Signature {
    let mut signature = hash_to_curve(message);
    signature.mul_assign(signing_key);
    signature.into_affine()
}

pub fn verify_signature(message: &[u8], public_key: &PublicKey, signature: &Signature) -> bool {
    let mut h = hash_to_curve(message);
    let generator = PublicKey::one();
    let product = <pairing_plus::bls12_381::Bls12 as Engine>::pairing_product(
        h.into_affine(),
        *public_key,
        *signature,
        generator,
    );
    product == Fq12::one()
}

pub fn aggregate_public_keys(keys: Vec<PublicKey>) -> PublicKey {
    let mut agg = keys[0].into_projective();
    for i in 1..keys.len() {
        agg.add_assign(&keys[i].into_projective());
    }
    agg.into_affine()
}

pub fn aggregate_signatures(sigs: Vec<Signature>) -> Signature {
    let mut agg = sigs[0].into_projective();
    for i in 1..sigs.len() {
        agg.add_assign(&sigs[i].into_projective());
    }
    agg.into_affine()
}
