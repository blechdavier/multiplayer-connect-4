use std::error::Error;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn send_packet<T: Serialize>(
    packet: T,
    stream: &mut TcpStream,
) -> Result<(), Box<dyn Error>> {
    // write to temporary buf
    let buf = packet.serialize();
    // write length of buf
    stream.write_all(&[buf.len() as u8]).await?;
    // write buf
    stream.write_all(&buf).await?;
    Ok(())
}

pub trait Serialize {
    fn serialize(&self) -> Vec<u8>;
}

pub trait Deserialize {
    fn deserialize(buf: &[u8]) -> Self;
}

#[derive(PartialEq, Debug)]
pub enum ServerBoundPacket {
    Init { name: String },
    Move { col: u8 },
    ProposeDraw,
    AcceptDraw,
    Forfeit,
}

#[derive(PartialEq, Debug)]
pub enum ClientBoundPacket {
    AddedToQueue,
    GameStart { opponent: String, your_color: Color },
    Move { col: u8, color: Color },
    DrawWasProposed,
    GameResult { result: GameResult },
}
#[derive(PartialEq, Debug)]

pub enum Color {
    Red,
    Yellow,
}
#[derive(PartialEq, Debug, Clone)]

pub enum GameResult {
    InProgress,
    RedWin,
    YellowWin,
    Draw,
}
impl Serialize for ServerBoundPacket {
    fn serialize(&self) -> Vec<u8> {
        match self {
            ServerBoundPacket::Init { name } => {
                let mut buf = Vec::new();
                buf.push(0);
                if name.len() > 32 {
                    buf.extend(&name.as_bytes()[..32]);
                } else {
                    buf.extend(name.as_bytes());
                }
                buf
            }
            ServerBoundPacket::Move { col } => {
                let mut buf = Vec::new();
                buf.push(1);
                buf.push(*col);
                buf
            }
            ServerBoundPacket::ProposeDraw => {
                let mut buf = Vec::new();
                buf.push(2);
                buf
            }
            ServerBoundPacket::AcceptDraw => {
                let mut buf = Vec::new();
                buf.push(3);
                buf
            }
            ServerBoundPacket::Forfeit => {
                let mut buf = Vec::new();
                buf.push(4);
                buf
            }
        }
    }
}

impl Deserialize for ServerBoundPacket {
    fn deserialize(buf: &[u8]) -> Self {
        match buf[0] {
            0 => ServerBoundPacket::Init {
                name: String::from_utf8_lossy(&buf[1..]).to_string(),
            },
            1 => ServerBoundPacket::Move { col: buf[1] },
            2 => ServerBoundPacket::ProposeDraw,
            3 => ServerBoundPacket::AcceptDraw,
            4 => ServerBoundPacket::Forfeit,
            _ => panic!("Invalid packet type"),
        }
    }
}

impl Serialize for ClientBoundPacket {
    fn serialize(&self) -> Vec<u8> {
        match self {
            ClientBoundPacket::AddedToQueue => {
                let mut buf = Vec::new();
                buf.push(0);
                buf
            }
            ClientBoundPacket::GameStart {
                opponent,
                your_color,
            } => {
                let mut buf = Vec::new();
                buf.push(1);
                buf.extend(opponent.as_bytes());

                buf.push(match your_color {
                    Color::Red => 0,
                    Color::Yellow => 1,
                });
                buf
            }
            ClientBoundPacket::Move { col, color } => {
                let mut buf = Vec::new();
                buf.push(2);
                buf.push(*col);
                buf.push(match color {
                    Color::Red => 0,
                    Color::Yellow => 1,
                });
                buf
            }
            ClientBoundPacket::DrawWasProposed => {
                let mut buf = Vec::new();
                buf.push(3);
                buf
            }
            ClientBoundPacket::GameResult { result } => {
                let mut buf = Vec::new();
                buf.push(4);
                buf.push(match result {
                    GameResult::InProgress => 0,
                    GameResult::RedWin => 1,
                    GameResult::YellowWin => 2,
                    GameResult::Draw => 3,
                });
                buf
            }
        }
    }
}

impl Deserialize for ClientBoundPacket {
    fn deserialize(buf: &[u8]) -> Self {
        match buf[0] {
            0 => ClientBoundPacket::AddedToQueue,
            1 => ClientBoundPacket::GameStart {
                opponent: String::from_utf8_lossy(&buf[1..buf.len() - 1]).to_string(),
                your_color: match buf[buf.len() - 1] {
                    0 => Color::Red,
                    1 => Color::Yellow,
                    _ => panic!("Invalid color"),
                },
            },
            2 => ClientBoundPacket::Move {
                col: buf[1],
                color: match buf[2] {
                    0 => Color::Red,
                    1 => Color::Yellow,
                    _ => panic!("Invalid color"),
                },
            },
            3 => ClientBoundPacket::DrawWasProposed,
            4 => ClientBoundPacket::GameResult {
                result: match buf[1] {
                    0 => GameResult::InProgress,
                    1 => GameResult::RedWin,
                    2 => GameResult::YellowWin,
                    3 => GameResult::Draw,
                    _ => panic!("Invalid game result"),
                },
            },
            _ => panic!("Invalid packet type"),
        }
    }
}

#[test]
fn test_serialize_deserialize() {
    let serverbound_packets = vec![
        ServerBoundPacket::Init {
            name: "Blechdavier".to_string(),
        },
        ServerBoundPacket::Move { col: 3 },
        ServerBoundPacket::ProposeDraw,
        ServerBoundPacket::AcceptDraw,
        ServerBoundPacket::Forfeit,
    ];
    let clientbound_packets = vec![
        ClientBoundPacket::AddedToQueue,
        ClientBoundPacket::GameStart {
            opponent: "Blechdavier".to_string(),
            your_color: Color::Red,
        },
        ClientBoundPacket::Move {
            col: 3,
            color: Color::Red,
        },
        ClientBoundPacket::DrawWasProposed,
        ClientBoundPacket::GameResult {
            result: GameResult::RedWin,
        },
    ];
    for packet in serverbound_packets {
        assert_eq!(packet, ServerBoundPacket::deserialize(&packet.serialize()));
    }
    for packet in clientbound_packets {
        assert_eq!(packet, ClientBoundPacket::deserialize(&packet.serialize()));
    }
}

pub struct Board([[i32; 7]; 6]);

impl Board {
    pub fn new() -> Self {
        Board([[0; 7]; 6])
    }
    pub fn score(&self) -> GameResult {
        let board = self.0;
        // check for horizontal wins
        for row in 0..6 {
            for col in 0..4 {
                if board[row][col] != 0
                    && board[row][col] == board[row][col + 1]
                    && board[row][col] == board[row][col + 2]
                    && board[row][col] == board[row][col + 3]
                {
                    return match board[row][col] {
                        1 => GameResult::RedWin,
                        2 => GameResult::YellowWin,
                        _ => panic!("Invalid board state"),
                    };
                }
            }
        }
        // check for vertical wins
        for row in 0..3 {
            for col in 0..7 {
                if board[row][col] != 0
                    && board[row][col] == board[row + 1][col]
                    && board[row][col] == board[row + 2][col]
                    && board[row][col] == board[row + 3][col]
                {
                    return match board[row][col] {
                        1 => GameResult::RedWin,
                        2 => GameResult::YellowWin,
                        _ => panic!("Invalid board state"),
                    };
                }
            }
        }
        // check for diagonal wins
        for row in 0..3 {
            for col in 0..4 {
                if board[row][col] != 0
                    && board[row][col] == board[row + 1][col + 1]
                    && board[row][col] == board[row + 2][col + 2]
                    && board[row][col] == board[row + 3][col + 3]
                {
                    return match board[row][col] {
                        1 => GameResult::RedWin,
                        2 => GameResult::YellowWin,
                        _ => panic!("Invalid board state"),
                    };
                }
            }
        }
        // check for diagonal wins
        for row in 0..3 {
            for col in 3..7 {
                if board[row][col] != 0
                    && board[row][col] == board[row + 1][col - 1]
                    && board[row][col] == board[row + 2][col - 2]
                    && board[row][col] == board[row + 3][col - 3]
                {
                    return match board[row][col] {
                        1 => GameResult::RedWin,
                        2 => GameResult::YellowWin,
                        _ => panic!("Invalid board state"),
                    };
                }
            }
        }
        // check for draw
        for row in 0..6 {
            for col in 0..7 {
                if board[row][col] == 0 {
                    return GameResult::InProgress;
                }
            }
        }
        // if we get here, the board is full and there are no wins
        GameResult::Draw
    }

    pub fn play_move(&mut self, col: u8, piece: i32) -> Result<(), ()> {
        let mut board = self.0;
        if col > 6 {
            return Err(());
        }
        for row in (0..6).rev() {
            if board[row as usize][col as usize] == 0 {
                board[row as usize][col as usize] = piece;
                return Ok(());
            }
        }
        Err(())
    }

    pub fn legal_move(&mut self, col: u8) -> Result<(), ()> {
        let board = self.0;
        if col > 6 {
            return Err(());
        }
        for row in (0..6).rev() {
            if board[row as usize][col as usize] == 0 {
                return Ok(());
            }
        }
        Err(())
    }
}
