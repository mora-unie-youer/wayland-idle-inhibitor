use wayland_client::{
    protocol::{wl_compositor, wl_registry, wl_surface},
    Connection, Dispatch, QueueHandle,
};
use wayland_protocols::wp::idle_inhibit::zv1::client::{
    zwp_idle_inhibit_manager_v1, zwp_idle_inhibitor_v1,
};

#[derive(Debug)]
struct HyprlandIdleInhibitor {
    base_surface: Option<wl_surface::WlSurface>,
    idle_inhibit_manager: Option<zwp_idle_inhibit_manager_v1::ZwpIdleInhibitManagerV1>,
}

impl Dispatch<wl_registry::WlRegistry, ()> for HyprlandIdleInhibitor {
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

impl Dispatch<wl_compositor::WlCompositor, ()> for HyprlandIdleInhibitor {
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

impl Dispatch<wl_surface::WlSurface, ()> for HyprlandIdleInhibitor {
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

impl Dispatch<zwp_idle_inhibit_manager_v1::ZwpIdleInhibitManagerV1, ()> for HyprlandIdleInhibitor {
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

impl Dispatch<zwp_idle_inhibitor_v1::ZwpIdleInhibitorV1, ()> for HyprlandIdleInhibitor {
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

fn main() {
    let conn = Connection::connect_to_env().expect("Couldn't connect to Wayland socket");
    let display = conn.display();

    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();
    let _registry = display.get_registry(&qh, ());

    let mut state = HyprlandIdleInhibitor {
        base_surface: None,
        idle_inhibit_manager: None,
    };

    event_queue.roundtrip(&mut state).unwrap();
}
