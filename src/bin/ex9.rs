use std::collections::HashMap;
use std::net::TcpStream;
use std::io::{Write, Read};
use std::sync::{Arc, RwLock};

use rand::prelude::*;

use rustdemo::{protocol::*, load_cities, Geometry};

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
    name: Option<String>,
    guess: Option<apricity::Coordinate>,
}

impl Client {
    pub fn new(socket: TcpStream) -> Client {
        Client {
            socket,
            name: None,
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
        let mut current_city = cities.choose(&mut rng);
        for event in rx {
            match event {
                SocketEvent::Connect(socket_id, socket) => {
                    clients.insert(socket_id, Client::new(socket));
                },
                SocketEvent::Message(socket_id, ClientMessage::Hello { name }) => {
                    let mut client = clients.get_mut(&socket_id).unwrap();
                    client.name = Some(name);

                    let welcome = ServerMessage::Welcome {
                        server_name: server_name.to_string(),
                    };

                    let welcome = bincode::serialize(&welcome).unwrap();
                    client.write(&welcome).unwrap();

                    // Send ServerMessage::NewRound with current_city
                    if let Some(city) = &current_city {
                        client.write(&bincode::serialize(&ServerMessage::NewRound {
                            city_name: city.fields.name.clone(),
                        }).unwrap()).unwrap();
                    }
                },
                SocketEvent::Message(socket_id, ClientMessage::Guess(coordinate)) => {
                    let client = clients.get_mut(&socket_id).unwrap();
                    client.guess = Some(coordinate);
                },
                SocketEvent::Disconnect(socket_id) => {
                    clients.remove(&socket_id);
                },
            }

            // Check if everyone has submitted their guess, figure out who won,
            // and print results to console

            let city_coords = match &current_city.unwrap().geometry {
                Geometry::Point(p) => p.coordinates,
                _ => unreachable!(),
            };

            let mut scores: Vec<_> = clients.values()
                .filter_map(|client| {
                    let name = client.name.clone()?;
                    let guess = client.guess?;
                    let distance = city_coords.great_circle_distance(guess);

                    Some((name, distance))
                })
                .collect();

            // When the round finishes, send the correct answer to client in
            // ServerMessage::RoundResults
            if scores.len() == clients.len() && clients.len() > 0 {
                scores.sort_by_key(|x| x.1 as i64);

                for (name, score) in scores {
                    println!("{} - {}", name, score);
                }

                for client in clients.values_mut() {
                    client.guess = None;
                    client.write(&bincode::serialize(&ServerMessage::RoundResults {
                        actual_location: city_coords,
                    }).unwrap()).unwrap();
                }

                // Pick a new city
                let new_city = cities.choose(&mut rng).unwrap();
                for client in clients.values_mut() {
                    client.write(&bincode::serialize(&ServerMessage::NewRound {
                        city_name: new_city.fields.name.clone(),
                    }).unwrap()).unwrap();
                }

                println!("New city: {}", &new_city.fields.name);

                current_city = Some(new_city);
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

// Update the client to send ClientMessage::Hello
// and then read the ServerMessage response and print it