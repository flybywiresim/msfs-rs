#![crate_type = "cdylib"]

use ::msfs::{msfs, sim_connect::SimConnect};

static mut SIM: Option<SimConnect> = None;

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
    match service_id {
        msfs::PanelServiceID::PreInstall => match SimConnect::open("log") {
            Ok(s) => {
                unsafe { SIM = Some(s) };
                Ok(())
            }
            Err(_) => Err(()),
        },
        msfs::PanelServiceID::PreKill => {
            drop(unsafe { SIM.take() });
            Ok(())
        }
        msfs::PanelServiceID::PreUpdate => {
            println!("SimConnect Dispatch {:?}", unsafe {
                SIM.as_ref().unwrap().get_next_dispatch()
            });
            Ok(())
        }
        _ => Ok(()),
    }
}
