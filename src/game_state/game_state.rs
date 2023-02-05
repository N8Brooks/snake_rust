use super::seeder::*;
use crate::data_transfer::{Cell, Direction};
use crate::value_objects::{InvalidDirection, Position, Velocity};
use std::collections::{HashSet, VecDeque};

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
    pub fn build(&self) -> Result<GameState<N_ROWS, N_COLS>, InvalidOptions> {
        let mut board = [[Cell::Empty; N_ROWS]; N_COLS];
        board[N_ROWS / 2][N_COLS / 2] = Cell::Snake(None);
        if self.is_valid() {
            Ok(GameState {
                velocity: Velocity(0, 0),
                board,
                empty: HashSet::from_iter(board.iter().enumerate().flat_map(|(i, row)| {
                    row.iter()
                        .enumerate()
                        .filter(|(_, cell)| matches!(cell, Cell::Empty))
                        .map(move |(j, _)| Position(i, j))
                })),
                snake: VecDeque::from([Position(N_ROWS / 2, N_COLS / 2)]),
            })
        } else {
            Err(InvalidOptions)
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

    const EXPECTED_BOARD: [[Cell; 3]; 3] = [
        [Cell::Empty; 3],
        [Cell::Empty, Cell::Snake(None), Cell::Empty],
        [Cell::Empty; 3],
    ];

    const EXPECTED_EMPTY: [Position; 8] = [
        Position(0, 0),
        Position(0, 1),
        Position(0, 2),
        Position(1, 0),
        Position(1, 2),
        Position(2, 0),
        Position(2, 1),
        Position(2, 2),
    ];

    const EXPECTED_SNAKE: [Position; 1] = [Position(1, 1)];

    #[test]
    fn build_with_valid() {
        let options = Options::<3, 3>::with_mock_seeder(1, 0);
        let game_state = options.build().unwrap();
        assert_eq!(game_state.velocity, Velocity(0, 0));
        assert_eq!(game_state.board, EXPECTED_BOARD);
        let expected_empty = HashSet::from(EXPECTED_EMPTY);
        let empty_diff_count = game_state
            .empty
            .symmetric_difference(&expected_empty)
            .count();
        assert_eq!(empty_diff_count, 0);
        assert_eq!(game_state.snake, VecDeque::from(EXPECTED_SNAKE));
    }

    #[test]
    fn build_with_invalid() {
        let options = Options::<3, 3>::with_mock_seeder(9, 0);
        let game_state = options.build().unwrap_err();
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
pub struct GameState<const N_ROWS: usize, const N_COLS: usize> {
    velocity: Velocity,
    board: [[Cell; N_ROWS]; N_COLS],
    empty: HashSet<Position>,
    snake: VecDeque<Position>,
}

impl<const N_ROWS: usize, const N_COLS: usize> GameState<N_ROWS, N_COLS> {
    pub fn set_direction(&mut self, direction: Direction) -> Result<(), InvalidDirection> {
        let velocity = Velocity::from_direction(&direction);
        self.velocity.check_acceleration(&velocity)?;
        self.velocity = velocity;
        Ok(())
    }
}

#[cfg(test)]
mod game_state_tests {
    use super::*;

    #[test]
    fn set_direction_valid() {
        let mut game_state = Options::<3, 3>::new(1).build().unwrap();
        assert!(game_state.set_direction(Direction::Up).is_ok());
        assert_eq!(game_state.velocity, Velocity(-1, 0));
    }

    #[test]
    fn set_direction() {
        let mut game_state = Options::<3, 3>::new(1).build().unwrap();
        game_state.velocity = Velocity::from_direction(&Direction::Up);
        assert!(game_state.set_direction(Direction::Down).is_err());
    }
}

impl Velocity {
    pub fn check_acceleration(&self, other: &Velocity) -> Result<(), InvalidDirection> {
        if self.is_vertical() != other.is_vertical() {
            Ok(())
        } else {
            Err(InvalidDirection)
        }
    }
}

#[cfg(test)]
mod velocity_tests {
    use crate::value_objects::Velocity;

    #[test]
    fn check_acceleration_ok() {
        let v_0 = Velocity(1, 0);
        let v_1 = Velocity(0, 1);
        assert!(v_0.check_acceleration(&v_1).is_ok());
    }

    #[test]
    fn check_acceleration_err() {
        let v_0 = Velocity(1, 0);
        let v_1 = Velocity(-1, 0);
        assert!(v_0.check_acceleration(&v_1).is_err())
    }
}
