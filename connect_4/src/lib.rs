use std::error::Error;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn send_packet(packet: C2SPacket, stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
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
pub enum C2SPacket {
    Init { name: String },
    Move { col: u8 },
    ProposeDraw,
    AcceptDraw,
    Forfeit,
}

#[derive(PartialEq, Debug)]
pub enum S2CPacket {
    AddedToQueue,
    GameStart { opponent: String, your_color: Color },
    Move { col: u8 },
    DrawWasProposed,
    GameResult { result: GameResult },
}
#[derive(PartialEq, Debug)]

pub enum Color {
    Red,
    Yellow,
}
#[derive(PartialEq, Debug)]

pub enum GameResult {
    RedWin,
    YellowWin,
    Draw,
}

impl Serialize for C2SPacket {
    fn serialize(&self) -> Vec<u8> {
        match self {
            C2SPacket::Init { name } => {
                let mut buf = Vec::new();
                buf.push(0);
                if name.len() > 32 {
                    buf.extend(&name.as_bytes()[..32]);
                } else {
                    buf.extend(name.as_bytes());
                }
                buf
            }
            C2SPacket::Move { col } => {
                let mut buf = Vec::new();
                buf.push(1);
                buf.push(*col);
                buf
            }
            C2SPacket::ProposeDraw => {
                let mut buf = Vec::new();
                buf.push(2);
                buf
            }
            C2SPacket::AcceptDraw => {
                let mut buf = Vec::new();
                buf.push(3);
                buf
            }
            C2SPacket::Forfeit => {
                let mut buf = Vec::new();
                buf.push(4);
                buf
            }
        }
    }
}

impl Deserialize for C2SPacket {
    fn deserialize(buf: &[u8]) -> Self {
        match buf[0] {
            0 => C2SPacket::Init {
                name: String::from_utf8_lossy(&buf[1..]).to_string(),
            },
            1 => C2SPacket::Move { col: buf[1] },
            2 => C2SPacket::ProposeDraw,
            3 => C2SPacket::AcceptDraw,
            4 => C2SPacket::Forfeit,
            _ => panic!("Invalid packet type"),
        }
    }
}

impl Serialize for S2CPacket {
    fn serialize(&self) -> Vec<u8> {
        match self {
            S2CPacket::AddedToQueue => {
                let mut buf = Vec::new();
                buf.push(0);
                buf
            }
            S2CPacket::GameStart {
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
            S2CPacket::Move { col } => {
                let mut buf = Vec::new();
                buf.push(2);
                buf.push(*col);
                buf
            }
            S2CPacket::DrawWasProposed => {
                let mut buf = Vec::new();
                buf.push(3);
                buf
            }
            S2CPacket::GameResult { result } => {
                let mut buf = Vec::new();
                buf.push(4);
                buf.push(match result {
                    GameResult::RedWin => 0,
                    GameResult::YellowWin => 1,
                    GameResult::Draw => 2,
                });
                buf
            }
        }
    }
}

impl Deserialize for S2CPacket {
    fn deserialize(buf: &[u8]) -> Self {
        match buf[0] {
            0 => S2CPacket::AddedToQueue,
            1 => S2CPacket::GameStart {
                opponent: String::from_utf8_lossy(&buf[1..buf.len() - 1]).to_string(),
                your_color: match buf[buf.len() - 1] {
                    0 => Color::Red,
                    1 => Color::Yellow,
                    _ => panic!("Invalid color"),
                },
            },
            2 => S2CPacket::Move { col: buf[1] },
            3 => S2CPacket::DrawWasProposed,
            4 => S2CPacket::GameResult {
                result: match buf[1] {
                    0 => GameResult::RedWin,
                    1 => GameResult::YellowWin,
                    2 => GameResult::Draw,
                    _ => panic!("Invalid game result"),
                },
            },
            _ => panic!("Invalid packet type"),
        }
    }
}

#[test]
fn test_serialize_deserialize() {
    let c2s_packets = vec![
        C2SPacket::Init {
            name: "Blechdavier".to_string(),
        },
        C2SPacket::Move { col: 3 },
        C2SPacket::ProposeDraw,
        C2SPacket::AcceptDraw,
        C2SPacket::Forfeit,
    ];
    let s2c_packets = vec![
        S2CPacket::AddedToQueue,
        S2CPacket::GameStart {
            opponent: "Blechdavier".to_string(),
            your_color: Color::Red,
        },
        S2CPacket::Move { col: 3 },
        S2CPacket::DrawWasProposed,
        S2CPacket::GameResult {
            result: GameResult::RedWin,
        },
    ];
    for packet in c2s_packets {
        assert_eq!(packet, C2SPacket::deserialize(&packet.serialize()));
    }
    for packet in s2c_packets {
        assert_eq!(packet, S2CPacket::deserialize(&packet.serialize()));
    }
}
