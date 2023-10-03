
pub enum Player {
    White,
    Black,
}

pub enum Piece {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

enum MoveResult {
    Ok,
    Illegal,
}

pub trait Interface {
    
    fn get_turn(&self) -> Player;
    fn get_my_color(&self) -> Player;
    fn get_piece_at(&self, x: u8, y: u8) -> Option<(Piece, Player)>;
    fn get_hightlights(&self) -> &[(u8, u8)];
    fn play_move(&mut self, from: (u8, u8), to: (u8, u8)) -> MoveResult;
}

pub type Layer = Box<dyn Interface>;
