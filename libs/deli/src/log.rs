//!
//! Workaround for logging debug messages in tests as console!() macro crashes
//! with SIGSEGV, or code doesn't link.
//!

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

#[cfg(all(
    feature = "debug",
    feature = "stylus",
    not(feature = "stylus-test"),
    not(feature = "export-abi")
))]
#[macro_export]
macro_rules! log_msg {
    ($($msg:tt)*) => {
        stylus_sdk::debug::console_log(alloc::format!($($msg)*));
    };
}
