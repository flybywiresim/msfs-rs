![FlyByWire Simulations](https://raw.githubusercontent.com/flybywiresim/branding/1391fc003d8b5d439d01ad86e2778ae0bfc8b682/tails-with-text/FBW-Color-Light.svg#gh-dark-mode-only)

# ✈️ msfs-rs 🦀

**Use Rust to interact with Microsoft Flight Simulator**

[**As seen in the FlyByWire A32NX and A380X!**](https://flybywiresim.com/projects/)

* Write custom aircraft logic in Rust
  * Systems
  * Gauges
  * Instrument panels (NanoVG)
  * Replace aircraft code
    * RPL
    * C++
    * JavaScript
    * TypeScript
    * WASM
* Interact with the Microsoft SDK
* Create external applications that interact with MSFS using SimConnect

[![Discord](https://img.shields.io/discord/738864299392630914.svg?label=&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.gg/flybywire)
[![X](https://img.shields.io/badge/-@FlyByWireSim-e84393?label=&logo=X&logoColor=ffffff&color=6399AE&labelColor=00C2CB)](https://x.com/FlybywireSim)
[![YouTube](https://img.shields.io/badge/-FlyByWireSimulations-e84393?label=&logo=youtube&logoColor=ffffff&color=6399AE&labelColor=00C2CB)](https://www.youtube.com/c/FlyByWire-Simulations)
[![Facebook](https://img.shields.io/badge/-FlyByWireSimulations-e84393?label=&logo=facebook&logoColor=ffffff&color=6399AE&labelColor=00C2CB)](https://www.facebook.com/FlyByWireSimulations/)
[![Instagram](https://img.shields.io/badge/-@FlyByWireSim-e84393?label=&logo=instagram&logoColor=ffffff&color=6399AE&labelColor=00C2CB)](https://instagram.com/flybywiresim)
[![Bluesky](https://img.shields.io/badge/-@FlyByWireSim-e84393?label=&logo=Bluesky&logoColor=ffffff&color=6399AE&labelColor=00C2CB)](https://bsky.app/profile/flybywiresim.com)

---

## Example
Drawing a red rectangle on a glass instrument panel inside an aircraft cockpit:

```rust
// gauge_logic.rs
use msfs::{nvg, MSFSEvent};

#[msfs::gauge(name=Demo)]
async fn demo(mut gauge: msfs::Gauge) -> Result<(), Box<dyn std::error::Error>> {
    // Use NanoVG to draw
    let nvg = gauge.create_nanovg().unwrap();

    let black = nvg::Style::default().fill(nvg::Color::from_rgb(0, 0, 0));
    let red = nvg::Style::default().fill(nvg::Color::from_rgb(255, 0, 0));

    // Reacting to MSFS events
    while let Some(event) = gauge.next_event().await {
        match event {
            MSFSEvent::PreDraw(d) => {
                nvg.draw_frame(d.width(), d.height(), |f| {
                    // Red rectangle
                    f.draw_path(&red, |p| {
                        p.rect(20.0, 20.0, 40.0, 40.0);
                        println!("Hello rusty world!");

                        Ok(())
                    })?;

                    Ok(())
                });
            }
            _ => {}
        }
    }
    Ok(())
}
```

```
[VCockpit01]
size_mm=1024,768
pixel_size=1024,768
texture=$SCREEN_1
background_color=0,0,255
htmlgauge00=WasmInstrument/WasmInstrument.html?wasm_module=gauge_logic.wasm&wasm_gauge=Demo, 0,0,1024,768
```
<img width="700" alt="grafik" src="https://github.com/user-attachments/assets/babe0670-51bf-4a33-8fc6-5e282667febe" />


---

## Getting started
* [Check out the examples](examples/)
* [Download the MSFS SDK](https://docs.flightsimulator.com/html/Introduction/SDK_Overview.htm)

## Further reading
* [Documentation](https://flybywiresim.github.io/msfs-rs/msfs/)
* [Microsoft Flight Simulator SDK](https://docs.flightsimulator.com/html/Introduction/Introduction.htm)
  * [Creating an aircraft with custom logic](https://docs.flightsimulator.com/html/Samples_And_Tutorials/Samples/SimObjects_Aircraft/GaugeAircraft.htm)
  * [Programming APIs](https://docs.flightsimulator.com/html/Programming_Tools/Programming_APIs.htm)
  * [Microsoft's Samples and Examples](https://docs.flightsimulator.com/html/Samples_And_Tutorials/Samples_And_Tutorials.htm)
  * [External interaction with SimConnect](https://docs.flightsimulator.com/html/Programming_Tools/SimConnect/SimConnect_SDK.htm)

## Help
* Join the [Discord #rust-lang channel](https://discord.gg/flybywire)
