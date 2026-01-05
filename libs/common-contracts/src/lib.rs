#![cfg_attr(all(feature = "stylus", not(feature = "stylus-test")), no_std)]

//#[macro_use]
extern crate alloc;

#[cfg(feature = "stylus")]
pub mod contracts {
    pub mod acl;
    pub mod formulas;
    pub mod calls;
    pub mod castle;
    pub mod delegate;
    pub mod clerk;
    pub mod keep;
    pub mod keep_calls;
    pub mod storage;
}

pub mod interfaces {
    pub mod banker;
    pub mod castle;
    pub mod abacus;
    pub mod constable;
    pub mod factor;
    pub mod clerk;
    pub mod guildmaster;
    pub mod scribe;
    pub mod treasury;
    pub mod worksman;
}
