//! Functional tests for classic / pre-modern-horizons sets, one module per
//! set. Grouped into a single integration-test binary to keep link time and
//! `target/` size in check (one binary per set would link ~40 copies of the
//! engine).

mod ktk;
