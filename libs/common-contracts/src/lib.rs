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
    pub mod clerk_util;
    pub mod storage;
    pub mod vault;
    pub mod vault_native;
}

pub mod interfaces {
    pub mod alchemist;
    pub mod banker;
    pub mod castle;
    pub mod clerk;
    pub mod constable;
    pub mod factor;
    pub mod guildmaster;
    pub mod scribe;
    pub mod steward;
    pub mod treasury;
    pub mod vault;
    pub mod vault_native;
    pub mod vault_native_orders;
    pub mod vault_native_claims;
    pub mod worksman;
}
