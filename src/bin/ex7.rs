use std::net::TcpStream;
use std::io::{Write, Read};
use std::sync::{Arc, RwLock};

// nc localhost 12345
pub enum SocketEvent {
    Connect(TcpStream),
    Message(String),
    // Disconnect
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = std::sync::mpsc::channel::<SocketEvent>();
    std::thread::spawn(move || {
        let mut sockets = vec![];
        for event in rx {
            match event {
                SocketEvent::Connect(socket) => {
                    sockets.push(socket);
                },
                SocketEvent::Message(msg) => {
                    for socket in &mut sockets {
                        socket.write(msg.as_bytes()).unwrap();
                    }
                },
            }
        }
    });

    let listener = std::net::TcpListener::bind(("0.0.0.0", 12345))?;
    for socket in listener.incoming() {
        let mut socket = match socket {
            Ok(x) => x,
            Err(e) => {
                eprintln!("{:?}", e);
                continue;
            },
        };

        let socket2 = socket.try_clone().unwrap();
        tx.send(SocketEvent::Connect(socket2)).unwrap();

        let tx = tx.clone();
        std::thread::spawn(move || {
            socket.write(b"Hello!").unwrap();
            let mut buffer = [0; 1024];
            let len = socket.read(&mut buffer).unwrap();
            let s = std::str::from_utf8(&buffer[0..len]).unwrap();
            println!("{}", &s);

            tx.send(SocketEvent::Message(s.to_string()));
        });
    }

    Ok(())
}