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
//! ## Known Issues
//!
//! Until https://github.com/rust-lang/rfcs/issues/2771 is fixed, you will have to
//! run the `msfs-fix` util on your output wasm files, like so:
//! ```shell
//! $ cargo build
//! $ msfs-fix target/wasm32-wasi/release/foo.wasm > target/wasm32-wasi/release/foo.wasm
//! ```

pub mod msfs;
pub mod sim_connect;
pub mod sys;

// Prevent compilation of non wasm32-wasi targets
#[cfg(not(target_os = "wasi"))]
#[doc(hidden)]
fn invalid() {
    let _: [(); 0] = [0]; // This library only supports wasm32-wasi
}
