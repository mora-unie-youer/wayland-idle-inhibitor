use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
};

use crate::{daemon::socket::get_socket_path, Command};

pub fn start_client(command: Command) {
    // Start connection to socket
    let socket_path = get_socket_path();
    let mut stream = match UnixStream::connect(socket_path) {
        Ok(stream) => stream,
        Err(_) => {
            eprintln!("Couldn't connect to daemon");
            std::process::exit(1);
        }
    };

    // Send command to daemon
    stream.write_all(&[command as u8]).unwrap();

    // Receive status from daemon
    let mut data = [0xff];
    stream.read_exact(&mut data).unwrap();
    match data[0] {
        0 => println!("Disabled"),
        1 => println!("Enabled"),
        _ => unreachable!(),
    }

    // Exit client with status of disabled/enabled for some purposes
    std::process::exit(data[0] as i32);
}
