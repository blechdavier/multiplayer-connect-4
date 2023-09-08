use connect_4::send_packet;
use connect_4::C2SPacket;
use connect_4::Color;
use connect_4::Deserialize;
use connect_4::S2CPacket;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};

use rand::Rng;
use std::{io, rc::Rc};

// async fn accept_connection(mut stream: TcpStream, state: Arc<Mutex<i32>>) {
//     loop {
//         let packet = read_c2s_packet(&mut stream).await;
//         println!("read c2s packet: {:?}", packet);
//         let mut state = state.lock().await;
//         *state += 1;
//         println!("total packets read: {}", state);
//     }
// }

async fn play_game(mut stream1: TcpStream, mut stream2: TcpStream) {
    let mut board = [[0; 7]; 6];
    let mut turn = Color::Red;
    loop {
        let name1 = match read_c2s_packet(&mut stream1).await {
            C2SPacket::Init { name } => name,
            _ => panic!("Expected init packet"),
        };
        let name2 = match read_c2s_packet(&mut stream2).await {
            C2SPacket::Init { name } => name,
            _ => panic!("Expected init packet"),
        };
        // choose player to go first randomly
        let mut rng = rand::thread_rng();
        let color1 = if rng.gen() { Color::Red } else { Color::Yellow };
        let color2 = if color1 == Color::Red {
            Color::Yellow
        } else {
            Color::Red
        };
        // send startgame packet to each client
        send_packet(
            S2CPacket::GameStart {
                opponent: name2.clone(),
                your_color: color1,
            },
            &mut stream1,
        )
        .await;
        send_packet(
            S2CPacket::GameStart {
                opponent: name1.clone(),
                your_color: color2,
            },
            &mut stream2,
        )
        .await;
    }
}

async fn read_c2s_packet(stream: &mut TcpStream) -> C2SPacket {
    let len = match stream.read_u8().await {
        Ok(len) => len,
        Err(_) => {
            panic!("Failed to read length of packet. This could mean the client disconnected.");
        }
    };
    let mut buf = vec![0; len as usize];
    stream.read_exact(&mut buf).await.unwrap();
    C2SPacket::deserialize(&buf)
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    let mut queue: Option<Rc<TcpStream>> = None;

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
