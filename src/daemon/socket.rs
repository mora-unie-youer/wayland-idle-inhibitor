use std::{
    ops::{Deref, DerefMut},
    os::unix::net::UnixListener,
};

pub fn get_socket_path() -> String {
    const SOCKET_FILE: &str = "wayland-idle-inhibitor";
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").expect("$XDG_RUNTIME_DIR is not set");
    format!("{runtime_dir}/{SOCKET_FILE}")
}

#[derive(Debug)]
pub struct IdleInhibitorSocket {
    socket: UnixListener,
}

impl IdleInhibitorSocket {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let socket = UnixListener::bind(get_socket_path())?;
        Ok(Self { socket })
    }
}

impl Deref for IdleInhibitorSocket {
    type Target = UnixListener;

    fn deref(&self) -> &Self::Target {
        &self.socket
    }
}

impl DerefMut for IdleInhibitorSocket {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.socket
    }
}

impl Drop for IdleInhibitorSocket {
    fn drop(&mut self) {
        std::fs::remove_file(get_socket_path()).unwrap();
    }
}
