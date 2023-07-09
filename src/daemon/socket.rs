use std::{
    ops::{Deref, DerefMut},
    os::unix::net::UnixListener,
};

pub struct IdleInhibitorSocket {
    path: String,
    socket: UnixListener,
}

impl IdleInhibitorSocket {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").expect("$XDG_RUNTIME_DIR is not set");
        const SOCKET_FILE: &str = "wayland-idle-inhibitor.sock";

        let path = format!("{runtime_dir}/{SOCKET_FILE}");
        let socket = UnixListener::bind(&path)?;
        Ok(Self { path, socket })
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
        let _ = std::fs::remove_file(&self.path).unwrap();
    }
}
