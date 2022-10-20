use std::collections::HashMap;
use std::net::TcpStream;
use std::io::{Write, Read};
use std::sync::{Arc, RwLock};

use rustdemo::protocol::*;

// nc localhost 12345
pub enum SocketEvent {
    Connect(u32, TcpStream),
    Message(u32, ClientMessage),
    Disconnect(u32),
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_name = "Emil's excellent server";

    let (tx, rx) = std::sync::mpsc::channel::<SocketEvent>();
    std::thread::spawn(move || {
        let mut sockets = HashMap::new();
        for event in rx {
            match event {
                SocketEvent::Connect(socket_id, socket) => {
                    sockets.insert(socket_id, socket);
                },
                SocketEvent::Message(socket_id, ClientMessage::Hello { name }) => {
                    let mut client = sockets.get_mut(&socket_id).unwrap();

                    let welcome = ServerMessage::Welcome {
                        server_name: server_name.to_string(),
                    };

                    let welcome = bincode::serialize(&welcome).unwrap();
                    client.write(&welcome).unwrap();
                },
                SocketEvent::Message(socket_id, ClientMessage::Guess(coordinate)) => {
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
            while let Ok(msg) = bincode::deserialize_from(&socket) {
                tx.send(SocketEvent::Message(socket_id, msg)).unwrap();
            }

            tx.send(SocketEvent::Disconnect(socket_id)).unwrap();
        });
    }

    Ok(())
}