#![crate_type = "cdylib"]

use msfs::msfs;

#[msfs::gauge(name=FBW)]
fn fbw(_: &msfs::FsContext, service_id: msfs::PanelServiceID) -> msfs::GaugeCallbackResult {
    println!("RUST: FBWCB {:?}", service_id);
    Ok(())
}
