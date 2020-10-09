use crate::sys;

/// MSFS Context. Hashable.
#[derive(PartialEq, Hash)]
pub struct FsContext(sys::FsContext);

impl From<sys::FsContext> for FsContext {
    fn from(ctx: sys::FsContext) -> FsContext {
        FsContext(ctx)
    }
}

/// `PanelServiceID` is used in `GaugeCallback`s and is generated from
/// `sys::PANEL_SERVICE_*` constants.
#[repr(u32)]
#[derive(PartialEq, Clone, Copy)]
pub enum PanelServiceID {
    PreQuery = sys::PANEL_SERVICE_PRE_QUERY,
    PostQuery = sys::PANEL_SERVICE_POST_QUERY,
    PreInstall = sys::PANEL_SERVICE_PRE_INSTALL,
    PostInstall = sys::PANEL_SERVICE_POST_INSTALL,
    PreInitialize = sys::PANEL_SERVICE_PRE_INITIALIZE,
    PostInitializer = sys::PANEL_SERVICE_POST_INITIALIZE,
    PreUpdate = sys::PANEL_SERVICE_PRE_UPDATE,
    PostUpdate = sys::PANEL_SERVICE_POST_UPDATE,
    PreGenerate = sys::PANEL_SERVICE_PRE_GENERATE,
    PostGenerate = sys::PANEL_SERVICE_POST_GENERATE,
    PreDraw = sys::PANEL_SERVICE_PRE_DRAW,
    PostDraw = sys::PANEL_SERVICE_POST_DRAW,
    PreKill = sys::PANEL_SERVICE_PRE_KILL,
    PostKill = sys::PANEL_SERVICE_POST_KILL,
    ConnectToWindow = sys::PANEL_SERVICE_CONNECT_TO_WINDOW,
    Disconnect = sys::PANEL_SERVICE_DISCONNECT,
    PanelOpen = sys::PANEL_SERVICE_PANEL_OPEN,
    PanelClose = sys::PANEL_SERVICE_PANEL_CLOSE,
}

/// MSFS Gauges are managed using this lifecycle callback.
pub type GaugeCallback = fn(&FsContext, PanelServiceID) -> GuageCallbackResult;
pub type GuageCallbackResult = Result<(), ()>;

/// `gauge!` is used to generate the exported function which is exposed to
/// MSFS when it loads the WASM binary. Use it like so:
/// ```rs
/// fn my_gauge(ctx: &FsContext, servce_id: PanelServiceID) -> GaugeResult {
///     Ok(())
/// }
/// gauge!(my_gauge);
/// ```
// FIXME: this should be a proc macro like:
// ```rs
// #[msfs::gauge]
// fn XYZ(ctx: &FsContext, service_id: PanelServiceID) {
//   // ...
// }
// ```
// which generates XYZ_gauge_callback
#[macro_export]
macro_rules! gauge {
    ($name:ident) => {
        #[no_mangle]
        pub extern "C" fn RUST_gauge_callback(
            ctx: $crate::sys::FsContext,
            service_id: i32,
        ) -> bool {
            let external: $crate::GaugeCallback = $name;
            let ctx = $crate::FsContext::from(ctx);
            let service_id = service_id as PanelServiceID;
            match external(&ctx, service_id) {
                Ok(()) => true,
                Err(()) => false,
            }
        }
    };
}
