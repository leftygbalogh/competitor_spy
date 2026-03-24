// competitor_spy_cli lib — exposes run_with_urls for acceptance tests.
// The binary (main.rs) calls run_with_urls with production URLs.

pub mod runner;
pub use runner::{AdapterUrls, run_with_urls};
