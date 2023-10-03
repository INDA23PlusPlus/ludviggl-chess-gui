
use simonsev_chess::{ self, Game, };
use chess_network_protocol::{
    ServerToClient as Stc,
    ClientToServer as Cts,
    ServerToClientHandshake as StcHandshake,
    ClientToServerHandshake as CtsHandshake,
    self as protocol,
};

use crate::logic;
use crate::tcp_thread::{ 
    TcpThread,
    write as tcp_write,
    read as tcp_read,
};

use std::net::{ TcpListener, TcpStream, };

pub struct Server {

    game: Game,
    tcp_thread: TcpThread<Cts, Stc>,
}

impl Server {

    pub fn new(port: String) -> logic::Layer {

        let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).unwrap();
        let (stream, _) = listener.accept().unwrap();

        // TODO: Handshake

        Self {

            game: Game::new(),
            tcp_thread: TcpThread::new(stream),
        }
    }

    fn board_conv() -> protocol::
}
