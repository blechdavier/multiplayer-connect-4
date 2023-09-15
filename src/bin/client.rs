use connect_4::{send_packet, Board, ClientBoundPacket, Color, Deserialize, ServerBoundPacket};
use core::panic;
use std::error::Error;
use std::io;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.contains(&"auto".to_string()) {
        println!("auto mode");
    }
    // Connect to a peer
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    let mut name = String::new();
    let mut opponent_name = String::new();
    println!("What is your name?");
    if let Err(_) = io::stdin().read_line(&mut name) {
        println!("Failed to read line. Your name is now \"Player\".");
        name = "Player".to_string();
    } else {
        name = name.trim().to_string();
    }

    send_packet(ServerBoundPacket::Init { name: name.clone() }, &mut stream).await?;

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
                opponent_name = opponent;
                client_color = your_color;
                print!("\x1B[2J\x1B[1;1H");
                if &client_color == &Color::Red {
                    println!("Red: {} (you)\nYellow: {}\n{}", &name, opponent_name, board);
                } else {
                    println!("Red: {}\nYellow: {} (you)\n{}", opponent_name, &name, board);
                }
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
                print!("\x1B[2J\x1B[1;1H");
                if &client_color == &Color::Red {
                    println!("Red: {} (you)\nYellow: {}\n{}", &name, opponent_name, board);
                } else {
                    println!("Red: {}\nYellow: {} (you)\n{}", opponent_name, &name, board);
                }

                if color != client_color {
                    play(&mut board, &mut stream).await?;
                }
                print!("\x1B[2J\x1B[1;1H");
            }
            ClientBoundPacket::GameResult { result, col, color } => {
                if let Some(col) = col {
                    board
                        .play_move(
                            col,
                            match color {
                                Color::Red => 1,
                                Color::Yellow => 2,
                            },
                        )
                        .unwrap();
                }
                if &client_color == &Color::Red {
                    println!("Red: {} (you)\nYellow: {}\n{}", &name, opponent_name, board);
                } else {
                    println!("Red: {}\nYellow: {} (you)\n{}", opponent_name, &name, board);
                }
                println!("Game over! Result: {:?}", result);
                break;
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
        } else if buf.trim() == "forfeit" {
            println!("Forfeiting game");
            send_packet(ServerBoundPacket::Forfeit, stream).await?;
            return Ok(());
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
