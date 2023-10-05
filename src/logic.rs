
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum State {
    OpponentTurn,
    ResponsePending,
    SelectPiece,
    SelectMove { from: (u8, u8), },
    SelectPromotion { at: (u8, u8), },
    CheckMate(Player),
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Player {
    White,
    Black,
}

impl Player {
    pub fn other(&self) -> Self {
        match *self {
            Player::White => Player::Black,
            Player::Black => Player::White,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Piece {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

pub trait Interface {
    
    fn get_state(&self) -> State;
    fn update(&mut self);
    fn get_piece_at(&self, x: u8, y: u8) -> Option<(Piece, Player)>;
    fn select_piece(&mut self, at: (u8, u8));
    fn play_move(&mut self, dst: (u8, u8));
}

pub type Layer = Box<dyn Interface>;
