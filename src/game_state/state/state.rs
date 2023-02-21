use std::collections::VecDeque;

use rand_chacha::ChaCha8Rng;

use super::{board::Board, value_objects::*};

#[derive(Debug, Clone, PartialEq)]
pub struct State<const N_ROWS: usize, const N_COLS: usize> {
    pub board: Board<N_ROWS, N_COLS>,
    pub empty: Vec<Position>,
    pub foods: Vec<Position>,
    pub snake: VecDeque<Position>,
    pub rng: ChaCha8Rng,
}

impl<const N_ROWS: usize, const N_COLS: usize> State<N_ROWS, N_COLS> {
    pub fn new(board: Board<N_ROWS, N_COLS>, rng: ChaCha8Rng) -> State<N_ROWS, N_COLS> {
        let empty = board.get_empty();
        let foods = board.get_foods();
        let snake = board.get_snake();
        State {
            board,
            empty,
            foods,
            snake,
            rng,
        }
    }
}

#[cfg(test)]
mod tests {}
