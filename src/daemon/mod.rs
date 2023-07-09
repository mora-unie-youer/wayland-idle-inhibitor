use std::{sync::atomic::Ordering, time::Duration};

use lockfile::Lockfile;
use wayland_client::Connection;

use self::{socket::IdleInhibitorSocket, state::IdleInhibitorDaemon};

mod socket;
mod state;

fn create_lockfile() -> Result<Lockfile, lockfile::Error> {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").expect("$XDG_RUNTIME_DIR is not set");
    const LOCK_FILE: &str = "wayland-idle-inhibitor.lock";

    let path = format!("{runtime_dir}/{LOCK_FILE}");
    Lockfile::create(path)
}

pub fn start_daemon() {
    // Create lockfile
    let lockfile = match create_lockfile() {
        Ok(lockfile) => lockfile,
        Err(_) => {
            eprintln!("Couldn't create lockfile. Maybe there's another instance already?");
            std::process::exit(1);
        }
    };

    // Create socket
    let socket_listener = match IdleInhibitorSocket::new() {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!("Couldn't create UNIX socket: {err}");
            std::process::exit(1);
        }
    };

    // Create wayland client connection
    let conn = Connection::connect_to_env().expect("Couldn't connect to Wayland socket");
    let display = conn.display();

    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();
    let _registry = display.get_registry(&qh, ());

    // Create daemon state
    let mut state = IdleInhibitorDaemon::new(qh);

    // Handling signals
    signal_hook::flag::register(signal_hook::consts::SIGINT, state.terminate.clone())
        .expect("Couldn't setup SIGINT hook");
    signal_hook::flag::register(signal_hook::consts::SIGTERM, state.terminate.clone())
        .expect("Couldn't setup SIGTERM hook");

    // Initializing Wayland client
    event_queue.roundtrip(&mut state).unwrap();
    state.enable_idle_inhibit();

    while !state.terminate.load(Ordering::Relaxed) {
        // TODO: process socket
        std::thread::sleep(Duration::from_secs(1));

        event_queue.roundtrip(&mut state).unwrap();
    }

    // Release lockfile
    lockfile.release().expect("Couldn't release lockfile");
}
