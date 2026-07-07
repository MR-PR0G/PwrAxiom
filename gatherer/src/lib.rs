pub mod logging;

pub(crate) use logging::{critical, debug, error, warning};
pub mod platform;
pub mod utils;

#[link(name = "nvtop", kind = "static")]
extern "C" {}