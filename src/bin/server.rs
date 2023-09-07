use connect_4::C2SPacket;
use connect_4::Deserialize;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};

use std::{io, rc::Rc};
use tokio::sync::Mutex;

// async fn accept_connection(mut stream: TcpStream, state: Arc<Mutex<i32>>) {
//     loop {
//         let packet = read_c2s_packet(&mut stream).await;
//         println!("read c2s packet: {:?}", packet);
//         let mut state = state.lock().await;
//         *state += 1;
//         println!("total packets read: {}", state);
//     }
// }

async fn play_game(stream1: TcpStream, stream2: TcpStream) {
    let mut board = [[0; 7]; 6];
    let mut turn = 0;
    loop {
        let packet = read_c2s_packet(stream1).await;
        println!("read c2s packet from client 1: {:?}", packet);
        let packet = read_c2s_packet(stream2).await;
        println!("read c2s packet from client 2: {:?}", packet);
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
        if let Some(other_player) = queue {
            tokio::spawn(async move {
                play_game(socket, *other_player).await;
            });
            queue = None;
        } else {
            queue = Some(Rc::new(socket));
        }
    }
}
