use connect_4::send_packet;
use connect_4::Board;
use connect_4::ClientBoundPacket;
use connect_4::Color;
use connect_4::Deserialize;
use connect_4::GameResult;
use connect_4::ServerBoundPacket;
use rand::Rng;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};

use std::io;

// async fn accept_connection(mut stream: TcpStream, state: Arc<Mutex<i32>>) {
//     loop {
//         let packet = read_serverbound_packet(&mut stream).await;
//         println!("read serverbound packet: {:?}", packet);
//         let mut state = state.lock().await;
//         *state += 1;
//         println!("total packets read: {}", state);
//     }
// }

async fn play_game(stream1: TcpStream, stream2: TcpStream) {
    // choose player to go first randomly
    let rng = rand::thread_rng().gen_bool(0.5);
    let (mut red_player, mut yellow_player) = if rng {
        (stream1, stream2)
    } else {
        (stream2, stream1)
    };

    let mut board = Board::new();
    let mut turn = Color::Red;
    let name1 = match read_serverbound_packet(&mut red_player).await {
        ServerBoundPacket::Init { name } => name,
        _ => panic!("Expected init packet"),
    };
    let name2 = match read_serverbound_packet(&mut yellow_player).await {
        ServerBoundPacket::Init { name } => name,
        _ => panic!("Expected init packet"),
    };

    // send startgame packet to each client
    send_packet(
        ClientBoundPacket::GameStart {
            opponent: name2.clone(),
            your_color: Color::Red,
        },
        &mut red_player,
    )
    .await
    .unwrap();
    send_packet(
        ClientBoundPacket::GameStart {
            opponent: name1.clone(),
            your_color: Color::Yellow,
        },
        &mut yellow_player,
    )
    .await
    .unwrap();
    loop {
        // wait for turn
        match turn {
            Color::Red => {
                // read move packet
                let packet = read_serverbound_packet(&mut red_player).await;
                println!("read serverbound packet from red: {:?}", packet);
                match packet {
                    ServerBoundPacket::Move { col } => {
                        board.play_move(col, 1).unwrap();
                        match board.score() {
                            GameResult::InProgress => {
                                send_packet(
                                    ClientBoundPacket::Move {
                                        col,
                                        color: Color::Red,
                                    },
                                    &mut red_player,
                                )
                                .await
                                .unwrap();
                                send_packet(
                                    ClientBoundPacket::Move {
                                        col,
                                        color: Color::Red,
                                    },
                                    &mut yellow_player,
                                )
                                .await
                                .unwrap();
                                turn = Color::Yellow
                            }
                            result => {
                                send_packet(
                                    ClientBoundPacket::GameResult {
                                        result: result.clone(),
                                        col: Some(col),
                                        color: Color::Red,
                                    },
                                    &mut red_player,
                                )
                                .await
                                .unwrap();
                                send_packet(
                                    ClientBoundPacket::GameResult {
                                        result,
                                        col: Some(col),
                                        color: Color::Red,
                                    },
                                    &mut yellow_player,
                                )
                                .await
                                .unwrap();
                                break;
                            }
                        }
                    }
                    _ => {
                        panic!("Expected move packet")
                    }
                }
            }
            Color::Yellow => {
                // read move packet
                let packet = read_serverbound_packet(&mut yellow_player).await;
                println!("read serverbound packet from yellow: {:?}", packet);
                match packet {
                    ServerBoundPacket::Move { col } => {
                        board.play_move(col, 2).unwrap();
                        match board.score() {
                            GameResult::InProgress => {
                                send_packet(
                                    ClientBoundPacket::Move {
                                        col,
                                        color: Color::Yellow,
                                    },
                                    &mut red_player,
                                )
                                .await
                                .unwrap();
                                send_packet(
                                    ClientBoundPacket::Move {
                                        col,
                                        color: Color::Yellow,
                                    },
                                    &mut yellow_player,
                                )
                                .await
                                .unwrap();
                                turn = Color::Red
                            }
                            result => {
                                send_packet(
                                    ClientBoundPacket::GameResult {
                                        result: result.clone(),
                                        col: Some(col),
                                        color: Color::Yellow,
                                    },
                                    &mut red_player,
                                )
                                .await
                                .unwrap();
                                send_packet(
                                    ClientBoundPacket::GameResult {
                                        result,
                                        col: Some(col),
                                        color: Color::Yellow,
                                    },
                                    &mut yellow_player,
                                )
                                .await
                                .unwrap();
                                break;
                            }
                        }
                    }
                    _ => {
                        panic!("Expected move packet")
                    }
                }
            }
        }
    }
}

async fn read_serverbound_packet(stream: &mut TcpStream) -> ServerBoundPacket {
    let len = match stream.read_u8().await {
        Ok(len) => len,
        Err(_) => {
            panic!("Failed to read length of packet. This could mean the client disconnected.");
        }
    };
    let mut buf = vec![0; len as usize];
    stream.read_exact(&mut buf).await.unwrap();
    ServerBoundPacket::deserialize(&buf)
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (socket, _) = listener.accept().await?;
        println!("accepted connection, waiting for second player");
        let (socket2, _) = listener.accept().await?;
        println!("accepted second connection, starting game");
        tokio::spawn(async move {
            play_game(socket, socket2).await;
        });
    }
}
