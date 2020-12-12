use crate::sys;

#[cfg(any(target_arch = "wasm32", doc))]
pub mod legacy;

#[cfg(any(target_arch = "wasm32", doc))]
pub mod nvg;

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

use crate::sim_connect::{SimConnect, SimConnectRecv};
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
            executor
                .executor
                .send(Some(MSFSEvent::SimConnect(recv)))
                .unwrap();
        })?;
        Ok(sim)
    }

    /// Create a NanoVG rendering context. See `Context` for more details.
    #[cfg(any(target_arch = "wasm32", doc))]
    pub fn create_nanovg(&self) -> Option<nvg::Context> {
        nvg::Context::create(unsafe { (*self.executor).fs_ctx })
    }

    /// Consume the next event from MSFS.
    pub fn next_event(&mut self) -> impl futures::Future<Output = Option<MSFSEvent<'_>>> + '_ {
        use futures::stream::StreamExt;
        async move { self.rx.next().await }
    }
}

#[doc(hidden)]
pub struct GaugeExecutor {
    fs_ctx: sys::FsContext,
    pub executor: executor::Executor<Gauge, MSFSEvent<'static>>,
}

#[doc(hidden)]
impl GaugeExecutor {
    pub fn handle_gauge(
        &mut self,
        ctx: sys::FsContext,
        service_id: std::os::raw::c_int,
        p_data: *mut std::ffi::c_void,
    ) -> bool {
        match service_id as u32 {
            sys::PANEL_SERVICE_PRE_INSTALL => {
                let executor = self as *mut GaugeExecutor;
                self.fs_ctx = ctx;
                self.executor
                    .start(Box::new(move |rx| Gauge { executor, rx }))
                    .is_ok()
            }
            sys::PANEL_SERVICE_POST_KILL => self.executor.send(None).is_ok(),
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
                    self.executor
                        .send(Some(MSFSEvent::PanelServiceID(data)))
                        .is_ok()
                } else {
                    true
                }
            }
        }
    }

    pub fn handle_mouse(&mut self, x: f32, y: f32, flags: u32) {
        self.executor
            .send(Some(MSFSEvent::Mouse { x, y, flags }))
            .unwrap();
    }
}

pub struct StandaloneModule {
    executor: *mut StandaloneModuleExecutor,
    rx: futures::channel::mpsc::Receiver<SimConnectRecv<'static>>,
}

impl StandaloneModule {
    /// Send a request to the Microsoft Flight Simulator server to open up communications with a new client.
    pub fn open_simconnect(
        &mut self,
        name: &str,
    ) -> Result<std::pin::Pin<Box<SimConnect>>, Box<dyn std::error::Error>> {
        let executor = self.executor;
        let mut sim = SimConnect::open(name, move |_sim, recv| {
            let executor = unsafe { &mut *executor };
            let recv =
                unsafe { std::mem::transmute::<SimConnectRecv<'_>, SimConnectRecv<'static>>(recv) };
            executor.executor.send(Some(recv)).unwrap();
        })?;
        if let Some(ref mut list) = unsafe { self.executor.as_mut() }.unwrap().simconnects {
            list.push(sim.as_mut_ptr());
        }
        Ok(sim)
    }

    /// Consume the next event from MSFS.
    pub fn next_event(&mut self) -> impl futures::Future<Output = Option<SimConnectRecv<'_>>> + '_ {
        use futures::stream::StreamExt;
        async move { self.rx.next().await }
    }

    pub fn simulate<Fut: 'static, F: Fn(StandaloneModule) -> Fut>(
        _f: F,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        Fut: futures::Future<Output = Result<(), Box<dyn std::error::Error>>>,
    {
        let mut e = StandaloneModuleExecutor {
            executor: executor::Executor {
                handle: |m| {
                    assert!(std::mem::size_of::<F>() == 0);
                    let f: F = unsafe { std::mem::MaybeUninit::zeroed().assume_init() };
                    Box::pin(f(m))
                },
                future: None,
                tx: None,
            },
            simconnects: Some(vec![]),
        };

        e.start()?;

        loop {
            std::thread::sleep(std::time::Duration::from_millis(10));
            let simconnects = e.simconnects.as_ref().unwrap();
            if simconnects.is_empty() {
                break;
            }
            for s in simconnects {
                unsafe {
                    (&mut **s).call_dispatch()?;
                }
            }
        }

        e.end()?;

        Ok(())
    }
}

#[doc(hidden)]
pub struct StandaloneModuleExecutor {
    pub executor: executor::Executor<StandaloneModule, SimConnectRecv<'static>>,
    pub simconnects: Option<Vec<*mut SimConnect>>,
}

#[doc(hidden)]
impl StandaloneModuleExecutor {
    fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let executor = self as *mut StandaloneModuleExecutor;
        self.executor
            .start(Box::new(move |rx| StandaloneModule { executor, rx }))
    }

    pub fn handle_init(&mut self) {
        self.start().unwrap();
    }

    fn end(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.executor.send(None)
    }

    pub fn handle_deinit(&mut self) {
        self.end().unwrap();
    }
}
