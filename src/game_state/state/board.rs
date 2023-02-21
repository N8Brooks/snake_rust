use std::collections::VecDeque;

use crate::data_transfer_objects as _dto; // Limited usage in `from`

use super::value_objects::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Board<const N_ROWS: usize, const N_COLS: usize>([[Cell; N_COLS]; N_ROWS]);

impl<const N_ROWS: usize, const N_COLS: usize> Default for Board<N_ROWS, N_COLS> {
    fn default() -> Self {
        let mut empty_index = 0;
        let board = (0..N_ROWS)
            .map(|i| {
                (0..N_COLS)
                    .map(|j| {
                        if i == N_ROWS / 2 && j == N_COLS / 2 {
                            Cell::Snake(Path {
                                entry: None,
                                exit: None,
                            })
                        } else {
                            let empty = Cell::Empty(empty_index);
                            empty_index += 1;
                            empty
                        }
                    })
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap()
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        Board(board)
    }
}

impl<const N_ROWS: usize, const N_COLS: usize> Board<N_ROWS, N_COLS> {
    pub fn new(board: [[Cell; N_COLS]; N_ROWS]) -> Self {
        Board(board)
    }

    pub fn get_empty(&self) -> Vec<Position> {
        Vec::from_iter(self.0.iter().enumerate().flat_map(|(i, row)| {
            row.iter()
                .enumerate()
                .filter(|(_, cell)| matches!(cell, Cell::Empty(_)))
                .map(move |(j, _)| Position(i, j))
        }))
    }

    pub fn get_foods(&self) -> Vec<Position> {
        Vec::from_iter(self.0.iter().enumerate().flat_map(|(i, row)| {
            row.iter()
                .enumerate()
                .filter(|(_, cell)| matches!(cell, Cell::Foods(_)))
                .map(move |(j, _)| Position(i, j))
        }))
    }

    pub fn get_snake(&self) -> VecDeque<Position> {
        let mut position = self.find_snake_head().expect("snake head");
        let mut snake = VecDeque::from([position]);
        while let Cell::Snake(Path {
            entry: Some(direction),
            exit: _,
        }) = self.at(&position)
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

    pub fn at_mut(&mut self, position: &Position) -> &mut Cell {
        let Position(i, j) = position;
        &mut self.0[*i][*j]
    }

    fn find_snake_head(&self) -> Option<Position> {
        self.0
            .iter()
            .enumerate()
            .find_map(|item| self.find_snake_head_from_row(item))
    }

    fn find_snake_head_from_row(&self, (i, row): (usize, &[Cell; N_COLS])) -> Option<Position> {
        row.iter().enumerate().find_map(|(j, &cell)| {
            if matches!(cell, Cell::Snake(Path { exit: None, .. })) {
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

impl<const N_ROWS: usize, const N_COLS: usize> From<[[_dto::Cell; N_COLS]; N_ROWS]>
    for Board<N_ROWS, N_COLS>
{
    fn from(board: [[_dto::Cell; N_COLS]; N_ROWS]) -> Self {
        let mut empty_count = 0;
        let mut foods_count = 0;
        let board = board.map(|row| {
            row.map(|cell| match cell {
                _dto::Cell::Empty => {
                    let empty_index = empty_count;
                    empty_count += 1;
                    Cell::Empty(empty_index)
                }
                _dto::Cell::Foods => {
                    let foods_index = foods_count;
                    foods_count += 1;
                    Cell::Foods(foods_index)
                }
                _dto::Cell::Snake(path) => Cell::Snake(path),
            })
        });
        Board::new(board)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT_BOARD: [[Cell; 3]; 3] = [
        [Cell::Empty(0), Cell::Foods(0), Cell::Empty(1)],
        [
            Cell::Empty(2),
            Cell::Snake(Path {
                entry: Some(Direction::Down),
                exit: None,
            }),
            Cell::Empty(3),
        ],
        [
            Cell::Snake(Path {
                entry: None,
                exit: Some(Direction::Up),
            }),
            Cell::Snake(Path {
                entry: Some(Direction::Left),
                exit: Some(Direction::Right),
            }),
            Cell::Empty(4),
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
        let board = Board::new(INPUT_BOARD);
        let empty = board.get_empty();
        assert_eq!(empty, EXPECTED_EMPTY);
    }

    #[test]
    fn parse_snake() {
        let board = Board::new(INPUT_BOARD);
        let snake = board.get_snake();
        assert_eq!(snake, EXPECTED_SNAKE);
    }

    #[test]
    fn at() {
        let board = Board::new(INPUT_BOARD);
        let position = Position(0, 1);
        let cell = board.at(&position);
        assert_eq!(cell, Cell::Foods(0));
    }

    #[test]
    fn at_mut() {
        let mut board = Board::new(INPUT_BOARD);
        let position = Position(2, 2);
        let cell = *board.at_mut(&position);
        assert_eq!(cell, Cell::Empty(4));
    }

    const DTO_BOARD: [[_dto::Cell; 3]; 3] = [
        [_dto::Cell::Empty, _dto::Cell::Foods, _dto::Cell::Empty],
        [
            _dto::Cell::Empty,
            _dto::Cell::Snake(Path {
                entry: Some(Direction::Down),
                exit: None,
            }),
            _dto::Cell::Empty,
        ],
        [
            _dto::Cell::Snake(Path {
                entry: None,
                exit: Some(Direction::Up),
            }),
            _dto::Cell::Snake(Path {
                entry: Some(Direction::Left),
                exit: Some(Direction::Right),
            }),
            _dto::Cell::Empty,
        ],
    ];

    #[test]
    fn from_dto() {
        let board: Board<3, 3> = DTO_BOARD.into();
        assert_eq!(board, Board::new(INPUT_BOARD));
    }
}
