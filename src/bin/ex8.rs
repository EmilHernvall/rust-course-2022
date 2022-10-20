use std::net::TcpStream;
use std::io::Write;

use rustdemo::protocol::{ClientMessage, ServerMessage};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut socket = TcpStream::connect(("127.0.0.1", 12345))?;
    
    let msg = ClientMessage::Hello {
        name: "Emil".to_string(),
    };
    let buffer = bincode::serialize(&msg)?;
    socket.write(&buffer)?;

    let reply: ServerMessage = bincode::deserialize_from(&socket)?;

    println!("{:?}", &reply);

    Ok(())
}