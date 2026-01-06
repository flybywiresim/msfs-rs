pub fn calculate_msfs_sdk_path() -> Result<String, &'static str> {
    if let Ok(sdk) = std::env::var("MSFS2024_SDK") {
        return Ok(sdk);
    }
    for p in ["/mnt/c/MSFS 2024 SDK", r"C:\MSFS 2024 SDK"].iter() {
        if std::path::Path::new(p).exists() {
            return Ok(p.to_string());
        }
    }
    Err("Could not locate MSFS SDK. Make sure you have it installed or try setting the MSFS_SDK env var.")
}
