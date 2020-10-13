fn main() {
    let msfs_sdk = std::env::var("MSFS_SDK").unwrap_or_else(calculate_msfs_sdk_path);
    println!("Found MSFS SDK: {:?}", msfs_sdk);

    println!("cargo:rerun-if-changed=src/bindgen_support/wrapper.h");
    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}", msfs_sdk))
        .clang_arg(format!("-I{}", "src/bindgen_support"))
        .clang_arg("-fms-extensions")
        .clang_arg("-xc++")
        .header("src/bindgen_support/wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .impl_debug(true)
        .generate()
        .unwrap();
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .unwrap();
}

fn calculate_msfs_sdk_path(_: std::env::VarError) -> String {
    for p in ["/mnt/c/MSFS SDK", r"C:\MSFS SDK"].iter() {
        if std::path::Path::new(p).exists() {
            return p.to_string();
        }
    }
    panic!("Could not locate MSFS SDK. Make sure you have it installed or try setting the MSFS_SDK env var.");
}
