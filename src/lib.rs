pub mod msfs;
pub mod sys;
// pub mod sim_connect;

// Prevent compilation of non wasm32-wasi targets
#[cfg(not(target_os = "wasi"))]
#[doc(hidden)]
fn invalid() {
    let _: [(); 0] = [0]; // This library only supports wasm32-wasi
}
