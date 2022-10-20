use std::io::Write;
use std::net::TcpStream;
use std::{collections::HashMap, ptr};
use std::sync::mpsc;

use apricity::gui::{Event, Rect, Font, SimpleImage};
use apricity::{Coordinate, Point, gui::SimpleWindow};

use rustdemo::protocol::{ServerMessage, ClientMessage};
use rustdemo::{FeatureCollection, Geometry, load_cities, City};

fn load_font() -> Font<'static> {
    Font::try_from_bytes(ttf_noto_sans::REGULAR).unwrap()
}


pub enum DisplayState {
    WaitForGuess,
    WaitForServer {
        guess: Coordinate,
    },
    WaitForContinue {
        guess: Coordinate,
        actual: Coordinate,
    },
}

pub struct GameState {
    pub current_city: String,
    pub display: DisplayState,
    pub sender: mpsc::Sender<ClientMessage>,
}

impl GameState {
    fn handle_click(
        &mut self,
        click_point: Point,
        window_width: u32,
        window_height: u32,
    ) {
        match self.display {
            DisplayState::WaitForGuess => {
                let coordinate = click_point.coordinate(
                    window_width as f64,
                    window_height as f64,
                );

                self.sender.send(
                    ClientMessage::Guess(coordinate)
                ).unwrap();
                self.display = DisplayState::WaitForServer {
                    guess: coordinate,
                };
            },
            DisplayState::WaitForServer { .. } => {},
            DisplayState::WaitForContinue { .. } => {
                self.display = DisplayState::WaitForGuess;
            },
        }
    }
}

fn connect_to_server() -> (mpsc::Sender<ClientMessage>, mpsc::Receiver<ServerMessage>) {
    let (client_tx, client_rx) = mpsc::channel();
    let (server_tx, server_rx) = mpsc::channel();

    let mut socket = TcpStream::connect(("127.0.0.1", 12345)).unwrap();

    let socket2 = socket.try_clone().unwrap();
    std::thread::spawn(move || {
        loop {
            let response: ServerMessage = bincode::deserialize_from(&socket2).unwrap();
            server_tx.send(response).unwrap()
        }
    });

    std::thread::spawn(move || {
        for msg in client_rx {
            let buffer = bincode::serialize(&msg).unwrap();
            socket.write(&buffer).unwrap();
        }
    });

    (client_tx, server_rx)
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let font = load_font();

    let data = std::fs::read_to_string("countries.geojson")?;
    let countries: FeatureCollection = serde_json::from_str(&data)?;

    let cities: Vec<City> = load_cities()?;

    let mut cities_by_country: HashMap<String, Vec<&City>> = HashMap::new();
    for city in &cities {
        cities_by_country.entry(city.fields.country_code.to_string())
            .or_default()
            .push(city);
    }

    let largest_cities: Vec<&City> = cities_by_country.values()
        .map(|city_list| *city_list.iter().max_by_key(|x| x.fields.population).unwrap())
        .collect();

    let (sender, receiver) = connect_to_server();

    let width = 1500;
    let height = 750;

    let mut image = apricity::gui::SimpleImage::new(width, height);

    for country in &countries.features {
        let multi_polygon = match &country.geometry {
            Geometry::MultiPolygon(p) => p,
            _ => continue,
        };

        for polygon in &multi_polygon.coordinates {
            for ring in polygon {
                let ring: Vec<Point> = ring.iter()
                    .map(|x| x.screen(width as f64, height as f64))
                    .collect();

                image.draw_polygon(&ring, [ 0, 0xFF, 0, 0xFF ]);
            }
        }
    }

    sender.send(ClientMessage::Hello {
        name: "Emil".to_string(),
    }).unwrap();

    match receiver.recv()? {
        ServerMessage::Welcome { server_name } => {
            println!("Connected to {}", server_name);
        },
        _ => panic!(),
    }

    let current_city = match receiver.recv()? {
        ServerMessage::NewRound { city_name } => city_name.clone(),
        _ => panic!(),
    };

    let state = GameState {
        current_city,
        display: DisplayState::WaitForGuess,
        sender,
    };

    let window = SimpleWindow::new(width, height, state)?;

    window.run(|window, events| {
        while let Ok(msg) = receiver.try_recv() {
            match dbg!(msg) {
                ServerMessage::Welcome { server_name } => {},
                ServerMessage::NewRound { city_name } => {
                    let state = window.state_mut();
                    state.current_city = city_name;
                }
                ServerMessage::RoundResults { actual_location: actual } => {
                    let state = window.state_mut();
                    let guess = match state.display {
                        DisplayState::WaitForServer { guess } => guess,
                        _ => panic!(),
                    };

                    state.display = DisplayState::WaitForContinue {
                        guess,
                        actual,
                    };
                },
            }
        }

        window.draw_image(&image, None, false)?;


        // Draw current state
        let state = window.state();
        let text = match state.display {
            DisplayState::WaitForServer { guess } => {
                let p = guess.screen(window.width() as f64, window.height() as f64);
                window.stroke_circle(
                    p.x,
                    p.y,
                    5.0,
                    1.0,
                    [ 0xFF, 0, 0, 0xFF ],
                )?;
            
                "Waiting for other players".to_string()
            },
            DisplayState::WaitForContinue { guess, actual } => {
                let guess_point = guess.screen(window.width() as f64, window.height() as f64);
                let actual_point = actual.screen(window.width() as f64, window.height() as f64);

                window.stroke_circle(
                    guess_point.x,
                    guess_point.y,
                    5.0,
                    1.0,
                    [ 0xFF, 0, 0, 0xFF ],
                )?;

                window.stroke_circle(
                    actual_point.x,
                    actual_point.y,
                    5.0,
                    1.0,
                    [ 0, 0, 0xFF, 0xFF ],
                )?;

                let distance_km = guess.great_circle_distance(actual)/1000.0;

                format!("You were {}km off", distance_km)
            },
            DisplayState::WaitForGuess => {
                format!("Click on {}", state.current_city)
            },
        };

        let window_width = window.width();
        let window_height = window.height();
        let text_image = SimpleImage::create_text_image(
            &font,
            &text,
            128.0,
            [0xFF, 0, 0],
        )?;
        window.draw_image(
            &text_image,
            Some(Rect::new(
                0,
                0,
                text_image.width(),
                text_image.height(),
            )),
            true,
        )?;

        // Handle clicks
        let state = window.state_mut();
        for event in events {
            match event {
                Event::MouseButtonDown { x, y, .. } => {
                    let p = Point { 
                        x: x as f64,
                        y: y as f64,
                    };
                    state.handle_click(
                        p,
                        window_width,
                        window_height,
                    );
                },
                _ => {},
            }
        }

        Ok(())
    })?;

    Ok(())
}
