use super::seeder::*;
use crate::controller::Controller;
use crate::data_transfer::Cell;
use crate::value_objects::{Position, Velocity};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::collections::VecDeque;

pub struct Options<const N_ROWS: usize, const N_COLS: usize> {
    pub n_foods: usize,
    pub seeder: Box<dyn Seeder>,
}

impl<const N_ROWS: usize, const N_COLS: usize> Options<N_ROWS, N_COLS> {
    pub fn new(n_foods: usize) -> Self {
        Options {
            n_foods,
            seeder: Box::new(SecondsSeeder::SECONDS_SEEDER),
        }
    }

    pub fn with_mock_seeder(n_foods: usize, seed: u64) -> Self {
        Options {
            n_foods,
            seeder: Box::new(MockSeeder(seed)),
        }
    }
}

impl<const N_ROWS: usize, const N_COLS: usize> Options<N_ROWS, N_COLS> {
    pub fn build(
        &self,
        controller: Box<dyn Controller>,
    ) -> Result<GameState<N_ROWS, N_COLS>, InvalidOptions> {
        if self.is_valid() {
            Ok(self.get_game_state(controller))
        } else {
            Err(InvalidOptions)
        }
    }

    fn get_game_state(&self, controller: Box<dyn Controller>) -> GameState<N_ROWS, N_COLS> {
        let board = Options::get_board();
        let mut game_state = self.get_init_game_state(board, controller);
        self.add_foods(&mut game_state);
        game_state
    }

    fn get_board() -> [[Cell; N_ROWS]; N_COLS] {
        let mut board = [[Cell::Empty; N_ROWS]; N_COLS];
        board[N_ROWS / 2][N_COLS / 2] = Cell::Snake(None);
        board
    }

    fn get_init_game_state(
        &self,
        board: [[Cell; N_ROWS]; N_COLS],
        controller: Box<dyn Controller>,
    ) -> GameState<N_ROWS, N_COLS> {
        GameState {
            board,
            empty: Options::get_empty(&board),
            snake: self.get_snake(),
            controller,
            rng: self.get_rng(),
        }
    }

    fn get_empty(board: &[[Cell; N_ROWS]; N_COLS]) -> Vec<Position> {
        Vec::from_iter(board.iter().enumerate().flat_map(|(i, row)| {
            row.iter()
                .enumerate()
                .filter(|(_, cell)| matches!(cell, Cell::Empty))
                .map(move |(j, _)| Position(i, j))
        }))
    }

    fn get_snake(&self) -> VecDeque<Position> {
        VecDeque::from([Position(N_ROWS / 2, N_COLS / 2)])
    }

    fn get_rng(&self) -> ChaCha8Rng {
        let seed = self.seeder.get_seed();
        ChaCha8Rng::seed_from_u64(seed)
    }

    fn add_foods(&self, game_state: &mut GameState<N_ROWS, N_COLS>) {
        for _ in 0..self.n_foods {
            game_state.add_food().expect("room for foods");
        }
    }

    fn is_valid(&self) -> bool {
        self.area() >= self.n_non_empty()
    }

    fn area(&self) -> usize {
        N_ROWS * N_COLS
    }

    fn n_non_empty(&self) -> usize {
        let n_snake = 1;
        self.n_foods + n_snake
    }
}

#[derive(Debug)]
pub struct InvalidOptions;

#[cfg(test)]
mod options_tests {
    use super::*;
    use crate::controller::MockController;
    use crate::data_transfer::Direction;

    const EXPECTED_BOARD: [[Cell; 3]; 3] = [
        [Cell::Foods, Cell::Empty, Cell::Empty],
        [Cell::Empty, Cell::Snake(None), Cell::Empty],
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
        assert_eq!(game_state.board, EXPECTED_BOARD);
        assert_eq!(game_state.empty, Vec::from(EXPECTED_EMPTY));
        assert_eq!(game_state.snake, VecDeque::from(EXPECTED_SNAKE));
    }

    #[test]
    fn build_with_invalid() {
        let options = Options::<3, 3>::with_mock_seeder(9, 0);
        let controller = Box::new(MockController {
            direction: Direction::Right,
        });
        let game_state = options.build(controller).unwrap_err();
        assert!(matches!(game_state, InvalidOptions));
    }

    #[test]
    fn is_valid_true() {
        let options = Options::<3, 3>::with_mock_seeder(8, 0);
        assert!(options.is_valid());
    }

    #[test]
    fn is_valid_false() {
        let options = Options::<3, 3>::with_mock_seeder(9, 0);
        assert!(!options.is_valid());
    }

    #[test]
    fn area() {
        let options = Options::<3, 4>::with_mock_seeder(1, 0);
        assert_eq!(options.area(), 12);
    }

    #[test]
    fn n_non_empty() {
        let options = Options::<3, 3>::with_mock_seeder(1, 0);
        assert_eq!(options.n_non_empty(), 2);
    }
}

#[derive(Debug)]
pub struct GameIsOver;

#[derive(Debug)]
pub struct MaxFoods;

#[derive(Debug)]
pub struct GameState<const N_ROWS: usize, const N_COLS: usize> {
    board: [[Cell; N_ROWS]; N_COLS],
    empty: Vec<Position>,
    snake: VecDeque<Position>,
    controller: Box<dyn Controller>,
    rng: ChaCha8Rng,
}

impl<const N_ROWS: usize, const N_COLS: usize> GameState<N_ROWS, N_COLS> {
    /// This builds a `GameState` from a board without checking for invariants
    // fn from_board(
    //     board: [[Cell; N_COLS]; N_ROWS],
    //     controller: Box<dyn Controller>,
    // ) -> GameState<N_ROWS, N_COLS> {
    // }

    fn find_snake_head((i, row): (usize, &[Cell; N_COLS])) -> Option<(usize, usize)> {
        row.iter().enumerate().find_map(|(j, &cell)| {
            if cell == Cell::Snake(None) {
                Some((i, j))
            } else {
                None
            }
        })
    }

    pub fn iterate_turn(&mut self) -> Result<(), GameIsOver> {
        todo!();
    }

    fn get_next_head(&mut self) -> Position {
        let direction = self.controller.get_direction();
        let position = self.get_head();
        let velocity = direction.as_velocity();
        let i = position
            .0
            .checked_add_signed(velocity.0)
            .unwrap_or(N_ROWS - Velocity::DEFAULT_MAGNITUDE)
            % N_ROWS;
        let j = position
            .1
            .checked_add_signed(velocity.1)
            .unwrap_or(N_COLS - Velocity::DEFAULT_MAGNITUDE)
            % N_COLS;
        Position(i, j)
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
            self.board[i][j] = Cell::Foods;
            Ok(())
        }
    }
}

#[cfg(test)]
mod game_state_tests {
    use super::*;
    use crate::controller::MockController;
    use crate::data_transfer::Direction;

    // const INPUT_BOARD: [[Cell; 3]; 3] = [
    //     [Cell::Empty, Cell::Foods, Cell::Empty],
    //     [Cell::Empty, Cell::Snake(None), Cell::Empty],
    //     [
    //         Cell::Snake(Some(Direction::Right)),
    //         Cell::Snake(Some(Direction::Up)),
    //         Cell::Empty,
    //     ],
    // ];
    //
    // const EXPECTED_EMPTY: [Position; 5] = [
    //     Position(0, 0),
    //     Position(0, 2),
    //     Position(1, 0),
    //     Position(1, 2),
    //     Position(2, 2),
    // ];
    //
    // const EXPECTED_SNAKE: [Position; 3] = [Position(1, 1), Position(2, 1), Position(2, 0)];
    //
    // #[test]
    // pub fn from_board() {
    //     let controller = Box::new(MockController {
    //         direction: Direction::Right,
    //     });
    //     let game_state = GameState::from_board(INPUT_BOARD, controller);
    //     assert_eq!(game_state.empty, EXPECTED_EMPTY);
    //     assert_eq!(game_state.snake, VecDeque::from(EXPECTED_SNAKE));
    // }

    #[test]
    pub fn get_next_head() {
        let options = Options::<3, 3>::with_mock_seeder(1, 0);
        let controller = Box::new(MockController {
            direction: Direction::Right,
        });
        let mut game_state = options.build(controller).unwrap();
        assert_eq!(game_state.get_next_head(), Position(1, 2));
    }

    // #[test]
    // pub fn get_next_head_wrapping() {
    //     let options = Options::<3, 3>::with_mock_seeder(1, 0);
    //     let controller = Box::new(MockController {
    //         direction: Direction::Right,
    //     });
    //     let mut game_state = options.build(controller).unwrap();
    //     game_state.iterate_turn();
    //     game_state.head = assert_eq!(game_state.get_next_head(), Position(1, 0));
    // }

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
