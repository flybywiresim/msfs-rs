use ::msfs::{
    msfs,
    sim_connect::{data_definition, SimConnect, SimConnectRecv},
};

#[data_definition]
struct ControlSurfaces {
    #[name = "ELEVATOR POSITION"]
    #[unit = "Position"]
    elevator: f64,
    #[name = "AILERON POSITION"]
    #[unit = "Position"]
    ailerons: f64,
    #[name = "RUDDER POSITION"]
    #[unit = "Position"]
    rudder: f64,
}

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
        msfs::PanelServiceID::PreInstall => {
            let sim = SimConnect::open("log", simconnect_cb).map_err(|_| ())?;
            sim.add_data_definition::<ControlSurfaces>(0)
                .map_err(|_| ())?;
            unsafe { SIM = Some(sim) };
            Ok(())
        }
        msfs::PanelServiceID::PreKill => {
            drop(unsafe { SIM.take() });
            Ok(())
        }
        _ => Ok(()),
    }
}
