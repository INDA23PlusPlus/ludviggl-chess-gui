
use crate::app::{ App, AppInterface, };
use crate::{ Board, State };
use std::net::{ TcpListener, TcpStream, };
use std::thread::JoinHandle;
use std::sync::Mutex;
use simonsev_chess;

struct Server {

    thread: JoinHandle<()>,
    game: Mutex<simonsev_chess::Game>, 
}

impl Server {

    pub fn new(port: String) -> App {

        let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).unwrap();
        let (stream, _) = listener.accept().unwrap();

        let game = Mutex::new(simonsev_chess::Game::new());

        let thread = std::thread::spawn(
            move || Self::thread_fn(stream, &game)
        );

        Box::new(Self { thread, game, })
    }

    fn thread_fn(stream: TcpStream, game: &Mutex<simonsev_chess::Game>) {

    }
}

impl AppInterface for Server {

    fn get_board(&self) -> Board {

        // Here we actually convert the backend representation to the 
        // board defined by protocol
        unimplemented!();
    }

    fn get_state(&self) -> State {

        // Here we calculate the state depending on the backend
        unimplemented!();
    }
}
