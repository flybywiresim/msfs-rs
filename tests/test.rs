extern crate msfs as msfs_root;

use msfs_root::{msfs, sim_connect::SimConnect};

#[msfs::gauge(name=FBW)]
pub fn fbw(_: &msfs::FsContext, service_id: msfs::PanelServiceID) -> msfs::GaugeCallbackResult {
    match service_id {
        msfs::PanelServiceID::PreInstall => {
            let _ = SimConnect::open("test").unwrap();
        }
        _ => {}
    }
    Ok(())
}

#[test]
fn test() {}
