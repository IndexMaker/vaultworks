use alloy_primitives::{uint, B256, U256};
use stylus_sdk::keccak_const;

pub const UPGRADE_INTERFACE_VERSION: &str = "5.0.0";

pub const IMPLEMENTATION_SLOT: B256 = {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"eip1967.proxy.implementation")
        .finalize();
    B256::new(
        U256::from_be_bytes(HASH)
            .wrapping_sub(uint!(1_U256))
            .to_be_bytes(),
    )
};
