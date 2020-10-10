# msfs-rs

These bindings include:

- MSFS Gauge API
- SimConnect API

## Building

If your MSFS SDK is not installed to `C:\MSFS SDK` you will need to set the
`MSFS_SDK` env variable to the correct path.

## Known Issues

Until https://github.com/rust-lang/rfcs/issues/2771 is fixed, you will have to
manually modify your output wasm file to re-export `malloc` and `free`. It
usually looks something like this:

```wat
(export "malloc" (func $malloc))
(export "free" (func $free))
```
