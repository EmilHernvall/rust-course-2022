#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum ServerMessage {
    Welcome {
        server_name: String,
    },
    NewRound {
        city_name: String,
    },
    RoundResults {
        actual_location: apricity::Coordinate,
    },
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum ClientMessage {
    Hello {
        name: String,
    },
    Guess(apricity::Coordinate),
}