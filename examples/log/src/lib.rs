#![crate_type = "cdylib"]

use msfs::msfs;

/// ```cfg
/// [VCockpit0]
/// size_mm=0,0
/// pixel_size=0,0
/// texture=$PDF
/// htmlgauge00=WasmInstrument/WasmInstrument.html?wasm_module=log.wasm&wasm_gauge=LOG, 0,0,0,0
/// ```
#[msfs::gauge(name=LOG)]
fn log(_: &msfs::FsContext, service_id: msfs::PanelServiceID) -> msfs::GaugeCallbackResult {
    println!("RUST: FBWCB {:?}", service_id);
    Ok(())
}
