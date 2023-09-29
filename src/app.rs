
use crate::{ State, Board, };

pub trait AppInterface {

    fn get_board(&self) -> Board;
    fn get_state(&self) -> State;
}

pub type App = Box<dyn AppInterface>;
