#![cfg_attr(all(feature = "stylus", not(feature = "stylus-test")), no_std)]

//#[macro_use]
extern crate alloc;

#[cfg(feature = "stylus")]
pub mod contracts {
    pub mod acl;
    pub mod calls;
    pub mod castle;
    pub mod clerk;
    pub mod delegate;
    pub mod formulas;
    pub mod gate;
    pub mod keep;
    pub mod keep_calls;
    pub mod storage;
    pub mod vault;
    pub mod vault_native;
    pub mod vault_requests;
}

pub mod interfaces {
    pub mod banker;
    pub mod castle;
    pub mod clerk;
    pub mod constable;
    pub mod factor;
    pub mod guildmaster;
    pub mod scribe;
    pub mod treasury;
    pub mod vault;
    pub mod vault_native;
    pub mod vault_requests;
    pub mod worksman;
}
