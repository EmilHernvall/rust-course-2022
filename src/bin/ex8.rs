use std::net::TcpStream;
use std::io::Write;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut socket = TcpStream::connect(("127.0.0.1", 12345))?;
    socket.write(b"Hello!")?;

    Ok(())
}