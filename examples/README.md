## Nvg
NanoVG demo to show how to draw a custom Gauge in an aircraft.

* Compiles to WASM to be [loaded by MSFS](https://docs.flightsimulator.com/html/mergedProjects/How_To_Make_An_Aircraft/Contents/Instruments/Creating_WASM_Gauges.htm).
* Similar to how the MSFS Sample [GaugeAircraft](https://docs.flightsimulator.com/html/Samples_And_Tutorials/Samples/SimObjects_Aircraft/GaugeAircraft.htm) works.

#### Building
```bash
$ cd examples/nvg
$ cargo build --target wasm32-wasip1
```
#### Running
To see it in action you have to load an aircraft into MSFS which uses the compiled wasm file.

[MrMinimal's Rust Aircraft Template](https://github.com/MrMinimal/msfs-rust-aircraft) shows how that can be done.

---

## Client Data
SimConnect demo to show how a standalone application would interact with MSFS.

---

## Log
SimConnect demo printing data from MSFS in a standalone application.

---

## Other Projects / Aircraft
If you want to see msfs-rs fully integrated in working aircraft
* [FlyByWire A32X and A380](https://github.com/flybywiresim/aircraft)
