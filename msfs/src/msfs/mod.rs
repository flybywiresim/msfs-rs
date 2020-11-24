use crate::sys;

use futures::{channel::mpsc, Future};
use std::pin::Pin;
use std::task::Poll;

#[cfg(any(target_arch = "wasm32", doc))]
pub mod legacy;

/// `PanelServiceID` is used in `GaugeCallback`.
#[derive(Debug)]
pub enum PanelServiceID<'a> {
    PostInstall,
    PreInitialize,
    PostInitialize,
    PreUpdate,
    PostUpdate,
    PreDraw(&'a sys::sGaugeDrawData),
    PostDraw(&'a sys::sGaugeDrawData),
    PreKill,
}

use crate::sim_connect::SimConnectRecv;
pub use msfs_derive::{gauge, standalone_module};

#[derive(Debug)]
pub enum MSFSEvent<'a> {
    PanelServiceID(PanelServiceID<'a>),
    Mouse { x: f32, y: f32, flags: u32 },
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
    pub fn handle_gauge(
        &mut self,
        _ctx: sys::FsContext,
        service_id: std::os::raw::c_int,
        p_data: *mut std::ffi::c_void,
    ) -> bool {
        match service_id as u32 {
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
            service_id => {
                if let Some(data) = match service_id {
                    sys::PANEL_SERVICE_POST_INSTALL => Some(PanelServiceID::PostInstall),
                    sys::PANEL_SERVICE_PRE_INITIALIZE => Some(PanelServiceID::PreInitialize),
                    sys::PANEL_SERVICE_POST_INITIALIZE => Some(PanelServiceID::PostInitialize),
                    sys::PANEL_SERVICE_PRE_UPDATE => Some(PanelServiceID::PreUpdate),
                    sys::PANEL_SERVICE_POST_UPDATE => Some(PanelServiceID::PostUpdate),
                    sys::PANEL_SERVICE_PRE_DRAW => Some(PanelServiceID::PreDraw(unsafe {
                        &*(p_data as *const sys::sGaugeDrawData)
                    })),
                    sys::PANEL_SERVICE_POST_DRAW => Some(PanelServiceID::PostDraw(unsafe {
                        &*(p_data as *const sys::sGaugeDrawData)
                    })),
                    sys::PANEL_SERVICE_PRE_KILL => Some(PanelServiceID::PreKill),
                    _ => None,
                } {
                    self.send(Some(MSFSEvent::PanelServiceID(data)))
                } else {
                    true
                }
            }
        }
    }

    pub fn handle_mouse(&mut self, x: f32, y: f32, flags: u32) {
        self.send(Some(MSFSEvent::Mouse { x, y, flags }));
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
