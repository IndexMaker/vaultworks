use common::{labels::Labels, vector::Vector};
use ethers::types::Bytes;

pub mod contracts;
pub mod tx_sender;

pub trait ToBytes {
    fn to_bytes(self) -> Bytes;
}

impl ToBytes for Vec<u8> {
    fn to_bytes(self) -> Bytes {
        Bytes::from(self)
    }
}

impl ToBytes for Labels {
    fn to_bytes(self) -> Bytes {
        Bytes::from(self.to_vec())
    }
}

impl ToBytes for Vector {
    fn to_bytes(self) -> Bytes {
        Bytes::from(self.to_vec())
    }
}
