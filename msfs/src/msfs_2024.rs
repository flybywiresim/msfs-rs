use crate::{executor, sys};

use crate::sim_connect::{SimConnect, SimConnectRecv};
pub use msfs_derive::{gauge2024, system};

/// Used in Systems to dispatch lifetime events and SimConnect events.
#[derive(Debug)]
pub enum MSFS2024SystemEvent<'a> {
    Init(sys::sSystemInstallData),
    Update(std::os::raw::c_float),
    Kill,
    SimConnect(SimConnectRecv<'a>),
}

/// System
pub struct System {
    executor: *mut SystemExecutor,
    rx: futures::channel::mpsc::Receiver<MSFS2024SystemEvent<'static>>,
}

impl System {
    /// Send a request to the Microsoft Flight Simulator server to open up communications with a new client.
    pub fn open_simconnect<'a>(
        &self,
        name: &str,
    ) -> Result<std::pin::Pin<Box<SimConnect<'a>>>, Box<dyn std::error::Error>> {
        let executor = self.executor;
        let sim = SimConnect::open(name, move |_sim, recv| {
            let executor = unsafe { &mut *executor };
            let recv =
                unsafe { std::mem::transmute::<SimConnectRecv<'_>, SimConnectRecv<'static>>(recv) };
            executor
                .executor
                .send(Some(MSFS2024SystemEvent::SimConnect(recv)))
                .unwrap();
        })?;
        Ok(sim)
    }

    /// Create a NanoVG rendering context. See `Context` for more details.
    #[cfg(any(target_arch = "wasm32", doc))]
    pub fn create_nanovg(&self) -> Option<crate::nvg::Context> {
        crate::nvg::Context::create(unsafe { (*self.executor).fs_ctx.unwrap() })
    }

    /// Consume the next event from MSFS.
    pub fn next_event(
        &mut self,
    ) -> impl futures::Future<Output = Option<MSFS2024SystemEvent<'_>>> + '_ {
        use futures::stream::StreamExt;
        async move { self.rx.next().await }
    }
}

#[doc(hidden)]
pub struct SystemExecutor {
    pub fs_ctx: Option<sys::FsContext>,
    pub executor: executor::Executor<System, MSFS2024SystemEvent<'static>>,
}

#[doc(hidden)]
impl SystemExecutor {
    pub fn handle_systems_init(
        &mut self,
        ctx: sys::FsContext,
        p_install_data: &sys::sSystemInstallData,
    ) -> bool {
        let executor = self as *mut SystemExecutor;
        self.fs_ctx = Some(ctx);
        self.executor
            .start(Box::new(move |rx| System { executor, rx }))
            .and_then(|()| {
                self.executor
                    .send(Some(MSFS2024SystemEvent::Init(*p_install_data)))
            })
            .is_ok()
    }

    pub fn handle_systems_update(
        &mut self,
        _ctx: sys::FsContext,
        d_time: std::os::raw::c_float,
    ) -> bool {
        self.executor
            .send(Some(MSFS2024SystemEvent::Update(d_time)))
            .is_ok()
    }

    pub fn handle_systems_kill(&mut self, _ctx: sys::FsContext) -> bool {
        self.executor
            .send(Some(MSFS2024SystemEvent::Kill))
            .and_then(|()| self.executor.send(None))
            .is_ok()
    }
}

/// Used in MSFS 2024 Gauges to dispatch lifetime events, mouse events, and SimConnect events.
#[derive(Debug)]
pub enum MSFS2024GaugeEvent<'a> {
    Init(sys::sGaugeInstallData),
    Update(std::os::raw::c_float),
    Draw(sys::sGaugeDrawData),
    Kill,
    Mouse { x: f32, y: f32, flags: i32 },
    SimConnect(SimConnectRecv<'a>),
}

/// MSFS 2024 Gauge
pub struct Gauge2024 {
    executor: *mut Gauge2024Executor,
    rx: futures::channel::mpsc::Receiver<MSFS2024GaugeEvent<'static>>,
}

impl Gauge2024 {
    /// Send a request to the Microsoft Flight Simulator server to open up communications with a new client.
    pub fn open_simconnect<'a>(
        &self,
        name: &str,
    ) -> Result<std::pin::Pin<Box<SimConnect<'a>>>, Box<dyn std::error::Error>> {
        let executor = self.executor;
        let sim = SimConnect::open(name, move |_sim, recv| {
            let executor = unsafe { &mut *executor };
            let recv =
                unsafe { std::mem::transmute::<SimConnectRecv<'_>, SimConnectRecv<'static>>(recv) };
            executor
                .executor
                .send(Some(MSFS2024GaugeEvent::SimConnect(recv)))
                .unwrap();
        })?;
        Ok(sim)
    }

    /// Create a NanoVG rendering context. See `Context` for more details.
    #[cfg(any(target_arch = "wasm32", doc))]
    pub fn create_nanovg(&self) -> Option<crate::nvg::Context> {
        crate::nvg::Context::create(unsafe { (*self.executor).fs_ctx.unwrap() })
    }

    /// Consume the next event from MSFS.
    pub fn next_event(
        &mut self,
    ) -> impl futures::Future<Output = Option<MSFS2024GaugeEvent<'_>>> + '_ {
        use futures::stream::StreamExt;
        async move { self.rx.next().await }
    }
}

#[doc(hidden)]
pub struct Gauge2024Executor {
    pub fs_ctx: Option<sys::FsContext>,
    pub executor: executor::Executor<Gauge2024, MSFS2024GaugeEvent<'static>>,
}

#[doc(hidden)]
impl Gauge2024Executor {
    pub fn handle_gauge_init(
        &mut self,
        ctx: sys::FsContext,
        p_install_data: &sys::sGaugeInstallData,
    ) -> bool {
        let executor = self as *mut Gauge2024Executor;
        self.fs_ctx = Some(ctx);
        self.executor
            .start(Box::new(move |rx| Gauge2024 { executor, rx }))
            .and_then(|()| {
                self.executor
                    .send(Some(MSFS2024GaugeEvent::Init(*p_install_data)))
            })
            .is_ok()
    }

    pub fn handle_gauge_update(
        &mut self,
        _ctx: sys::FsContext,
        d_time: std::os::raw::c_float,
    ) -> bool {
        self.executor
            .send(Some(MSFS2024GaugeEvent::Update(d_time)))
            .is_ok()
    }

    pub fn handle_gauge_draw(
        &mut self,
        _ctx: sys::FsContext,
        p_draw_data: &sys::sGaugeDrawData,
    ) -> bool {
        self.executor
            .send(Some(MSFS2024GaugeEvent::Draw(*p_draw_data)))
            .is_ok()
    }

    pub fn handle_gauge_kill(&mut self, _ctx: sys::FsContext) -> bool {
        self.executor
            .send(Some(MSFS2024GaugeEvent::Kill))
            .and_then(|()| self.executor.send(None))
            .is_ok()
    }

    pub fn handle_mouse(&mut self, _ctx: sys::FsContext, x: f32, y: f32, flags: i32) {
        self.executor
            .send(Some(MSFS2024GaugeEvent::Mouse { x, y, flags }))
            .unwrap();
    }
}
