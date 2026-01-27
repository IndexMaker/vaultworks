use bls12_381::{
    hash_to_curve::{ExpandMsgXmd, HashToCurve},
    Bls12, G2Projective, Gt,
};
use pairing::{group::Curve, MultiMillerLoop};

use crate::{
    affine::{PublicKey, Signature},
    keccak_hash::Keccak256Hash,
};

pub fn hash_to_curve(message: &[u8]) -> G2Projective {
    let cs = b"BLS_SIG_BLS12381G1_XMD:KECCAK-256_SSWU_RO_";

    let point =
        <G2Projective as HashToCurve<ExpandMsgXmd<Keccak256Hash>>>::hash_to_curve(message, cs);
    point
}

pub fn verify_signature(message: &[u8], public_key: &PublicKey, signature: &Signature) -> bool {
    let h = hash_to_curve(message);
    let h_prepared = h.to_affine().into();
    let signature_prepared = (*signature).into();
    let gen_neg = -PublicKey::generator();

    let product =
        Bls12::multi_miller_loop(&[(public_key, &h_prepared), (&gen_neg, &signature_prepared)])
            .final_exponentiation();

    product == Gt::identity()
}
