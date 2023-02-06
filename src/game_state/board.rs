use std::collections::VecDeque;

use crate::data_transfer::{Cell, Direction};
use crate::value_objects::{Position, Velocity};

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
mod tests {
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
