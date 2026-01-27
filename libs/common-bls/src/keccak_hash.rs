#[cfg(not(feature = "stylus"))]
use alloy_primitives::keccak256;

use alloy_primitives::{Keccak256, B256};

use digest::generic_array::typenum::U64;
#[cfg(feature = "stylus")]
use digest::Output;

#[cfg(not(feature = "stylus"))]
pub struct Keccak256Hash(Option<Keccak256>);

#[cfg(feature = "stylus")]
pub struct Keccak256Hash(Keccak256);

impl digest::BlockInput for Keccak256Hash {
    type BlockSize = U64;
}

// TODO: Fixme
#[cfg(not(feature = "stylus"))]
impl digest::Digest for Keccak256Hash {
    type OutputSize = U64;

    fn new() -> Self {
        Self(Some(Keccak256::new()))
    }

    fn input<B: AsRef<[u8]>>(&mut self, data: B) {
        self.0.update(data);
    }

    fn chain<B: AsRef<[u8]>>(mut self, data: B) -> Self
    where
        Self: Sized,
    {
        let Some(inner) = &mut self.0 else {
            panic!("unexpected empty Keccak256")
        };
        inner.update(data);
        self
    }

    fn result(mut self) -> digest::generic_array::GenericArray<u8, Self::OutputSize> {
        let Some(inner) = self.0 else {
            panic!("unexpected empty Keccak256")
        };
        let res = inner.finalize();
        let mut arr = digest::generic_array::GenericArray::default();
        arr.copy_from_slice(&res.0);
        arr
    }

    fn result_reset(&mut self) -> digest::generic_array::GenericArray<u8, Self::OutputSize> {
        let Some(inner) = self.0.replace(Keccak256::new()) else {
            panic!("unexpected empty Keccak256")
        };
        let res = inner.finalize();
        let mut arr = digest::generic_array::GenericArray::default();
        arr.copy_from_slice(&res.0);
        arr
    }

    fn reset(&mut self) {
        self.0.replace(Keccak256::new());
    }

    fn output_size() -> usize {
        B256::len_bytes()
    }

    fn digest(data: &[u8]) -> digest::generic_array::GenericArray<u8, Self::OutputSize> {
        let res = keccak256(data);
        let mut arr = digest::generic_array::GenericArray::default();
        arr.copy_from_slice(&res.0);
        arr
    }
}

#[cfg(feature = "stylus")]
impl digest::Digest for Keccak256Hash {
    type OutputSize = U64;

    fn new() -> Self {
        Self(Keccak256::default())
    }

    fn output_size() -> usize {
        B256::len_bytes()
    }

    fn chain(mut self, data: impl AsRef<[u8]>) -> Self
    where
        Self: Sized,
    {
        self.0.update(data);
        self
    }
    fn update(&mut self, data: impl AsRef<[u8]>) {
        self.0.update(data);
    }

    fn finalize(self) -> Output<Self> {
        let res = self.0.finalize();
        let mut arr = digest::generic_array::GenericArray::default();
        arr.copy_from_slice(&res.0);
        arr
    }

    fn reset(&mut self) {
        unimplemented!()
    }

    fn digest(_data: &[u8]) -> digest::generic_array::GenericArray<u8, Self::OutputSize> {
        unimplemented!()
    }

    fn finalize_reset(&mut self) -> Output<Self> {
        unimplemented!()
    }
}
