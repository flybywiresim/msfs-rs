//! # msfs-rs
//!
//! These bindings include:
//!
//! - MSFS Gauge API
//! - SimConnect API
//!
//! ## Building
//!
//! If your MSFS SDK is not installed to `C:\MSFS SDK` you will need to set the
//! `MSFS_SDK` env variable to the correct path.
//!
//! ## Known Issues and Work-Arounds
//!
//! ### Missing various exports
//! Add a .cargo/config.toml file with the following settings:
//! ```toml
//! [target.wasm32-wasi]
//! rustflags = [
//!   "-Clink-arg=--export-table",
//!   "-Clink-arg=--export=malloc",
//!   "-Clink-arg=--export=free",
//! ]
//! ```

pub mod msfs;
pub mod sim_connect;
pub mod sys;
