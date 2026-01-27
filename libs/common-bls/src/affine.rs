use bls12_381::{G1Affine, G2Affine};

pub type Signature = G2Affine;
pub type PublicKey = G1Affine;

pub const SIGNATURE_COMPRESSED_LEN: usize = 96;
pub const SIGNATURE_UNCOMPRESSED_LEN: usize = 192;
pub const PUBLIC_KEY_COMPRESSED_LEN: usize = 48;
pub const PUBLIC_KEY_UNCOMPRESSED_LEN: usize = 96;

#[cfg(feature = "stylus-export-abi")]
pub fn public_key_from_data(data: &[u8]) -> Result<PublicKey, Vec<u8>> {
    unimplemented!()
}

#[cfg(not(feature = "stylus-export-abi"))]
pub fn public_key_from_data(data: &[u8]) -> Result<PublicKey, Vec<u8>> {
    match data.len() {
        PUBLIC_KEY_COMPRESSED_LEN => {
            let mut bytes = [0u8; PUBLIC_KEY_COMPRESSED_LEN];
            bytes.copy_from_slice(data);
            let res = PublicKey::from_compressed(&bytes)
                .into_option()
                .ok_or_else(|| b"Failed to decode affine point")?;
            Ok(res)
        }
        #[cfg(not(feature = "stylus"))]
        PUBLIC_KEY_UNCOMPRESSED_LEN => {
            let mut bytes = [0u8; PUBLIC_KEY_UNCOMPRESSED_LEN];
            bytes.copy_from_slice(data);
            let res = PublicKey::from_uncompressed(&bytes)
                .into_option()
                .ok_or_else(|| b"Failed to decode affine point")?;
            Ok(res)
        }
        _ => {
            return Err(b"Invalid public key length")?;
        }
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
            let mut bytes = [0u8; SIGNATURE_COMPRESSED_LEN];
            bytes.copy_from_slice(data);
            let res = Signature::from_compressed(&bytes)
                .into_option()
                .ok_or_else(|| b"Failed to decode affine point")?;

            Ok(res)
        }
        #[cfg(not(feature = "stylus"))]
        SIGNATURE_UNCOMPRESSED_LEN => {
            let mut bytes = [0u8; SIGNATURE_UNCOMPRESSED_LEN];
            bytes.copy_from_slice(data);
            let res = Signature::from_uncompressed(&bytes)
                .into_option()
                .ok_or_else(|| b"Failed to decode affine point")?;

            Ok(res)
        }
        _ => {
            return Err(b"Invalid signature length")?;
        }
    }
}

#[cfg(not(any(feature = "stylus", feature = "stylus-export-abi")))]
pub fn public_key_into_bytes(public_key: &PublicKey, out: &mut Vec<u8>, compress: bool) {
    if compress {
        out.extend_from_slice(&public_key.to_compressed());
    } else {
        out.extend_from_slice(&public_key.to_uncompressed());
    }
}

#[cfg(not(any(feature = "stylus", feature = "stylus-export-abi")))]
pub fn signature_into_bytes(signature: &Signature, out: &mut Vec<u8>, compress: bool) {
    if compress {
        out.extend_from_slice(&signature.to_compressed());
    } else {
        out.extend_from_slice(&signature.to_uncompressed());
    }
}
