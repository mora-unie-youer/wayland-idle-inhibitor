use std::{
    io::{BufRead, BufReader, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use wayland_client::{
    protocol::{wl_compositor, wl_registry, wl_surface},
    Connection, Dispatch, EventQueue, QueueHandle,
};
use wayland_protocols::wp::idle_inhibit::zv1::client::{
    zwp_idle_inhibit_manager_v1, zwp_idle_inhibitor_v1,
};

use super::socket::IdleInhibitorSocket;

#[derive(Debug)]
pub struct IdleInhibitorDaemon {
    pub terminate: Arc<AtomicBool>,
    queue_handle: QueueHandle<Self>,

    base_surface: Option<wl_surface::WlSurface>,
    idle_inhibit_manager: Option<zwp_idle_inhibit_manager_v1::ZwpIdleInhibitManagerV1>,
    idle_inhibitor: Option<zwp_idle_inhibitor_v1::ZwpIdleInhibitorV1>,
}

impl IdleInhibitorDaemon {
    pub fn new(event_queue: &mut EventQueue<Self>) -> Self {
        let terminate = Arc::new(AtomicBool::new(false));
        signal_hook::flag::register(signal_hook::consts::SIGINT, terminate.clone())
            .expect("Couldn't setup SIGINT hook");
        signal_hook::flag::register(signal_hook::consts::SIGTERM, terminate.clone())
            .expect("Couldn't setup SIGTERM hook");

        let queue_handle = event_queue.handle();
        let mut state = Self {
            terminate,
            queue_handle,

            base_surface: None,
            idle_inhibit_manager: None,
            idle_inhibitor: None,
        };

        // Initializing Wayland client
        state.roundtrip(event_queue);
        state
    }

    fn roundtrip(&mut self, event_queue: &mut EventQueue<Self>) {
        event_queue.roundtrip(self).unwrap();
    }

    pub fn run(&mut self, event_queue: &mut EventQueue<Self>, socket: IdleInhibitorSocket) {
        let mut incoming = socket.incoming();
        while !self.terminate.load(Ordering::SeqCst) {
            if let Some(Ok(mut client)) = incoming.next() {
                let reader = BufReader::new(client.try_clone().unwrap());
                let command = reader.lines().next().unwrap().unwrap();
                match &command[..] {
                    "status" => client.write_all(&[self.is_enabled() as u8]).unwrap(),
                    "disable" => self.disable_idle_inhibit(),
                    "enable" => self.enable_idle_inhibit(),
                    "toggle" => self.toggle_idle_inhibit(),
                    _ => (),
                }

                self.roundtrip(event_queue);
            } else {
                // Process socket every 1 second
                std::thread::sleep(Duration::from_secs(1));
            }
        }
    }

    pub fn create_idle_inhibitor(&self) -> zwp_idle_inhibitor_v1::ZwpIdleInhibitorV1 {
        if self.base_surface.is_none() || self.idle_inhibit_manager.is_none() {
            panic!("Surface and idle inhibit manager were not initialized");
        }

        let surface = self.base_surface.as_ref().unwrap();
        let idle_inhibit_manager = self.idle_inhibit_manager.as_ref().unwrap();
        idle_inhibit_manager.create_inhibitor(surface, &self.queue_handle, ())
    }

    pub fn toggle_idle_inhibit(&mut self) {
        if let Some(idle_inhibitor) = &self.idle_inhibitor {
            idle_inhibitor.destroy();
            self.idle_inhibitor = None;
        } else {
            self.idle_inhibitor = Some(self.create_idle_inhibitor());
        }
    }

    pub fn enable_idle_inhibit(&mut self) {
        if self.idle_inhibitor.is_none() {
            self.idle_inhibitor = Some(self.create_idle_inhibitor());
        }
    }

    pub fn disable_idle_inhibit(&mut self) {
        if let Some(idle_inhibitor) = &self.idle_inhibitor {
            idle_inhibitor.destroy();
            self.idle_inhibitor = None;
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.idle_inhibitor.is_some()
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for IdleInhibitorDaemon {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name, interface, ..
        } = event
        {
            match &interface[..] {
                "wl_compositor" => {
                    let compositor =
                        registry.bind::<wl_compositor::WlCompositor, _, _>(name, 1, qh, ());
                    let surface = compositor.create_surface(qh, ());
                    state.base_surface = Some(surface);
                }
                "zwp_idle_inhibit_manager_v1" => {
                    state.idle_inhibit_manager = Some(registry.bind(name, 1, qh, ()));
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for IdleInhibitorDaemon {
    fn event(
        _: &mut Self,
        _: &wl_compositor::WlCompositor,
        _: wl_compositor::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        todo!()
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for IdleInhibitorDaemon {
    fn event(
        _: &mut Self,
        _: &wl_surface::WlSurface,
        _: wl_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        todo!()
    }
}

impl Dispatch<zwp_idle_inhibit_manager_v1::ZwpIdleInhibitManagerV1, ()> for IdleInhibitorDaemon {
    fn event(
        _: &mut Self,
        _: &zwp_idle_inhibit_manager_v1::ZwpIdleInhibitManagerV1,
        _: zwp_idle_inhibit_manager_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        todo!()
    }
}

impl Dispatch<zwp_idle_inhibitor_v1::ZwpIdleInhibitorV1, ()> for IdleInhibitorDaemon {
    fn event(
        _: &mut Self,
        _: &zwp_idle_inhibitor_v1::ZwpIdleInhibitorV1,
        _: zwp_idle_inhibitor_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        todo!()
    }
}
