extern crate msfs;

#[msfs::msfs::gauge]
pub fn x(
    _: &msfs::msfs::FsContext,
    _: msfs::msfs::PanelServiceID,
) -> msfs::msfs::GaugeCallbackResult {
    Ok(())
}

#[test]
fn test() {}
