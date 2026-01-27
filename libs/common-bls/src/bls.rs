#[cfg(not(feature = "stylus"))]
use bls12_381::G1Projective;
use bls12_381::{
    hash_to_curve::{ExpandMsgXmd, HashToCurve},
    Bls12, G2Projective, Gt,
};
use pairing::{group::Curve, MultiMillerLoop};

use crate::{
    affine::{PublicKey, Signature},
    keccak_hash::Keccak256Hash,
};

#[cfg(not(feature = "stylus"))]
use crate::affine::SigningKey;

pub fn hash_to_curve(message: &[u8]) -> G2Projective {
    let cs = b"BLS_SIG_BLS12381G1_XMD:KECCAK-256_SSWU_RO_";

    let point =
        <G2Projective as HashToCurve<ExpandMsgXmd<Keccak256Hash>>>::hash_to_curve(message, cs);
    point
}

#[cfg(not(feature = "stylus"))]
pub fn sign_message(message: &[u8], signing_key: SigningKey) -> Signature {
    let mut proj = hash_to_curve(message);
    proj *= signing_key;
    proj.to_affine()
}

pub fn verify_signature(message: &[u8], public_key: &PublicKey, signature: &Signature) -> bool {
    let h = hash_to_curve(message);
    let h_prepared = h.to_affine().into();
    let signature_prepared = (*signature).into();
    let gen_neg = -PublicKey::generator();

    let mut product = Bls12::multi_miller_loop(&[(public_key, &h_prepared)]);

    product += Bls12::multi_miller_loop(&[(&gen_neg, &signature_prepared)]);

    product.final_exponentiation() == Gt::identity()
}

#[cfg(not(feature = "stylus"))]
pub fn aggregate_public_keys(keys: Vec<PublicKey>) -> PublicKey {
    let mut agg = G1Projective::identity();
    for key in keys {
        agg += key;
    }
    agg.to_affine()
}

#[cfg(not(feature = "stylus"))]
pub fn aggregate_signatures(signatures: Vec<Signature>) -> Signature {
    let mut agg = G2Projective::identity();
    for signature in signatures {
        agg += signature;
    }
    agg.to_affine()
}
