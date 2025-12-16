#![cfg_attr(all(feature = "stylus", not(feature = "stylus-test")), no_std)]

//#[macro_use]
extern crate alloc;

pub mod amount;
pub mod asset;
pub mod contracts {
    pub mod interfaces {
        pub mod banker;
        pub mod castle;
        pub mod clerk;
        pub mod constable;
        pub mod factor;
        pub mod granary;
        pub mod guildmaster;
        pub mod scribe;
        pub mod worksman;
    }
    pub mod acl;
    pub mod calls;
    pub mod castle;
    pub mod delegate;
    pub mod granary;
    pub mod keep;
    pub mod keep_calls;
}
pub mod labels;
pub mod log;
pub mod math;
pub mod storage;
pub mod uint;
pub mod vector;
pub mod vis;
