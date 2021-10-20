//! # msfs-rs
//!
//! These bindings include:
//!
//! - MSFS Gauge API
//! - SimConnect API
//! - NanoVG API
//!
//! ## Building
//!
//! Tools such as `cargo-wasi` may not work. When in doubt, try invoking
//! `cargo build --target wasm32-wasi` directly.
//!
//! If your MSFS SDK is not installed to `C:\MSFS SDK` you will need to set the
//! `MSFS_SDK` env variable to the correct path.
//!
//! ## Known Issues and Work-Arounds
//!
//! ### Missing various exports
//! Add a local `.cargo/config.toml` file with the following settings:
//! ```toml
//! [target.wasm32-wasi]
//! rustflags = [
//!   "-Clink-arg=--export-table",
//!   "-Clink-arg=--export=malloc",
//!   "-Clink-arg=--export=free",
//! ]
//! ```

mod msfs;
pub mod sim_connect;
pub mod sys;

pub use msfs::*;

#[cfg(any(target_arch = "wasm32", doc))]
pub mod legacy;

#[cfg(any(target_arch = "wasm32", doc))]
pub mod nvg;

#[doc(hidden)]
pub mod executor;
