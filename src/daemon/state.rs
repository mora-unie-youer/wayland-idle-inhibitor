use std::{
    io::{Read, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use wayland_client::{
    protocol::{wl_compositor, wl_registry, wl_surface},
    Connection, Dispatch, EventQueue, QueueHandle,
};
use wayland_protocols::wp::idle_inhibit::zv1::client::{
    zwp_idle_inhibit_manager_v1, zwp_idle_inhibitor_v1,
};

use crate::Command;

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
        let mut state = Self {
            terminate: Arc::new(AtomicBool::new(false)),
            queue_handle: event_queue.handle(),

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
        for mut client in socket.incoming().flatten() {
            // Check if we need to terminate daemon
            if self.terminate.load(Ordering::SeqCst) {
                break;
            }

            let mut data = [0xff];
            client.read_exact(&mut data).unwrap();

            match Command::from(data[0]) {
                Command::Disable => self.disable_idle_inhibit(),
                Command::Enable => self.enable_idle_inhibit(),
                Command::Toggle => self.toggle_idle_inhibit(),
                _ => (),
            }

            // Send to client anyway if idle inhibitor is enabled
            client.write_all(&[self.is_enabled() as u8]).unwrap();
            self.roundtrip(event_queue);
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
