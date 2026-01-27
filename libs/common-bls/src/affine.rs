#[cfg(not(feature = "stylus"))]
use ff_zeroize::{PrimeField, ScalarEngine};

#[cfg(not(feature = "stylus"))]
use pairing_plus::bls12_381::{Fr, FrRepr};

use pairing_plus::{
    bls12_381::{Bls12, G1Compressed, G1Uncompressed, G2Compressed, G2Uncompressed},
    CurveAffine, EncodedPoint, Engine,
};

use crate::keccak_hash::Keccak256Hash;

#[cfg(not(feature = "stylus"))]
pub type SigningKey = <Bls12 as ScalarEngine>::Fr;

pub type PublicKey = <Bls12 as Engine>::G2Affine;
pub type Signature = <Bls12 as Engine>::G1Affine;

pub const SIGNING_KEY_LEN: usize = 32;

pub const PUBLIC_KEY_COMPRESSED_LEN: usize = 96;
pub const PUBLIC_KEY_UNCOMPRESSED_LEN: usize = 192;

pub const SIGNATURE_COMPRESSED_LEN: usize = 48;
pub const SIGNATURE_UNCOMPRESSED_LEN: usize = 96;

#[cfg(not(feature = "stylus"))]
pub fn signing_key_from_data(data: &[u8]) -> Result<SigningKey, Vec<u8>> {
    if data.len() != SIGNING_KEY_LEN {
        Err(b"Invalid signing key length")?;
    }
    let repr = FrRepr::default();
    let res = Fr::from_repr(repr).map_err(|_| b"Failed to deserialize signing key")?;
    Ok(res)
}

#[cfg(feature = "stylus-export-abi")]
pub fn public_key_from_data(data: &[u8]) -> Result<PublicKey, Vec<u8>> {
    unimplemented!()
}

#[cfg(not(feature = "stylus-export-abi"))]
pub fn public_key_from_data(data: &[u8]) -> Result<PublicKey, Vec<u8>> {
    match data.len() {
        PUBLIC_KEY_COMPRESSED_LEN => {
            let mut val = G2Compressed::empty();
            val.as_mut().copy_from_slice(data);
            let res = val
                .into_affine()
                .map_err(|_| b"Failed to decode affine point")?;
            Ok(res)
        }
        #[cfg(not(feature = "stylus"))]
        PUBLIC_KEY_UNCOMPRESSED_LEN => {
            let mut val = G2Uncompressed::empty();
            val.as_mut().copy_from_slice(data);
            let res = val
                .into_affine()
                .map_err(|_| b"Failed to decode affine point")?;
            Ok(res)
        }
        _ => Err(b"Invalid public key length")?,
    }
}

#[cfg(feature = "stylus-export-abi")]
pub fn signature_from_data(data: &[u8]) -> Result<Signature, Vec<u8>> {
    unimplemented!()
}

#[cfg(not(feature = "stylus-export-abi"))]
pub fn signature_from_data(data: &[u8]) -> Result<Signature, Vec<u8>> {
    match data.len() {
        SIGNATURE_COMPRESSED_LEN => {
            let mut val = G1Compressed::empty();
            val.as_mut().copy_from_slice(data);
            let res = val
                .into_affine()
                .map_err(|_| b"Failed to decode affine point")?;
            Ok(res)
        }
        #[cfg(not(feature = "stylus"))]
        SIGNATURE_UNCOMPRESSED_LEN => {
            let mut val = G1Uncompressed::empty();
            val.as_mut().copy_from_slice(data);
            let res = val
                .into_affine()
                .map_err(|_| b"Failed to decode affine point")?;
            Ok(res)
        }
        _ => Err(b"Invalid public key length")?,
    }
}

#[cfg(not(feature = "stylus"))]
pub fn public_key_into_bytes(public_key: &PublicKey, out: &mut Vec<u8>, compress: bool) {
    if compress {
        let compressed = public_key.into_compressed();
        out.extend_from_slice(compressed.as_ref());
    } else {
        let uncompressed = public_key.into_uncompressed();
        out.extend_from_slice(uncompressed.as_ref());
    }
}

#[cfg(not(feature = "stylus"))]
pub fn signature_into_bytes(signature: &Signature, out: &mut Vec<u8>, compress: bool) {
    if compress {
        let compressed = signature.into_compressed();
        out.extend_from_slice(compressed.as_ref());
    } else {
        let uncompressed = signature.into_uncompressed();
        out.extend_from_slice(uncompressed.as_ref());
    }
}
