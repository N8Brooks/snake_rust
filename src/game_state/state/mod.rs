// use std::collections::VecDeque;

mod board;
mod value_objects;

// TODO: separate state
pub use board::Board as State;
pub use value_objects::*;

// #[derive(Debug, Clone, PartialEq)]
// pub struct State<const N_ROWS: usize, const N_COLS: usize> {
//     board: Board<N_ROWS, N_COLS>,
//     empty: Vec<Position>,
//     foods: Vec<Position>,
//     snake: VecDeque<Position>,
// }
//
// impl<const N_ROWS: usize, const N_COLS: usize> State<N_ROWS, N_COLS> {
//     fn new(board: &[[dto::Cell; N_COLS]; N_ROWS]) -> State<N_ROWS, N_COLS> {
//         let board = Board::from(board);
//         let empty = board.get_empty();
//         let foods = board.get_foods();
//
//         State {
//             board,
//             empty: board.get_empty(),
//             foods: board.get_foods(),
//             snake: board.get_snake(),
//         }
//     }
// }
//
// #[cfg(test)]
// mod tests {}
