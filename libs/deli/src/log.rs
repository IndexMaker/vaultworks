//!
//! Workaround for logging debug messages in tests as console!() macro crashes
//! with SIGSEGV, or code doesn't link.
//!

#[cfg(all(feature = "debug", any(not(feature = "stylus"), feature = "stylus-test", feature = "export-abi")))]
pub fn print_msg(msg: &str) {
    println!("{}", msg);
}

#[cfg(not(feature = "debug"))]
#[macro_export]
macro_rules! log_msg {
    ($($t:tt)*) => {};
}

#[cfg(all(feature = "debug", any(not(feature = "stylus"), feature = "stylus-test", feature = "export-abi")))]
#[macro_export]
macro_rules! log_msg {
    ($fmt:literal $(, $args:expr)*) => {
        $crate::log::print_msg(&format!($fmt $(, $args)*));
    };
}

#[cfg(all(feature = "debug", not(all(not(feature = "stylus"), feature = "stylus-test", feature = "export-abi"))))]
#[macro_export]
macro_rules! log_msg {
    ($($msg:tt)*) => {
        stylus_sdk::debug::console_log(alloc::format!($($msg)*));
    };
}
