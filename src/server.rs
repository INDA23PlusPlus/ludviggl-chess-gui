
use simonsev_chess as backend;
use chess_network_protocol::{
    self as protocol,
    ClientToServer as Cts,
    ServerToClient as Stc,
    ClientToServerHandshake as CtsHand,
    ServerToClientHandshake as StcHand,
};

use crate::logic;
use crate::tcp_handler::{ self as tcp, TcpHandler, };

use std::net::TcpListener;

type ThreadResult = ();

fn sqstr(x: usize, y: usize) -> String {

    let a = match y {
        0 => "A",
        1 => "B",
        2 => "C",
        3 => "D",
        4 => "E",
        5 => "F",
        6 => "G",
        7 => "H",
        _ => panic!(),
    };

    format!("{}{}", a, x + 1)
}

pub struct Server {

    game: backend::Game,
    tcp_handler: TcpHandler<Cts, Stc>,
    state: logic::State,
    player: logic::Player,
}

impl Server {

    pub fn new(port: String) -> logic::Layer {

        println!("Waiting for opponent to connect...");
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).unwrap();
        let (mut stream, addr) = listener.accept().unwrap();
        println!("Connected! ({})", addr);

        let game = backend::Game::new();

        // Receive handshake
        println!("Waiting for client handshake...");
        let ctshand: CtsHand = tcp::read(&mut stream);
        println!("Client wants you to play as {:?}", ctshand.server_color);
        let (state, player) = match ctshand.server_color {
            protocol::Color::White => (
                logic::State::SelectPiece,
                logic::Player::White,
            ),
            protocol::Color::Black => (
                logic::State::OpponentTurn,
                logic::Player::Black,
            ),
        };
        
        // Send handshake
        let mut stchand = StcHand {

            board: Self::convert_board(&game),
            moves: Vec::new(),
            joever: protocol::Joever::Ongoing,
            features: Vec::new(),
        };

        println!("Sending handshake");
        tcp::write(&mut stream, stchand);
        println!("Handshake complete!");
        
        let tcp_handler = TcpHandler::new(stream);

        Box::new(Self {
            game,
            tcp_handler,
            state,
            player,
        })
    }

    fn convert_board(game: &backend::Game) -> [[protocol::Piece; 8]; 8] {

        let mut target = [[protocol::Piece::None; 8]; 8];

        for (backend_row, target_row) 
            in std::iter::zip(game.get_board(), &mut target) 
        {
            for (backend_square, target_square) 
                in std::iter::zip(backend_row, target_row)
            {
                *target_square = if backend_square.occupied {
                    match (
                        backend_square.piece.piece_type,
                        backend_square.piece.white,
                        ) {

                        (backend::PieceType::Pawn,   true) => protocol::Piece::WhitePawn,
                        (backend::PieceType::Rook,   true) => protocol::Piece::WhiteRook,
                        (backend::PieceType::Knight, true) => protocol::Piece::WhiteKnight,
                        (backend::PieceType::Bishop, true) => protocol::Piece::WhiteBishop,
                        (backend::PieceType::Queen,  true) => protocol::Piece::WhiteQueen,
                        (backend::PieceType::King,   true) => protocol::Piece::WhiteKing,
                        
                        (backend::PieceType::Pawn,   false) => protocol::Piece::BlackPawn,
                        (backend::PieceType::Rook,   false) => protocol::Piece::BlackRook,
                        (backend::PieceType::Knight, false) => protocol::Piece::BlackKnight,
                        (backend::PieceType::Bishop, false) => protocol::Piece::BlackBishop,
                        (backend::PieceType::Queen,  false) => protocol::Piece::BlackQueen,
                        (backend::PieceType::King,   false) => protocol::Piece::BlackKing,

                        _ => panic!(),
                    }
                } else { protocol::Piece::None };
            }
        }

        target
    }
}

impl logic::Interface for Server {
    
    fn get_state(&self) -> logic::State {

        self.state
    }

    fn get_piece_at(&self, x: u8, y: u8) -> Option<(logic::Piece, logic::Player)> {

        let square = &self.game.get_board()[x as usize][y as usize];
        if square.occupied {
            
            // Convert piece
            let piece = &square.piece;
            let piece_type = match piece.piece_type {
                backend::PieceType::Pawn       => logic::Piece::Pawn,
                backend::PieceType::Rook       => logic::Piece::Rook,
                backend::PieceType::Knight     => logic::Piece::Knight,
                backend::PieceType::Bishop     => logic::Piece::Bishop,
                backend::PieceType::Queen      => logic::Piece::Queen,
                backend::PieceType::King       => logic::Piece::King,
                backend::PieceType::Unoccupied => panic!(),
            };

            let player = if piece.white {
                logic::Player::White
            } else {
                logic::Player::Black
            };
            
            Some((piece_type, player))
        } else { None }
    }

    fn select_piece(&mut self, at: (u8, u8)) {
        
        if !matches!(self.state, logic::State::SelectPiece) {
            return;
        }

        let square = &self.game.get_board()[at.0 as usize][at.1 as usize];

        if !square.occupied {
            return;
        }

        if square.piece.white != matches!(self.player, logic::Player::White) {
            return;
        }

        self.state = logic::State::SelectMove { from: at, };
    }
    
    fn update(&mut self) {
        
        match self.state {
            logic::State::OpponentTurn => {
                
                match self.tcp_handler.read() {
                    None => (),
                    Some(cts) => match cts {
                        Cts::Move(mov) => {

                            let piece = self.game.get_board()
                                [mov.start_x][mov.start_y]
                                .piece.piece_type.clone();

                            use simonsev_chess::PieceType::*;
                            let piece_str = match piece {
                                Pawn => "pawn",
                                Rook => "rook",
                                Knight => "knight",
                                Bishop => "bishop",
                                Queen => "queen",
                                King => "king",
                                Unoccupied => "!!NONE!!",
                            };

                            let from = sqstr(mov.start_x, mov.start_y);
                            let to = sqstr(mov.end_x, mov.end_y);
                            println!("Opponent wants to move {} from {} to {}", piece_str, from, to);
                            self.game.input_move(from, to);
                            let valid = self.game.check_move_valid();
                            println!("That move is {}", if valid { "valid" } else { "invalid" });
                            self.game = self.game.clone().do_turn();
                            println!("It's now {}s turn", if self.game.white_turn { "white" } else { "black" });

                            // Move is valid if move_from is non-empty
                            let joever = if self.game.mate {
                                // Client check-mated server
                                match self.player {
                                    logic::Player::White => protocol::Joever::Black,
                                    logic::Player::Black => protocol::Joever::White,
                                }
                            } else {
                                protocol::Joever::Ongoing
                            };
                            
                            if valid && matches!(mov.promotion, protocol::Piece::None) {

                                let stc = Stc::State {
                                    board: Self::convert_board(&self.game),
                                    moves: Vec::new(),
                                    move_made: mov,
                                    joever,
                                };

                                self.tcp_handler.write(stc);
                                self.state = if matches!(joever, protocol::Joever::Ongoing) {
                                    logic::State::SelectPiece
                                } else {
                                    logic::State::CheckMate(self.player.other())
                                };
                            } else {

                                let stc = Stc::Error {
                                    board: Self::convert_board(&self.game),
                                    moves: Vec::new(),
                                    message: "".to_string(),
                                    joever,
                                };
                                self.tcp_handler.write(stc);
                            }
                        },
                        m => panic!("Unimplemented client message: {:?}", m),
                    },
                }
            }
            _ => (),
        }
    }

    fn play_move(&mut self, dst: (u8, u8)) {

        let from = match self.state {
            logic::State::SelectMove { from } => from,
            _ => return,
        };

        let from_str = sqstr(from.0 as usize, from.1 as usize);
        let to_str = sqstr(dst.0 as usize, dst.1 as usize);
        self.game.input_move(from_str, to_str);
        let valid = self.game.check_move_valid();
        self.game = self.game.clone().do_turn();

        if valid {

            let joever = if self.game.mate {
                match self.player {
                    logic::Player::White => protocol::Joever::White,
                    logic::Player::Black => protocol::Joever::Black,
                }
            } else {
                protocol::Joever::Ongoing
            };

            let stc = Stc::State {

                board: Self::convert_board(&self.game),
                moves: Vec::new(),
                joever,
                move_made: protocol::Move {
                    start_x: from.0 as usize,
                    start_y: from.1 as usize,
                    end_x: dst.0 as usize,
                    end_y: dst.1 as usize,
                    // Promotions not supported
                    promotion: protocol::Piece::None,
                }
            };

            self.state = if matches!(joever, protocol::Joever::Ongoing)
            {
                logic::State::OpponentTurn
            } else {
                logic::State::CheckMate(self.player)
            };
            self.tcp_handler.write(stc);
        } else {
            self.state = logic::State::SelectPiece;
        }
    }
}
