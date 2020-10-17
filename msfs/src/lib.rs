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
//! ### `FSAPI` Definition
//! The definition of `FSAPI` in `Legacy/gauges.h` needs to be modified
//! slightly:
//! ```diff
//! // Flightsim calling convention
//! +#ifndef FSAPI
//!  #ifdef _MSFS_WASM
//!  #define FSAPI
//!  #else
//!  #define FSAPI   __stdcall
//!  #endif
//! +#endif
//! ```
//! I have opened a ticket about this but I have no idea if Asobo will get
//! around to it.
//!
//! ### Symbol visibility bug in Rust
//! Until https://github.com/rust-lang/rfcs/issues/2771 is fixed, you will have
//! to run the `msfs-fix` util on your output wasm files, like so:
//! ```shell
//! $ cargo build
//! $ msfs-fix target/wasm32-wasi/release/foo.wasm > target/wasm32-wasi/release/foo.wasm
//! ```

pub mod msfs;
pub mod sim_connect;
pub mod sys;
