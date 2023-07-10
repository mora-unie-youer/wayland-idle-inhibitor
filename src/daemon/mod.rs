use std::{os::unix::net::UnixStream, sync::atomic::Ordering};

use lockfile::Lockfile;
use wayland_client::Connection;

use self::{socket::IdleInhibitorSocket, state::IdleInhibitorDaemon};

pub mod socket;
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
    let mut state = IdleInhibitorDaemon::new(&mut event_queue);

    // Setup interrupt and terminate handlers
    // UNSAFE: Only because this part of crate is unsafe, and there's no other way to do this
    // The only error could be, if client is doing request and at the same time daemon stops.
    // But I don't think that some person would do this :P
    unsafe {
        const TERM_SIGNALS: [i32; 2] = [signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM];
        for signal in TERM_SIGNALS {
            let socket_addr = socket_listener.local_addr().unwrap();
            let terminate = state.terminate.clone();
            signal_hook::low_level::register(signal, move || {
                // Store 'true' in 'terminate' boolean
                terminate.store(true, Ordering::SeqCst);
                // Create connection to socket to reach termination check
                UnixStream::connect_addr(&socket_addr).unwrap();
            })
            .expect("Couldn't setup signal handler");
        }
    }

    // Run daemon state
    state.run(&mut event_queue, socket_listener);

    // Release lockfile
    lockfile.release().expect("Couldn't release lockfile");
}
