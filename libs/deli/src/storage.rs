use alloy_primitives::U256;
#[cfg(all(
            not(target_arch = "wasm32"),
            any(feature = "reentrant", not(feature = "export-abi"))
        ))]
use stylus_sdk::host::VM;
use stylus_sdk::storage::StorageType;



/// Code borrowed from openzeppelin (rust-contracts-stylus)
pub struct StorageSlot;

impl StorageSlot {
    const SLOT_BYTE_SPACE: u8 = 32;

    #[must_use]
    pub fn get_slot<ST: StorageType>(slot: impl Into<U256>) -> ST {
        #[cfg(all(
            not(target_arch = "wasm32"),
            any(feature = "reentrant", not(feature = "export-abi"))
        ))]
        let host = VM {
            host: alloc::boxed::Box::new(stylus_sdk::host::WasmVM {}),
        };

        #[cfg(not(all(
            not(target_arch = "wasm32"),
            any(feature = "reentrant", not(feature = "export-abi"))
        )))]
        let host = VM(stylus_sdk::host::WasmVM {});

        #[allow(clippy::cast_possible_truncation)]
        unsafe {
            ST::new(
                slot.into(),
                Self::SLOT_BYTE_SPACE - ST::SLOT_BYTES as u8,
                host,
            )
        }
    }
}