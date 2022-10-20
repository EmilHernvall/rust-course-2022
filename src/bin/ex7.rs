use std::collections::HashMap;
use std::net::TcpStream;
use std::io::{Write, Read};
use std::sync::{Arc, RwLock};

// nc localhost 12345
pub enum SocketEvent {
    Connect(u32, TcpStream),
    Message(u32, String),
    Disconnect(u32),
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = std::sync::mpsc::channel::<SocketEvent>();
    std::thread::spawn(move || {
        let mut sockets = HashMap::new();
        for event in rx {
            match event {
                SocketEvent::Connect(socket_id, socket) => {
                    sockets.insert(socket_id, socket);
                },
                SocketEvent::Message(socket_id, msg) => {
                    for (id, mut socket) in &mut sockets {
                        if *id == socket_id {
                            continue;
                        }
                        socket.write(msg.as_bytes()).unwrap();
                    }
                },
                SocketEvent::Disconnect(socket_id) => {
                    sockets.remove(&socket_id);
                },
            }
        }
    });

    let listener = std::net::TcpListener::bind(("0.0.0.0", 12345))?;
    let mut socket_counter = 0;
    for socket in listener.incoming() {
        let mut socket = match socket {
            Ok(x) => x,
            Err(e) => {
                eprintln!("{:?}", e);
                continue;
            },
        };

        let socket_id = socket_counter;
        socket_counter += 1;

        let socket2 = socket.try_clone().unwrap();
        tx.send(SocketEvent::Connect(socket_id, socket2)).unwrap();

        let tx = tx.clone();
        std::thread::spawn(move || {
            let mut buffer = [0; 1024];
            while let Ok(len) = socket.read(&mut buffer) {
                if len == 0 {
                    break;
                }
                let s = std::str::from_utf8(&buffer[0..len]).unwrap();
                println!("{}", &s);

                tx.send(SocketEvent::Message(socket_id, s.to_string())).unwrap();
            }

            tx.send(SocketEvent::Disconnect(socket_id)).unwrap();
        });
    }

    Ok(())
}