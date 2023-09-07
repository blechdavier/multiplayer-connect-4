use connect_4::{send_packet, C2SPacket};
use std::error::Error;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Connect to a peer
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;

    let packet = C2SPacket::Init {
        name: "Blechdavier".to_string(),
    };
    send_packet(packet, &mut stream).await?;
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    let packet = C2SPacket::Init {
        name: "Second Packet".to_string(),
    };
    send_packet(packet, &mut stream).await?;
    Ok(())
}
