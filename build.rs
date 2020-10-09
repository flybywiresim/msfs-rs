fn main() {
    println!("cargo:rerun-if-changed=src/wrapper.hpp");

    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}", std::env::var("MSFS_SDK").unwrap()))
        .header("src/wrapper.hpp")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .unwrap();
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .unwrap();
}
