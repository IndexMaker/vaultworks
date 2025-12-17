//! log_msg!() vs ~console!()~ macro
//! ---
//! 
//! There is few cases when we would like to log debug messages:
//! 
//! 1. Stylus contract release deployed to Nitro node, then we must disable all logging.
//! 
//! 2. Stylus contract with `debug` feature deployed to Nitro node, then we must use `stylus_sdk::console!()` macro to print messages into Nitro log.
//! 
//! 3. Stylus contract with `debug` feature Unit Tests, then we must use `println!()` as the ~`console!()`~ macro will SIGSEGV in this scenario.
//! 
//! 4. Application calling Stylus contracts, then we must use `println!()` as the ~`console!()`~ macro will SIGSEGV in this scenario.
//!

//
// Workaround for logging debug messages in tests as console!() macro crashes
// with SIGSEGV, or code doesn't link.
//
#[cfg(all(
    feature = "debug",
    any(
        feature = "stylus-test",
        not(any(feature = "stylus", feature = "export-abi"))
    )
))]
pub fn print_msg(msg: &str) {
    println!("{}", msg);
}

#[cfg(any(not(feature = "debug"), feature = "export-abi"))]
#[macro_export]
macro_rules! log_msg {
    ($($t:tt)*) => {};
}

#[cfg(all(
    feature = "debug",
    any(
        feature = "stylus-test",
        not(any(feature = "stylus", feature = "export-abi"))
    )
))]
#[macro_export]
macro_rules! log_msg {
    ($fmt:literal $(, $args:expr)*) => {
        $crate::log::print_msg(&format!($fmt $(, $args)*));
    };
}

//
// Workaround to make rust-analyser happy!
// 
#[cfg(feature = "stylus")]
pub use stylus_sdk;

#[cfg(all(
    feature = "debug",
    feature = "stylus",
    not(feature = "stylus-test"),
    not(feature = "export-abi")
))]
#[macro_export]
macro_rules! log_msg {
    ($fmt:literal $(, $args:expr)*) => {
        $crate::log::stylus_sdk::console!($fmt $(, $args)*);
    };
}
