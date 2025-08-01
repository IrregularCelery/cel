#![cfg_attr(not(feature = "std"), no_std)]

mod decode;
mod encode;

pub use decode::from_string;
pub use encode::{to_mini, to_pretty, to_string};
