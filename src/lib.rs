#![cfg_attr(coverage, feature(coverage_attribute))]

pub mod configuration;
pub mod contracts;
pub mod http;
pub mod services;
// #[cfg(feature = "testing")]
// COVERAGE: disabled since the module is only compiled for testing purposes
#[cfg_attr(coverage, coverage(off))]
pub mod testing;
