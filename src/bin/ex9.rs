use std::collections::HashMap;
use std::net::TcpStream;
use std::io::{Write, Read};
use std::sync::{Arc, RwLock};

use rand::prelude::*;

use rustdemo::{protocol::*, load_cities};

// pub trait CanBeDoubled {
//     fn double(self) -> Self;
// }
// 
// impl CanBeDoubled for u32 {
//     fn double(self) -> Self {
//         2*self
//     }
// }

pub struct Client {
    socket: TcpStream,
    guess: Option<apricity::Coordinate>,
}

impl Client {
    pub fn new(socket: TcpStream) -> Client {
        Client {
            socket,
            guess: None,
        }
    }
}

impl Write for Client {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.socket.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.socket.flush()
    }
}

// nc localhost 12345
pub enum SocketEvent {
    Connect(u32, TcpStream),
    Message(u32, ClientMessage),
    Disconnect(u32),
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_name = "Emil's excellent server";

    let cities = load_cities()?;

    let (tx, rx) = std::sync::mpsc::channel::<SocketEvent>();
    std::thread::spawn(move || {
        let mut clients = HashMap::new();
        let mut rng = thread_rng();
        let mut current_city = cities.choose(&mut rng).unwrap();
        for event in rx {
            match event {
                SocketEvent::Connect(socket_id, socket) => {
                    clients.insert(socket_id, Client::new(socket));
                },
                SocketEvent::Message(socket_id, ClientMessage::Hello { name }) => {
                    let mut client = clients.get_mut(&socket_id).unwrap();

                    let welcome = ServerMessage::Welcome {
                        server_name: server_name.to_string(),
                    };

                    let welcome = bincode::serialize(&welcome).unwrap();
                    client.write(&welcome).unwrap();

                    // TODO: Send ServerMessage::NewRound with current_city
                },
                SocketEvent::Message(socket_id, ClientMessage::Guess(coordinate)) => {
                    // TODO: Update Client with incoming guess
                },
                SocketEvent::Disconnect(socket_id) => {
                    clients.remove(&socket_id);
                },
            }

            // TODO: If current_city is None, pick a new city and start a new round

            // TODO: Check if everyone has submitted their guess, figure out who won,
            // and print results to console

            // TODO: When the round finishes, send the correct answer to client in
            // ServerMessage::RoundResults
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

// Update the client to send ClientMessage::Hello
// and then read the ServerMessage response and print it