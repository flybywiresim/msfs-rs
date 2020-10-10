fn main() {
    let msfs_sdk = std::env::var("MSFS_SDK").unwrap_or_else(calculate_msfs_sdk_path);
    println!("Found MSFS SDK: {:?}", msfs_sdk);

    println!("cargo:rerun-if-changed=src/wrapper.hpp");
    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}", msfs_sdk))
        .header("src/wrapper.hpp")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
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
