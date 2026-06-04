pub fn calculate_msfs_sdk_path() -> Result<String, String> {
    let sdk_env_key = if cfg!(feature = "msfs_2024") {
        "MSFS2024_SDK"
    } else {
        "MSFS_SDK"
    };
    let sdk_normal_dirname = if cfg!(feature = "msfs_2024") {
        "MSFS 2024 SDK"
    } else {
        "MSFS SDK"
    };

    if let Ok(sdk) = std::env::var(sdk_env_key) {
        return Ok(sdk);
    }
    for p in [
        format!("/mnt/c/{sdk_normal_dirname}"),
        format!("C:\\{sdk_normal_dirname}"),
    ]
    .iter()
    {
        if std::path::Path::new(p).exists() {
            return Ok(p.to_string());
        }
    }
    Err(format!(
        "Could not locate {sdk_normal_dirname}. Make sure you have it installed or try setting the {sdk_env_key} env var."
    ))
}
