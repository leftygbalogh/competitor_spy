// competitor_spy_domain
//
// Pure domain crate: zero I/O, zero async, zero rendering dependencies.
// All business logic lives here. Adapter, output, and CLI crates depend
// on this crate — never the reverse.

pub mod query;
pub mod profile;
pub mod run;
pub mod ranking;
pub mod scoring;
