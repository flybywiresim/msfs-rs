use crate::sys;
use futures::{channel::mpsc, Future};
use std::pin::Pin;
use std::task::Poll;

/// `PanelServiceID` is used in `GaugeCallback`s and is generated from
/// `sys::PANEL_SERVICE_*` constants.
#[repr(u32)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PanelServiceID {
    PreQuery = sys::PANEL_SERVICE_PRE_QUERY,
    PostQuery = sys::PANEL_SERVICE_POST_QUERY,
    // PreInstall = sys::PANEL_SERVICE_PRE_INSTALL,
    PostInstall = sys::PANEL_SERVICE_POST_INSTALL,
    PreInitialize = sys::PANEL_SERVICE_PRE_INITIALIZE,
    PostInitialize = sys::PANEL_SERVICE_POST_INITIALIZE,
    PreUpdate = sys::PANEL_SERVICE_PRE_UPDATE,
    PostUpdate = sys::PANEL_SERVICE_POST_UPDATE,
    PreGenerate = sys::PANEL_SERVICE_PRE_GENERATE,
    PostGenerate = sys::PANEL_SERVICE_POST_GENERATE,
    PreDraw = sys::PANEL_SERVICE_PRE_DRAW,
    PostDraw = sys::PANEL_SERVICE_POST_DRAW,
    PreKill = sys::PANEL_SERVICE_PRE_KILL,
    // PostKill = sys::PANEL_SERVICE_POST_KILL,
    ConnectToWindow = sys::PANEL_SERVICE_CONNECT_TO_WINDOW,
    Disconnect = sys::PANEL_SERVICE_DISCONNECT,
    PanelOpen = sys::PANEL_SERVICE_PANEL_OPEN,
    PanelClose = sys::PANEL_SERVICE_PANEL_CLOSE,
}

/// Bindings to the Legacy/gauges.h API
pub struct Legacy {}
impl Legacy {
    /// aircraft_varget
    pub fn aircraft_varget(simvar: sys::ENUM, units: sys::ENUM, index: sys::SINT32) -> f64 {
        unsafe { sys::aircraft_varget(simvar, units, index) }
    }

    /// get_aircraft_var_enum
    pub fn get_aircraft_var_enum(name: &str) -> sys::ENUM {
        unsafe {
            let name = std::ffi::CString::new(name).unwrap();
            sys::get_aircraft_var_enum(name.as_ptr())
        }
    }

    /// get_units_enum
    pub fn get_units_enum(unitname: &str) -> sys::ENUM {
        unsafe {
            let name = std::ffi::CString::new(unitname).unwrap();
            sys::get_units_enum(name.as_ptr())
        }
    }
}

use crate::sim_connect::SimConnectRecv;
pub use msfs_derive::gauge;

#[derive(Debug)]
pub enum MSFSEvent {
    PanelServiceID(PanelServiceID),
    SimConnect(crate::sim_connect::SimConnectRecv<'static>),
}

/// Gauge
pub struct Gauge {
    executor: *mut GaugeExecutor,
    rx: mpsc::Receiver<MSFSEvent>,
}

impl Gauge {
    /// Send a request to the Microsoft Flight Simulator server to open up communications with a new client.
    pub fn open_simconnect(
        &self,
        name: &str,
    ) -> Result<crate::sim_connect::SimConnect, Box<dyn std::error::Error>> {
        let executor = self.executor;
        let sim = crate::sim_connect::SimConnect::open(name, move |_sim, recv| {
            let executor = unsafe { &mut *executor };
            let recv: SimConnectRecv<'static> = unsafe { std::mem::transmute(recv) };
            executor.send(Some(MSFSEvent::SimConnect(recv)));
        })?;
        Ok(sim)
    }

    /// Consume the next event from MSFS.
    pub fn next_event(&mut self) -> impl Future<Output = Option<MSFSEvent>> + '_ {
        use futures::stream::StreamExt;
        self.rx.next()
    }
}

type GaugeExecutorFuture =
    Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + 'static>>;
#[doc(hidden)]
pub struct GaugeExecutor {
    pub handle: fn(Gauge) -> GaugeExecutorFuture,
    pub future: Option<GaugeExecutorFuture>,
    pub tx: Option<mpsc::Sender<MSFSEvent>>,
}

#[doc(hidden)]
impl GaugeExecutor {
    pub fn handle(&mut self, _ctx: sys::FsContext, service_id: u32) -> bool {
        match service_id {
            sys::PANEL_SERVICE_PRE_INSTALL => {
                let (tx, rx) = mpsc::channel(1);
                self.tx = Some(tx);
                let gauge = Gauge { executor: self, rx };
                self.future = Some(Box::pin((self.handle)(gauge)));
                true
            }
            sys::PANEL_SERVICE_POST_KILL => self.send(None),
            _ => self.send(Some(MSFSEvent::PanelServiceID(unsafe {
                std::mem::transmute(service_id)
            }))),
        }
    }

    fn send(&mut self, data: Option<MSFSEvent>) -> bool {
        if let Some(data) = data {
            self.tx.as_mut().unwrap().try_send(data).unwrap();
        } else {
            self.tx.take();
        }
        let mut context = std::task::Context::from_waker(futures::task::noop_waker_ref());
        match self.future.as_mut().unwrap().as_mut().poll(&mut context) {
            Poll::Pending => true,
            Poll::Ready(v) => v.is_ok(),
        }
    }
}
