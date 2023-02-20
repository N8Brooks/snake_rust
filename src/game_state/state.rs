use std::collections::VecDeque;

use crate::{data_transfer_objects as dto, value_objects::*};

use super::board::Board;

#[derive(Debug, Clone, PartialEq)]
pub struct State<const N_ROWS: usize, const N_COLS: usize> {
    board: Board<N_ROWS, N_COLS>,
    empty: Vec<Position>,
    foods: Vec<Position>,
    snake: VecDeque<Position>,
}

impl<const N_ROWS: usize, const N_COLS: usize> State<N_ROWS, N_COLS> {
    // fn new(board: &[[dto::Cell; N_COLS]; N_ROWS]) -> State<N_ROWS, N_COLS> {
    //     let board = Board::from(board);
    //     let empty = board.get_empty();
    //     let foods = board.get_foods();
    //
    //     State {
    //         board,
    //         empty: board.get_empty(),
    //         foods: board.get_foods(),
    //         snake: board.get_snake(),
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
}
