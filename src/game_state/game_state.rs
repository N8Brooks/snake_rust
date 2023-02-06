use std::collections::VecDeque;

use crate::controller::Controller;
use crate::data_transfer::Cell;
use crate::value_objects::Position;
use rand::Rng;
use rand_chacha::ChaCha8Rng;

use super::board::Board;
use super::Options;

#[derive(Debug)]
pub struct GameIsOver;

#[derive(Debug)]
pub struct MaxFoods;

#[derive(Debug)]
pub struct GameState<const N_ROWS: usize, const N_COLS: usize> {
    board: Board<N_ROWS, N_COLS>,
    empty: Vec<Position>,
    snake: VecDeque<Position>,
    controller: Box<dyn Controller>,
    rng: ChaCha8Rng,
}

impl<const N_ROWS: usize, const N_COLS: usize> GameState<N_ROWS, N_COLS> {
    pub fn from_options(
        options: &Options<N_ROWS, N_COLS>,
        controller: Box<dyn Controller>,
    ) -> GameState<N_ROWS, N_COLS> {
        let board = Board::<N_ROWS, N_COLS>::default();
        let mut game_state = options.get_init_game_state(board, controller);
        options.add_foods(&mut game_state);
        game_state
    }

    /// This builds a `GameState` from a board without checking for invariants
    fn from_board(
        board: Board<N_ROWS, N_COLS>,
        controller: Box<dyn Controller>,
        rng: ChaCha8Rng,
    ) -> GameState<N_ROWS, N_COLS> {
        let empty = board.get_empty();
        let snake = board.get_snake();
        GameState {
            board,
            empty,
            snake,
            controller,
            rng,
        }
    }

    pub fn iterate_turn(&mut self) -> Result<(), GameIsOver> {
        todo!();
    }

    fn get_next_head(&mut self) -> Position {
        let direction = self.controller.get_direction();
        let position = self.get_head();
        self.board.move_in(position, &direction)
    }

    fn get_head(&self) -> &Position {
        self.snake.front().expect("non empty snake")
    }

    fn add_food(&mut self) -> Result<(), MaxFoods> {
        if self.empty.is_empty() {
            Err(MaxFoods)
        } else {
            let foods_index = self.rng.gen_range(0..self.empty.len());
            let Position(i, j) = self.empty.swap_remove(foods_index);
            self.board.0[i][j] = Cell::Foods;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;

    use crate::controller::MockController;
    use crate::data_transfer::Direction;

    use super::*;

    #[test]
    pub fn from_board() {
        let board = Board([[Cell::Snake {
            entry: None,
            exit: None,
        }]]);
        let controller = Box::new(MockController {
            direction: Direction::Right,
        });
        let rng = ChaCha8Rng::seed_from_u64(0);
        let game_state = GameState::from_board(board, controller, rng);
        assert_eq!(game_state.empty, Vec::new());
        assert_eq!(game_state.snake, VecDeque::from([Position(0, 0)]));
    }

    #[test]
    pub fn get_next_head() {
        let options = Options::<3, 3>::with_mock_seeder(1, 0);
        let controller = Box::new(MockController {
            direction: Direction::Right,
        });
        let mut game_state = options.build(controller).unwrap();
        assert_eq!(game_state.get_next_head(), Position(1, 2));
    }

    #[test]
    pub fn get_head() {
        let options = Options::<3, 3>::with_mock_seeder(1, 0);
        let controller = Box::new(MockController {
            direction: Direction::Right,
        });
        let game_state = options.build(controller).unwrap();
        assert_eq!(*game_state.get_head(), Position(1, 1));
    }
}

impl<const N_ROWS: usize, const N_COLS: usize> Options<N_ROWS, N_COLS> {
    fn get_init_game_state(
        &self,
        board: Board<N_ROWS, N_COLS>,
        controller: Box<dyn Controller>,
    ) -> GameState<N_ROWS, N_COLS> {
        let empty = board.get_empty();
        let snake = board.get_snake();
        GameState {
            board,
            empty,
            snake,
            controller,
            rng: self.seeder.get_rng(),
        }
    }

    fn add_foods(&self, game_state: &mut GameState<N_ROWS, N_COLS>) {
        for _ in 0..self.n_foods {
            game_state.add_food().expect("room for foods");
        }
    }
}

#[cfg(test)]
mod options_tests {
    use crate::{controller::MockController, data_transfer::Direction};

    use super::*;

    const EXPECTED_BOARD: [[Cell; 3]; 3] = [
        [Cell::Foods, Cell::Empty, Cell::Empty],
        [
            Cell::Empty,
            Cell::Snake {
                entry: None,
                exit: None,
            },
            Cell::Empty,
        ],
        [Cell::Empty; 3],
    ];

    const EXPECTED_EMPTY: [Position; 7] = [
        Position(2, 2),
        Position(0, 1),
        Position(0, 2),
        Position(1, 0),
        Position(1, 2),
        Position(2, 0),
        Position(2, 1),
    ];

    const EXPECTED_SNAKE: [Position; 1] = [Position(1, 1)];

    #[test]
    fn build_with_valid() {
        let options = Options::<3, 3>::with_mock_seeder(1, 0);
        let controller = Box::new(MockController {
            direction: Direction::Right,
        });
        let game_state = options.build(controller).unwrap();
        assert_eq!(game_state.board, Board(EXPECTED_BOARD));
        assert_eq!(game_state.empty, Vec::from(EXPECTED_EMPTY));
        assert_eq!(game_state.snake, VecDeque::from(EXPECTED_SNAKE));
    }
}
