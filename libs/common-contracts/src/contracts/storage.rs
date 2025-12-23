use alloy_primitives::U256;
use stylus_sdk::host::VM;
use stylus_sdk::storage::StorageType;

const SLOT_BYTE_SPACE: u8 = 32;

/// Code borrowed from openzeppelin (rust-contracts-stylus)
pub struct StorageSlot;

impl StorageSlot {
    /// Returns a [`StorageType`] located at `slot`.
    ///
    /// # Arguments
    ///
    /// * `slot` - The slot to get the address from.
    #[must_use]
    pub fn get_slot<ST: StorageType>(slot: impl Into<U256>) -> ST {
        // TODO: Remove this once we have a proper way to inject the host for
        // custom storage slot access.
        // This has been implemented on Stylus SDK 0.10.0.

        // Priority order:
        // 1. If wasm32 target -> always use tuple syntax (highest priority).
        // 2. If reentrant feature enabled (on non-wasm32) -> use struct syntax.
        // 3. If non-wasm32 without export-abi -> use struct syntax.
        // 4. Everything else -> use tuple syntax.

        #[cfg(not(target_arch = "wasm32"))]
        let host = VM {
            host: alloc::boxed::Box::new(stylus_sdk::host::WasmVM {}),
        };

        #[cfg(target_arch = "wasm32")]
        let host = VM(stylus_sdk::host::WasmVM {});

        // SAFETY: Truncation is safe here because ST::SLOT_BYTES is never
        // larger than 32, so the subtraction cannot underflow and the
        // cast is always valid.
        #[allow(clippy::cast_possible_truncation)]
        unsafe {
            ST::new(slot.into(), SLOT_BYTE_SPACE - ST::SLOT_BYTES as u8, host)
        }
    }
}
