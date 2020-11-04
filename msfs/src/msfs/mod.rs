use crate::sys;

use futures::{channel::mpsc, Future};
use std::pin::Pin;
use std::task::Poll;

#[cfg(target_os = "wasm32")]
pub mod legacy;

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

use crate::sim_connect::SimConnectRecv;
pub use msfs_derive::gauge;

#[derive(Debug)]
pub enum MSFSEvent<'a> {
    PanelServiceID(PanelServiceID),
    SimConnect(crate::sim_connect::SimConnectRecv<'a>),
}

/// Gauge
pub struct Gauge {
    executor: *mut GaugeExecutor,
    rx: mpsc::Receiver<MSFSEvent<'static>>,
}

impl Gauge {
    /// Send a request to the Microsoft Flight Simulator server to open up communications with a new client.
    pub fn open_simconnect(
        &self,
        name: &str,
    ) -> Result<std::pin::Pin<Box<crate::sim_connect::SimConnect>>, Box<dyn std::error::Error>>
    {
        let executor = self.executor;
        let sim = crate::sim_connect::SimConnect::open(name, move |_sim, recv| {
            let executor = unsafe { &mut *executor };
            let recv =
                unsafe { std::mem::transmute::<SimConnectRecv<'_>, SimConnectRecv<'static>>(recv) };
            executor.send(Some(MSFSEvent::SimConnect(recv)));
        })?;
        Ok(sim)
    }

    /// Consume the next event from MSFS.
    pub fn next_event(&mut self) -> impl Future<Output = Option<MSFSEvent<'_>>> + '_ {
        use futures::stream::StreamExt;
        async move { self.rx.next().await }
    }
}

type GaugeExecutorFuture =
    Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + 'static>>;
#[doc(hidden)]
pub struct GaugeExecutor {
    pub handle: fn(Gauge) -> GaugeExecutorFuture,
    pub future: Option<GaugeExecutorFuture>,
    pub tx: Option<mpsc::Sender<MSFSEvent<'static>>>,
}

#[doc(hidden)]
impl GaugeExecutor {
    pub fn handle(&mut self, _ctx: sys::FsContext, service_id: u32) -> bool {
        match service_id {
            sys::PANEL_SERVICE_PRE_INSTALL => {
                if self.future.is_none() {
                    let (tx, rx) = mpsc::channel(1);
                    self.tx = Some(tx);
                    let gauge = Gauge { executor: self, rx };
                    self.future = Some(Box::pin((self.handle)(gauge)));
                } else {
                    eprintln!("MSFS-RS: (warn) Multiple PRE_INSTALL events detected");
                }
                true
            }
            sys::PANEL_SERVICE_POST_KILL => self.send(None),
            _ => self.send(Some(MSFSEvent::PanelServiceID(unsafe {
                std::mem::transmute(service_id)
            }))),
        }
    }

    fn send(&mut self, data: Option<MSFSEvent<'static>>) -> bool {
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
