use std::path::{Path, PathBuf};

fn main() {
    let msfs_sdk_path = msfs_sdk_path();
    println!("Found MSFS SDK: {}", msfs_sdk_path);

    println!("cargo:rerun-if-changed=src/bindgen_support/wrapper.h");
    let bindings = bindgen::Builder::default()
        // On 2020-10-16, we observed what command line args were passed to clang by
        // MSFS SDK (version 0.6.0.0) when building a WASM gauge from Visual Studio 2019 using the
        // SDK's provided extension (the useless ones were thrown out).

        // It's unlikely that these variables will significantly change over the course of
        // MSFS SDK's lifecycle, but if they do -- the values can be recaptured by going to:
        //   Tools -> Options -> Projects and Solutions -> Build and Run
        // And setting:
        //   MSBuild project build output verbosity = Diagnostic

        // Then the command used to build the file will show up. (Search for "clang-cl.exe")
        // A caveat is that the version of clang used by the MSFS SDK is a version with
        // compatibility with MSVC's cl.exe, which means the flags will be non-standard.

        // The cl.exe flags can be read about here, and it should be a straightforward decoding:
        // https://docs.microsoft.com/en-us/cpp/build/reference/compiler-options?view=vs-2019
        .clang_arg(format!("-I{}/WASM/wasi-sysroot/include", &msfs_sdk_path))
        .clang_arg(format!("-I{}/WASM/wasi-sysroot/include/c++/v1", &msfs_sdk_path))
        .clang_arg(format!("-I{}/WASM/include", &msfs_sdk_path))
        .clang_arg(format!("-I{}/SimConnect SDK/include", &msfs_sdk_path))
        .clang_arg(format!("-D {}", get_debug_macro()))
        .clang_arg("-D _MSFS_WASM")
        .clang_arg("-D _STRING_H_CPLUSPLUS_98_CONFORMANCE_")
        .clang_arg("-D _WCHAR_H_CPLUSPLUS_98_CONFORMANCE_")
        .clang_arg("-D _LIBCPP_NO_EXCEPTIONS")
        .clang_arg("-D _LIBCPP_HAS_NO_THREADS")
        .clang_arg("-D _MBCS")
        .clang_arg("-fms-extensions")
        .clang_arg("-m32")
        .clang_arg("-xc++")
        .clang_arg("--target=wasm32-unknown-wasi")
        .clang_arg(format!("--sysroot={}/WASM/wasi-sysroot", &msfs_sdk_path))
        .header("src/bindgen_support/wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .impl_debug(true)
        .generate()
        .unwrap();
    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .unwrap();
}

/// This function returns the location of the MSFS SDK.
///
/// It will first try to use the MSFS_SDK environment variable, and then check in the default
/// install directories.
fn msfs_sdk_path() -> String {
    if let Ok(path) = std::env::var("MSFS_SDK") {
        println!("MSFS_SDK environment variable is set to: {}", path);
        if Path::new(&path).exists() {
            println!("MSFS_SDK environment variable points to a path that exists");
            return path;
        }
        println!("MSFS_SDK environment variable points to a path that is invalid or does not exist");
    } else {
        println!("MSFS_SDK environment variable is not set");
    }

    let path;
    let environment;
    if cfg!(windows) {
        path = r"C:\MSFS SDK";
        environment = "Windows";
    } else {
        path = r"/mnt/c/MSFS SDK";
        environment = "non-Windows";
    }
    println!("Detected {}: Using the MSFS SDK default path of {}", environment, path);

    if Path::new(path).exists() {
        println!("Default MSFS SDK path exists");
        return path.to_owned();
    }

    panic!("Unable to find a valid path to the MSFS SDK. \
           Check the preceding stdout messages, \
           and potentially set the MSFS_SDK environment variable to the right path.");
}

fn get_debug_macro() -> &'static str {
    return if cfg!(debug_assertions) {
        "_DEBUG"
    } else {
        "NDEBUG"
    }
}