
use chess_network_protocol::{
    self                    as protocol,
    ServerToClient          as Stc,
    ClientToServer          as Cts,
    ServerToClientHandshake as StcHand,
    ClientToServerHandshake as CtsHand,
};

use crate::logic;
use crate::tcp_handler::{ self as tcp, TcpHandler, };

use std::net::TcpStream;

pub struct Client {

    player:      logic::Player,
    board:       [[protocol::Piece; 8]; 8],
    state:       logic::State,
    tcp_handler: TcpHandler<Stc, Cts>,
}

impl Client {

    pub fn new(addr: String) -> logic::Layer {

        println!("Connecting to address {}", addr);
        let stream = TcpStream::connect(addr).expect("Could not connect to server");
        println!("Connected!");

        // We always wanna be white
        // hehe
        let ctsh = CtsHand {
            server_color: protocol::Color::Black,
        };
        println!("Sending handshake");
        tcp::write(&stream, ctsh);

        println!("Waiting for server handshake");
        let stch: StcHand = tcp::read(&stream);
        println!("Handshake complete!");
        
        let board = stch.board.clone();

        let tcp_handler = TcpHandler::new(stream);
        let state = logic::State::SelectPiece; // because we're white
        let player = logic::Player::White;

        Box::new(Self {
            player,
            board,
            state,
            tcp_handler,
        })
    }
}

impl logic::Interface for Client {

    fn get_state(&self) -> logic::State {
        self.state
    } 

    fn get_piece_at(&self, x: u8, y: u8) -> Option<(logic::Piece, logic::Player)> {

        proto_to_logic(self.board[x as usize][y as usize])
    }

    fn select_piece(&mut self, at: (u8, u8)) {
        
        if !matches!(self.state, logic::State::SelectPiece) {
            return;
        }

        let piece = self.board[at.0 as usize][at.1 as usize];
        let piece = match proto_to_logic(piece) {
            Some(piece) => piece,
            None => return,
        };

        // Must be our piece
        if piece.1 != self.player {
            return;
        }
        
        self.state = logic::State::SelectMove { from: at, };
    }

    fn play_move(&mut self, dst: (u8, u8)) {

        let from = match self.state {
            logic::State::SelectMove { from } => from,
            _ => return,
        };

        let cts = Cts::Move(protocol::Move {
            start_x: from.0 as usize,
            start_y: from.1 as usize,
            end_x: dst.0 as usize,
            end_y: dst.1 as usize,
            // We don't support promotions
            promotion: protocol::Piece::None,
        });

        self.tcp_handler.write(cts);        
        self.state = logic::State::ResponsePending;
    }

    fn update(&mut self) {

        match self.state {

            logic::State::ResponsePending => {

                if let Some(stc) = self.tcp_handler.read() {

                    match stc {
                        Stc::State {
                            board,
                            joever,
                            ..
                        } => {
                            self.board = board.clone();
                            match joever {
                                protocol::Joever::Ongoing => self.state = logic::State::OpponentTurn,
                                protocol::Joever::White => 
                                    self.state = logic::State::CheckMate(logic::Player::White),
                                protocol::Joever::Black => 
                                    self.state = logic::State::CheckMate(logic::Player::Black),
                                _ => panic!("Joever not implemented: {:?}", joever),
                            };
                        },
                        Stc::Error {
                            board,
                            ..
                        } => {
                            self.board = board.clone();
                            self.state = logic::State::SelectPiece;
                        },
                        _ => panic!("ServerToCLient not implemented: {:?}", stc),
                    }
                }
            },
            logic::State::OpponentTurn => {
                
                if let Some(stc) = self.tcp_handler.read() {

                    match stc {

                        Stc::State {
                            board,
                            joever,
                            ..
                        } => {

                            self.board = board.clone();
                            match joever {
                                protocol::Joever::Ongoing => self.state = logic::State::SelectPiece,
                                protocol::Joever::White => 
                                    self.state = logic::State::CheckMate(logic::Player::White),
                                protocol::Joever::Black => 
                                    self.state = logic::State::CheckMate(logic::Player::Black),
                                _ => panic!("Joever not implemented: {:?}", joever),
                            };
                        },

                        Stc::Error { .. } => (/* not possible */),
                        _ => panic!("ServerToClient not implemented: {:?}", stc),
                    }
                }
            },
            _ => (/* Other cases handled by other methods*/),
        }
    }
}

fn proto_to_logic(piece: protocol::Piece) -> Option<(logic::Piece, logic::Player)> {

    match piece {
        protocol::Piece::None        => None,

        protocol::Piece::BlackPawn   => Some((logic::Piece::Pawn, logic::Player::Black)),
        protocol::Piece::BlackRook   => Some((logic::Piece::Rook, logic::Player::Black)),
        protocol::Piece::BlackKnight => Some((logic::Piece::Knight, logic::Player::Black)),
        protocol::Piece::BlackBishop => Some((logic::Piece::Bishop, logic::Player::Black)),
        protocol::Piece::BlackQueen  => Some((logic::Piece::Queen, logic::Player::Black)),
        protocol::Piece::BlackKing   => Some((logic::Piece::King, logic::Player::Black)),

        protocol::Piece::WhitePawn   => Some((logic::Piece::Pawn, logic::Player::White)),
        protocol::Piece::WhiteRook   => Some((logic::Piece::Rook, logic::Player::White)),
        protocol::Piece::WhiteKnight => Some((logic::Piece::Knight, logic::Player::White)),
        protocol::Piece::WhiteBishop => Some((logic::Piece::Bishop, logic::Player::White)),
        protocol::Piece::WhiteQueen  => Some((logic::Piece::Queen, logic::Player::White)),
        protocol::Piece::WhiteKing   => Some((logic::Piece::King, logic::Player::White)),
    }
}
