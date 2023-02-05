use super::seeder::*;
use crate::data_transfer::Cell;
use crate::value_objects::Position;
use std::collections::{HashSet, VecDeque};

pub struct Options<const N_ROWS: usize, const N_COLS: usize, T: Seeder> {
    pub n_foods: usize,
    pub seeder: T,
}

impl<const N_ROWS: usize, const N_COLS: usize> Options<N_ROWS, N_COLS, SecondsSeeder> {
    pub fn with_n_foods(n_foods: usize) -> Self {
        Options {
            n_foods,
            seeder: SecondsSeeder::SECONDS_SEEDER,
        }
    }
}

impl<const N_ROWS: usize, const N_COLS: usize> Options<N_ROWS, N_COLS, MockSeeder> {
    pub fn with_n_foods(n_foods: usize) -> Self {
        Options {
            n_foods,
            seeder: MockSeeder(0),
        }
    }
}

impl<const N_ROWS: usize, const N_COLS: usize, T: Seeder> Options<N_ROWS, N_COLS, T> {
    pub fn build(&self) -> Result<GameState<N_ROWS, N_COLS>, InvalidOptions> {
        let mut board = [[Cell::Empty; N_ROWS]; N_COLS];
        board[N_ROWS / 2][N_COLS / 2] = Cell::Snake(None);
        if self.is_valid() {
            Ok(GameState {
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
        let options = Options::<3, 3, MockSeeder>::with_n_foods(1);
        let game_state = options.build().unwrap();
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
        let mut options = Options::<3, 3, MockSeeder>::with_n_foods(1);
        options.n_foods = 9;
        let game_state = options.build().unwrap_err();
        assert!(matches!(game_state, InvalidOptions));
    }

    #[test]
    fn is_valid_true() {
        let options = Options::<3, 3, MockSeeder>::with_n_foods(8);
        assert!(options.is_valid());
    }

    #[test]
    fn is_valid_false() {
        let options = Options::<3, 3, MockSeeder>::with_n_foods(9);
        assert!(!options.is_valid());
    }

    #[test]
    fn area() {
        let options = Options::<3, 4, MockSeeder>::with_n_foods(1);
        assert_eq!(options.area(), 12);
    }

    #[test]
    fn n_non_empty() {
        let options = Options::<3, 3, MockSeeder>::with_n_foods(1);
        assert_eq!(options.n_non_empty(), 2);
    }
}

#[derive(Debug)]
pub struct GameState<const N_ROWS: usize, const N_COLS: usize> {
    board: [[Cell; N_ROWS]; N_COLS],
    empty: HashSet<Position>,
    snake: VecDeque<Position>,
}
