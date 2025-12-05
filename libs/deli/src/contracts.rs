use alloc::string::String;
use alloy_primitives::U8;

pub trait IERC20Metadata {
    fn name(&self) -> String;
    fn symbol(&self) -> String;
    fn decimals(&self) -> U8;
}
