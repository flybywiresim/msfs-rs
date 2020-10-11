use ::msfs::{msfs, sim_connect::{SimConnect, SimConnectRecv}};

static mut SIM: Option<SimConnect> = None;

fn simconnect_cb(_sim: &SimConnect, recv: SimConnectRecv) {
    println!("SimConnect Dispatch {:?}", recv);
}

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
        msfs::PanelServiceID::PreInstall => match SimConnect::open("log", simconnect_cb) {
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
        _ => Ok(()),
    }
}
