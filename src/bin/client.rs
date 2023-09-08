use connect_4::{send_packet, Board, ClientBoundPacket, Color, Deserialize, ServerBoundPacket};
use std::error::Error;
use std::io;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Connect to a peer
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;

    send_packet(
        ServerBoundPacket::Init {
            name: "Blechdavier".to_string(),
        },
        &mut stream,
    )
    .await?;

    let mut client_color = Color::Red;
    let mut board = Board::new();
    loop {
        // wait for packets and print thenm
        let packet = read_clientbound_packet(&mut stream).await;
        println!("read clientbound packet: {:?}", packet);
        match packet {
            ClientBoundPacket::GameStart {
                opponent,
                your_color,
            } => {
                client_color = your_color;
                println!("Game started against {} as {:?}", opponent, &client_color);
                if &client_color == &Color::Red {
                    play(&mut board, &mut stream).await?;
                }
            }
            ClientBoundPacket::Move { col, color } => {
                board
                    .play_move(
                        col,
                        match color {
                            Color::Red => 1,
                            Color::Yellow => 2,
                        },
                    )
                    .unwrap();
                if color != client_color {
                    play(&mut board, &mut stream).await?;
                }
            }
            _ => {
                todo!();
            }
        }
    }
    Ok(())
}

async fn play(board: &mut Board, stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
    println!("It's your turn! What column do you want to play in? (0-6)");
    // get user input

    let col = loop {
        let mut buf = String::new();
        if let Err(_) = io::stdin().read_line(&mut buf) {
            println!("Failed to read line. Try again.");
            continue;
        }
        if let Ok(col) = buf.trim().parse::<u8>() {
            if col > 7 {
                println!("Column too high. Try again.");
                continue;
            }
            if board.legal_move(col).is_err() {
                println!("Column is full. Try again.");
                continue;
            }
            println!("Playing in column {}", col);
            break col;
        }
    };
    send_packet(ServerBoundPacket::Move { col }, stream).await?;
    Ok(())
}

async fn read_clientbound_packet(stream: &mut TcpStream) -> ClientBoundPacket {
    let len = match stream.read_u8().await {
        Ok(len) => len,
        Err(_) => {
            panic!("Failed to read length of packet. This could mean the client disconnected.");
        }
    };
    let mut buf = vec![0; len as usize];
    stream.read_exact(&mut buf).await.unwrap();
    ClientBoundPacket::deserialize(&buf)
}
