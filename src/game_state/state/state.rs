use std::collections::VecDeque;

use rand_chacha::ChaCha8Rng;

use crate::data_transfer_objects as dto;

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

    pub fn check_is_won_status(&self) -> dto::Status {
        if self.empty.is_empty() && self.foods.is_empty() {
            dto::Status::Over { is_won: true }
        } else {
            dto::Status::Ongoing
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::seeder::{MockSeeder, Seeder};

    use super::*;

    const BOARD: [[Cell; 3]; 2] = [
        [
            Cell::Snake(Path {
                entry: Some(Direction::Right),
                exit: Some(Direction::Down),
            }),
            Cell::Snake(Path {
                entry: Some(Direction::Right),
                exit: Some(Direction::Left),
            }),
            Cell::Snake(Path {
                entry: None,
                exit: Some(Direction::Left),
            }),
        ],
        [
            Cell::Snake(Path {
                entry: Some(Direction::Up),
                exit: Some(Direction::Right),
            }),
            Cell::Snake(Path {
                entry: Some(Direction::Left),
                exit: None,
            }),
            Cell::Empty(0),
        ],
    ];

    #[test]
    fn check_is_won_status_true() {
        let direction = Direction::Right;
        let board = Board::new([[
            Cell::Snake(Path {
                entry: None,
                exit: Some(direction),
            }),
            Cell::Snake(Path {
                entry: Some(direction.opposite()),
                exit: None,
            }),
        ]]);
        let rng = MockSeeder(0).get_rng();
        let state = State::new(board, rng);
        assert_eq!(
            state.check_is_won_status(),
            dto::Status::Over { is_won: true }
        );
    }

    #[test]
    fn check_is_won_status_false() {
        let board = Board::new([[
            Cell::Snake(Path {
                entry: None,
                exit: None,
            }),
            Cell::Empty(0),
        ]]);
        let rng = MockSeeder(0).get_rng();
        let state = State::new(board, rng);
        assert_eq!(state.check_is_won_status(), dto::Status::Ongoing);
    }
}
