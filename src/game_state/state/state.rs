use std::collections::VecDeque;

use rand_chacha::ChaCha8Rng;

use crate::data_transfer_objects as dto;

use super::{board::Board, value_objects::*};

// TODO: add update object
// TODO: add is_valid

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

    pub fn is_valid(&self) -> bool {
        // A valid `State`
        // * All `Position`s in `empty`, `foods`, and `snake` are unique and have a count of
        //   `N_ROWS * N_COLS`.
        // * `self.at(empty[i]) == Cell::Empty(i)` for each `i in 0..empty.len()`
        // * `self.at(foods[i]) == Cell::Foods(i)` for each `i in 0..foods.len()`
        // * `self.at(snake[i]) == Cell::Snake { .. }` for each  `i in 0..snake.len()`
        // * The snake itself is valid by having exactly one head and tail that lead to each
        // other.
        todo!()
    }

    fn is_board_valid(&self) -> bool {
        todo!()
    }

    fn is_empty_valid(&self) -> bool {
        self.empty
            .iter()
            .enumerate()
            .all(|(i, position)| match self.board.at(position) {
                Cell::Empty(j) => i == j,
                _ => false,
            })
    }

    fn is_foods_valid(&self) -> bool {
        self.foods
            .iter()
            .enumerate()
            .all(|(i, position)| match self.board.at(position) {
                Cell::Foods(j) => i == j,
                _ => false,
            })
    }

    fn is_snake_valid(&self) -> bool {
        self.snake
            .iter()
            .all(|position| matches!(self.board.at(position), Cell::Snake { .. }))
    }

    pub fn check_is_won_status(&self) -> dto::Status {
        if self.empty.is_empty() && self.foods.is_empty() {
            dto::Status::Over { is_won: true }
        } else {
            dto::Status::Ongoing
        }
    }

    pub fn get_next_head(&self, direction: &Direction) -> Position {
        let head = self.snake.front().expect("snake head");
        self.board.move_in(head, direction)
    }

    pub fn remove_last_tail(&mut self) -> Position {
        let tail = self.snake.pop_back().expect("snake tail");
        *self.board.at_mut(&tail) = if let Cell::Snake(Path {
            entry: None,
            exit: _,
        }) = self.board.at(&tail)
        {
            Cell::Empty(self.empty.len())
        } else {
            panic!("invariant invalid snake {:?}", self.board.at(&tail))
        };
        self.empty.push(tail);
        tail
    }
}

#[cfg(test)]
mod tests {
    use crate::seeder::{MockSeeder, Seeder};

    use super::*;

    const MOCK_BOARD: [[Cell; 3]; 2] = [
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

    fn get_mock_state() -> State<2, 3> {
        let rng = MockSeeder(0).get_rng();
        let board = Board::new(MOCK_BOARD);
        State::new(board, rng)
    }

    fn get_two_cell() -> State<1, 2> {
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
        State::new(board, rng)
    }

    // #[test]
    // fn is_valid_true() {
    //     let state = get_mock_state();
    //     assert!(state.is_valid());
    // }

    #[test]
    fn is_empty_valid_false() {
        let board = Board::new([[
            Cell::Snake(Path {
                entry: None,
                exit: None,
            }),
            Cell::Empty(1),
        ]]);
        let empty = board.get_empty();
        let foods = board.get_foods();
        let snake = board.get_snake();
        let state = State {
            board,
            empty,
            foods,
            snake,
            rng: MockSeeder(0).get_rng(),
        };
        assert!(!state.is_empty_valid());
    }

    #[test]
    fn is_foods_valid_false() {
        let board = Board::new([[
            Cell::Snake(Path {
                entry: None,
                exit: None,
            }),
            Cell::Foods(0),
        ]]);
        let empty = board.get_empty();
        let foods = vec![Position(0, 0)];
        let snake = board.get_snake();
        let state = State {
            board,
            empty,
            foods,
            snake,
            rng: MockSeeder(0).get_rng(),
        };
        assert!(!state.is_foods_valid());
    }

    #[test]
    fn is_snake_valid_false() {
        let board = Board::new([[
            Cell::Snake(Path {
                entry: None,
                exit: None,
            }),
            Cell::Empty(0),
        ]]);
        let empty = board.get_empty();
        let foods = board.get_foods();
        let snake = VecDeque::from([Position(0, 1)]);
        let state = State {
            board,
            empty,
            foods,
            snake,
            rng: MockSeeder(0).get_rng(),
        };
        assert!(!state.is_snake_valid());
    }

    #[test]
    fn check_is_won_status_true() {
        assert_eq!(
            get_two_cell().check_is_won_status(),
            dto::Status::Over { is_won: true }
        );
    }

    #[test]
    fn check_is_won_status_false() {
        let state = get_mock_state();
        let status = state.check_is_won_status();
        assert_eq!(status, dto::Status::Ongoing);
    }

    #[test]
    fn get_next_head() {
        let state = get_mock_state();
        let direction = Direction::Right;
        let head = state.get_next_head(&direction);
        assert_eq!(head, Position(1, 2));
    }

    #[test]
    fn remove_last_tail() {
        let mut state = get_mock_state();
        let position = Position(0, 2);
        assert_eq!(state.remove_last_tail(), position);
        assert_eq!(state.board.at(&position), Cell::Empty(1))
        // assert.is_valid()
    }
}
