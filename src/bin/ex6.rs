use std::io::{Write, Read};
use std::sync::{Arc, RwLock};

// nc localhost 12345

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = std::net::TcpListener::bind(("0.0.0.0", 12345))?;

    let mut sockets = Arc::new(RwLock::new(Vec::new()));
    for socket in listener.incoming() {
        let mut socket = match socket {
            Ok(x) => x,
            Err(e) => {
                eprintln!("{:?}", e);
                continue;
            },
        };

        {
            let mut sockets_lock = sockets.write().unwrap();
            sockets_lock.push(socket.try_clone()?);
        }

        let sockets2 = sockets.clone();
        std::thread::spawn(move || {
            socket.write(b"Hello!").unwrap();
            let mut buffer = [0; 1024];
            let len = socket.read(&mut buffer).unwrap();
            let s = std::str::from_utf8(&buffer[0..len]).unwrap();
            println!("{}", s);

            let sockets = sockets2.read().unwrap();
            for mut client in sockets.iter() {
                client.write(s.as_bytes()).unwrap();
            }
        });
    }

    Ok(())
}