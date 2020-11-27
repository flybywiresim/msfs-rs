use crate::sys;

#[cfg(any(target_arch = "wasm32", doc))]
pub mod legacy;

#[doc(hidden)]
pub mod executor;

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
    SimConnect(SimConnectRecv<'a>),
}

/// Gauge
pub struct Gauge {
    executor: *mut GaugeExecutor,
    rx: futures::channel::mpsc::Receiver<MSFSEvent<'static>>,
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
            executor.executor.send(Some(MSFSEvent::SimConnect(recv)));
        })?;
        Ok(sim)
    }

    /// Consume the next event from MSFS.
    pub fn next_event(&mut self) -> impl futures::Future<Output = Option<MSFSEvent<'_>>> + '_ {
        use futures::stream::StreamExt;
        async move { self.rx.next().await }
    }
}

#[doc(hidden)]
pub struct GaugeExecutor {
    pub executor: executor::Executor<Gauge, MSFSEvent<'static>>,
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
                let executor = self as *mut GaugeExecutor;
                self.executor
                    .start(Box::new(move |rx| Gauge { executor, rx }))
            }
            sys::PANEL_SERVICE_POST_KILL => self.executor.send(None),
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
                    self.executor.send(Some(MSFSEvent::PanelServiceID(data)))
                } else {
                    true
                }
            }
        }
    }

    pub fn handle_mouse(&mut self, x: f32, y: f32, flags: u32) {
        self.executor.send(Some(MSFSEvent::Mouse { x, y, flags }));
    }
}

pub struct StandaloneModule {
    executor: *mut StandaloneModuleExecutor,
    rx: futures::channel::mpsc::Receiver<SimConnectRecv<'static>>,
}

impl StandaloneModule {
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
            executor.executor.send(Some(recv));
        })?;
        Ok(sim)
    }

    /// Consume the next event from MSFS.
    pub fn next_event(&mut self) -> impl futures::Future<Output = Option<SimConnectRecv<'_>>> + '_ {
        use futures::stream::StreamExt;
        async move { self.rx.next().await }
    }
}

#[doc(hidden)]
pub struct StandaloneModuleExecutor {
    pub executor: executor::Executor<StandaloneModule, SimConnectRecv<'static>>,
}

#[doc(hidden)]
impl StandaloneModuleExecutor {
    pub fn handle_init(&mut self) {
        let executor = self as *mut StandaloneModuleExecutor;
        self.executor
            .start(Box::new(move |rx| StandaloneModule { executor, rx }));
    }

    pub fn handle_deinit(&mut self) {
        self.executor.send(None);
    }
}
