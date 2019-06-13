#[cfg_attr(not(feature = "no_std"), path = "lib_std.rs")]
#[cfg_attr(feature = "no_std", path = "lib_no_std.rs")]
pub mod lib;
