use connect_4::{send_packet, C2SPacket};
use std::error::Error;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Connect to a peer
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;

    send_packet(
        C2SPacket::Init {
            name: "Blechdavier".to_string(),
        },
        &mut stream,
    )
    .await?;
    loop {
        // wait or smth (play the game)
    }
    Ok(())
}
