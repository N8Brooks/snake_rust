use super::seeder::*;
use crate::controller::Controller;
use crate::data_transfer::{Cell, Direction};
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
        let board = Board::<N_ROWS, N_COLS>::default();
        let mut game_state = self.get_init_game_state(board, controller);
        self.add_foods(&mut game_state);
        game_state
    }

    fn get_init_game_state(
        &self,
        board: Board<N_ROWS, N_COLS>,
        controller: Box<dyn Controller>,
    ) -> GameState<N_ROWS, N_COLS> {
        let empty = board.get_empty();
        GameState {
            board,
            empty,
            snake: self.get_snake(),
            controller,
            rng: self.get_rng(),
        }
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
    board: Board<N_ROWS, N_COLS>,
    empty: Vec<Position>,
    snake: VecDeque<Position>,
    controller: Box<dyn Controller>,
    rng: ChaCha8Rng,
}

impl<const N_ROWS: usize, const N_COLS: usize> GameState<N_ROWS, N_COLS> {
    /// This builds a `GameState` from a board without checking for invariants
    fn from_board(
        board: Board<N_ROWS, N_COLS>,
        controller: Box<dyn Controller>,
    ) -> GameState<N_ROWS, N_COLS> {
        todo!();
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
mod game_state_tests {
    use super::*;
    use crate::controller::MockController;
    use crate::data_transfer::Direction;

    const INPUT_BOARD: [[Cell; 3]; 3] = [
        [Cell::Empty, Cell::Foods, Cell::Empty],
        [
            Cell::Empty,
            Cell::Snake {
                entry: Some(Direction::Down),
                exit: None,
            },
            Cell::Empty,
        ],
        [
            Cell::Snake {
                entry: None,
                exit: Some(Direction::Up),
            },
            Cell::Snake {
                entry: Some(Direction::Left),
                exit: Some(Direction::Right),
            },
            Cell::Empty,
        ],
    ];

    const EXPECTED_EMPTY: [Position; 5] = [
        Position(0, 0),
        Position(0, 2),
        Position(1, 0),
        Position(1, 2),
        Position(2, 2),
    ];

    const EXPECTED_SNAKE: [Position; 3] = [Position(1, 1), Position(2, 1), Position(2, 0)];

    #[test]
    pub fn from_board() {
        let controller = Box::new(MockController {
            direction: Direction::Right,
        });
        let game_state = GameState::from_board(Board(INPUT_BOARD), controller);
        assert_eq!(game_state.empty, EXPECTED_EMPTY);
        assert_eq!(game_state.snake, VecDeque::from(EXPECTED_SNAKE));
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

#[derive(Debug, PartialEq)]
pub struct Board<const N_ROWS: usize, const N_COLS: usize>(pub [[Cell; N_COLS]; N_ROWS]);

impl<const N_ROWS: usize, const N_COLS: usize> Default for Board<N_ROWS, N_COLS> {
    fn default() -> Self {
        let mut board = [[Cell::Empty; N_COLS]; N_ROWS];
        board[N_ROWS / 2][N_COLS / 2] = Cell::Snake {
            entry: None,
            exit: None,
        };
        Board(board)
    }
}

impl<const N_ROWS: usize, const N_COLS: usize> Board<N_ROWS, N_COLS> {
    pub fn get_empty(&self) -> Vec<Position> {
        Vec::from_iter(self.0.iter().enumerate().flat_map(|(i, row)| {
            row.iter()
                .enumerate()
                .filter(|(_, cell)| matches!(cell, Cell::Empty))
                .map(move |(j, _)| Position(i, j))
        }))
    }

    pub fn get_snake(&self) -> VecDeque<Position> {
        let mut position = self.find_snake_head().expect("snake head");
        let mut snake = VecDeque::from([position]);
        while let Cell::Snake {
            entry: Some(direction),
            exit: _,
        } = self.at(&position)
        {
            position = self.move_in(&position, &direction);
            snake.push_back(position);
        }
        snake
    }

    pub fn at(&self, position: &Position) -> Cell {
        let Position(i, j) = position;
        self.0[*i][*j]
    }

    fn find_snake_head(&self) -> Option<Position> {
        self.0
            .iter()
            .enumerate()
            .find_map(|item| self.find_snake_head_from_row(item))
    }

    fn find_snake_head_from_row(&self, (i, row): (usize, &[Cell; N_COLS])) -> Option<Position> {
        row.iter().enumerate().find_map(|(j, &cell)| {
            if matches!(cell, Cell::Snake { exit: None, .. }) {
                Some(Position(i, j))
            } else {
                None
            }
        })
    }

    pub fn move_in(&self, position: &Position, direction: &Direction) -> Position {
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
}

#[cfg(test)]
mod board_tests {
    use super::*;

    const INPUT_BOARD: [[Cell; 3]; 3] = [
        [Cell::Empty, Cell::Foods, Cell::Empty],
        [
            Cell::Empty,
            Cell::Snake {
                entry: Some(Direction::Down),
                exit: None,
            },
            Cell::Empty,
        ],
        [
            Cell::Snake {
                entry: None,
                exit: Some(Direction::Up),
            },
            Cell::Snake {
                entry: Some(Direction::Left),
                exit: Some(Direction::Right),
            },
            Cell::Empty,
        ],
    ];

    const EXPECTED_EMPTY: [Position; 5] = [
        Position(0, 0),
        Position(0, 2),
        Position(1, 0),
        Position(1, 2),
        Position(2, 2),
    ];

    const EXPECTED_SNAKE: [Position; 3] = [Position(1, 1), Position(2, 1), Position(2, 0)];

    #[test]
    fn get_empty() {
        let board = Board(INPUT_BOARD);
        let empty = board.get_empty();
        assert_eq!(empty, EXPECTED_EMPTY);
    }

    #[test]
    fn parse_snake() {
        let board = Board(INPUT_BOARD);
        let snake = board.get_snake();
        assert_eq!(snake, EXPECTED_SNAKE);
    }
}
