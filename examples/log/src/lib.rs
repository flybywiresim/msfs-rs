use ::msfs::{
    msfs,
    msfs::MSFSEvent,
    sim_connect::{data_definition, Period, SimConnectRecv, SIMCONNECT_OBJECT_ID_USER},
};

#[data_definition]
#[derive(Debug)]
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

/// ```cfg
/// [VCockpit0]
/// size_mm=0,0
/// pixel_size=0,0
/// texture=$PDF
/// htmlgauge00=WasmInstrument/WasmInstrument.html?wasm_module=log.wasm&wasm_gauge=LOG, 0,0,0,0
/// ```
#[msfs::gauge(name=LOG)]
async fn log(mut gauge: msfs::Gauge) -> Result<(), Box<dyn std::error::Error>> {
    let mut sim = gauge.open_simconnect("LOG")?;

    while let Some(event) = gauge.next_event().await {
        println!("RUST: EVENT {:?}", event);

        match event {
            MSFSEvent::PanelServiceID(service_id) => match service_id {
                msfs::PanelServiceID::PostInstall => {
                    sim.add_data_definition::<ControlSurfaces>()?;
                    sim.request_data_on_sim_object::<ControlSurfaces>(
                        0,
                        SIMCONNECT_OBJECT_ID_USER,
                        Period::SimFrame,
                    )?;
                }
                _ => {}
            },
            MSFSEvent::SimConnect(recv) => match recv {
                SimConnectRecv::SimObjectData(event) => {
                    let data = event.into::<ControlSurfaces>(&sim).unwrap();
                    println!("{:#?}", data);
                }
                _ => {}
            },
        }
    }

    Ok(())
}
