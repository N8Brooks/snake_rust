use std::collections::VecDeque;

use crate::controller::Controller;
use crate::data_transfer::{Cell, Direction, Status};
use crate::value_objects::Position;
use rand::Rng;
use rand_chacha::ChaCha8Rng;

use super::board::Board;
use super::Options;

#[derive(Debug)]
pub struct MaxFoods;

#[derive(Debug)]
pub struct GameState<const N_ROWS: usize, const N_COLS: usize> {
    board: Board<N_ROWS, N_COLS>,
    empty: Vec<Position>,
    foods: Vec<Position>,
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
    pub fn from_board(
        board: Board<N_ROWS, N_COLS>,
        controller: Box<dyn Controller>,
        rng: ChaCha8Rng,
    ) -> GameState<N_ROWS, N_COLS> {
        let empty = board.get_empty();
        let foods = board.get_foods();
        let snake = board.get_snake();
        GameState {
            board,
            empty,
            foods,
            snake,
            controller,
            rng,
        }
    }

    pub fn iterate_turn(&mut self) -> Status {
        let direction = self.controller.get_direction();
        let next_head = self.get_next_head(&direction);
        let last_head = self.get_head().clone();
        match self.board.at(&next_head) {
            Cell::Empty(empty_index) => {
                assert_eq!(self.empty.swap_remove(empty_index), next_head);
                if empty_index < self.empty.len() {
                    let position = self.empty[empty_index];
                    *self.board.at_mut(&position) = Cell::Empty(empty_index);
                }

                *self.board.at_mut(&last_head) =
                    if let Cell::Snake { entry, exit: None } = self.board.at(&last_head) {
                        Cell::Snake {
                            entry,
                            exit: Some(direction),
                        }
                    } else {
                        panic!("invariant not snake {:?}", self.board.at(&last_head))
                    };

                let next_tail = self.snake.back().expect("non empty snake next tail");
                *self.board.at_mut(&next_tail) =
                    if let Cell::Snake { entry: _, exit } = self.board.at(&next_tail) {
                        Cell::Snake { entry: None, exit }
                    } else {
                        panic!("invariant not snake {:?}", self.board.at(&next_tail))
                    };

                let last_tail = self.snake.pop_back().expect("non empty snake last tail");
                *self.board.at_mut(&last_tail) = if let Cell::Snake {
                    entry: None,
                    exit: _,
                } = self.board.at(&last_tail)
                {
                    Cell::Empty(self.empty.len())
                } else {
                    panic!("invariant not snake {:?}", self.board.at(&last_tail))
                };
                self.empty.push(last_tail);

                self.snake.push_front(next_head);
                *self.board.at_mut(&next_head) = Cell::Snake {
                    entry: if self.snake.len() > 1 {
                        Some(direction.opposite())
                    } else {
                        None
                    },
                    exit: None,
                };

                Status::Ongoing
            }
            Cell::Foods(foods_index) => {
                assert_eq!(self.foods.swap_remove(foods_index), next_head);
                if foods_index < self.foods.len() {
                    let position = self.foods[foods_index];
                    *self.board.at_mut(&position) = Cell::Foods(foods_index);
                }

                *self.board.at_mut(&last_head) =
                    if let Cell::Snake { entry, exit: None } = self.board.at(&last_head) {
                        Cell::Snake {
                            entry,
                            exit: Some(direction),
                        }
                    } else {
                        panic!("invariant not snake {:?}", self.board.at(&last_head))
                    };

                self.snake.push_front(next_head);
                *self.board.at_mut(&next_head) = Cell::Snake {
                    entry: Some(direction.opposite()),
                    exit: None,
                };

                let _ = self.add_food();

                if self.foods.is_empty() && self.empty.is_empty() {
                    Status::Over { is_won: true }
                } else {
                    Status::Ongoing
                }
            }
            Cell::Snake { .. } => Status::Over { is_won: false },
        }
    }

    fn get_next_head(&mut self, direction: &Direction) -> Position {
        let position = self.get_head();
        self.board.move_in(position, &direction)
    }

    fn get_head(&self) -> &Position {
        self.snake.front().expect("non empty snake head")
    }

    fn add_food(&mut self) -> Result<(), MaxFoods> {
        if self.empty.is_empty() {
            Err(MaxFoods)
        } else {
            let empty_index = self.rng.gen_range(0..self.empty.len());
            let position = self.empty.swap_remove(empty_index);
            if empty_index < self.empty.len() {
                let position = self.empty[empty_index];
                *self.board.at_mut(&position) = Cell::Empty(empty_index);
            }
            let foods_index = self.foods.len();
            *self.board.at_mut(&position) = Cell::Foods(foods_index);
            self.foods.push(position);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;

    use crate::controller::MockController;
    use crate::data_transfer::Direction;
    use crate::seeder::{MockSeeder, Seeder};

    use super::*;

    const LOOSABLE_BOARD: [[Cell; 3]; 2] = [
        [
            Cell::Snake {
                entry: Some(Direction::Right),
                exit: Some(Direction::Down),
            },
            Cell::Snake {
                entry: Some(Direction::Right),
                exit: Some(Direction::Left),
            },
            Cell::Snake {
                entry: None,
                exit: Some(Direction::Left),
            },
        ],
        [
            Cell::Snake {
                entry: Some(Direction::Up),
                exit: Some(Direction::Right),
            },
            Cell::Snake {
                entry: Some(Direction::Left),
                exit: None,
            },
            Cell::Empty(0),
        ],
    ];

    impl<const N_ROWS: usize, const N_COLS: usize> GameState<N_ROWS, N_COLS> {
        fn assert_is_empty(&self, position: &Position, empty_index: usize) {
            assert_eq!(Cell::Empty(empty_index), self.board.at(position));
            assert_eq!(self.empty[empty_index], *position);
            assert!(self.empty.contains(position));
            assert!(!self.foods.contains(position));
            assert!(!self.snake.contains(position));
        }

        fn assert_is_snake_with_directions(
            &self,
            position: &Position,
            entry: Option<Direction>,
            exit: Option<Direction>,
        ) {
            assert_eq!(self.board.at(position), Cell::Snake { entry, exit });
            assert!(!self.empty.contains(position));
            assert!(!self.foods.contains(position));
            assert!(self.snake.contains(position));
        }

        fn assert_is_foods(&self, position: &Position, foods_index: usize) {
            assert_eq!(self.board.at(position), Cell::Foods(foods_index));
            assert_eq!(self.foods[foods_index], *position);
            assert!(!self.empty.contains(position));
            assert!(self.foods.contains(position));
            assert!(!self.snake.contains(position));
        }
    }

    #[test]
    pub fn from_board() {
        let board = Board::new([[Cell::Snake {
            entry: None,
            exit: None,
        }]]);
        let controller = Box::new(MockController(Direction::Right));
        let rng = ChaCha8Rng::seed_from_u64(0);
        let game_state = GameState::from_board(board, controller, rng);
        assert_eq!(game_state.empty, Vec::new());
        assert_eq!(game_state.snake, VecDeque::from([Position(0, 0)]));
    }

    #[test]
    pub fn get_next_head() {
        let options = Options::<3, 3>::with_seed(1, 0);
        let controller = Box::new(MockController(Direction::Right));
        let mut game_state = options.build(controller).unwrap();
        assert_eq!(game_state.get_next_head(&Direction::Right), Position(1, 2));
    }

    #[test]
    pub fn get_head() {
        let options = Options::<3, 3>::with_seed(1, 0);
        let controller = Box::new(MockController(Direction::Right));
        let game_state = options.build(controller).unwrap();
        assert_eq!(*game_state.get_head(), Position(1, 1));
    }

    #[test]
    fn iterate_turn_empty() {
        let controller = Box::new(MockController(Direction::Right));
        let mut game_state = Options::<3, 3>::with_seed(0, 0).build(controller).unwrap();
        assert_eq!(game_state.iterate_turn(), Status::Ongoing);
        game_state.assert_is_empty(&Position(1, 1), 7);
        game_state.assert_is_snake_with_directions(&Position(1, 2), None, None);
    }

    #[test]
    fn iterate_turn_foods() {
        let new_foods_position = Position(1, 2);
        let controller = Box::new(MockController(Direction::Down));
        let mut game_state = Options::<3, 3>::with_seed(3, 0).build(controller).unwrap();
        game_state.assert_is_empty(&new_foods_position, 4);
        assert_eq!(game_state.iterate_turn(), Status::Ongoing);
        game_state.assert_is_snake_with_directions(&Position(1, 1), None, Some(Direction::Down));
        game_state.assert_is_snake_with_directions(&Position(2, 1), Some(Direction::Up), None);
        game_state.assert_is_foods(&new_foods_position, 2);
    }

    #[test]
    fn iterate_turn_snake_is_won_true() {
        let controller = Box::new(MockController(Direction::Right));
        let mut game_state = Options::<1, 2>::new(1).build(controller).unwrap();
        assert_eq!(game_state.iterate_turn(), Status::Over { is_won: true });
    }

    #[test]
    fn iterate_turn_snake_is_won_false() {
        let board = Board::new(LOOSABLE_BOARD);
        let controller = Box::new(MockController(Direction::Up));
        let rng = MockSeeder(0).get_rng();
        let mut game_state = GameState::from_board(board, controller, rng);
        assert_eq!(game_state.iterate_turn(), Status::Over { is_won: false });
    }
}

impl<const N_ROWS: usize, const N_COLS: usize> Options<N_ROWS, N_COLS> {
    fn get_init_game_state(
        &self,
        board: Board<N_ROWS, N_COLS>,
        controller: Box<dyn Controller>,
    ) -> GameState<N_ROWS, N_COLS> {
        let empty = board.get_empty();
        let foods = board.get_foods();
        let snake = board.get_snake();
        GameState {
            board,
            empty,
            foods,
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
        [Cell::Foods(0), Cell::Empty(1), Cell::Empty(2)],
        [
            Cell::Empty(3),
            Cell::Snake {
                entry: None,
                exit: None,
            },
            Cell::Empty(4),
        ],
        [Cell::Empty(5), Cell::Empty(6), Cell::Empty(0)],
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
        let options = Options::<3, 3>::with_seed(1, 0);
        let controller = Box::new(MockController(Direction::Right));
        let game_state = options.build(controller).unwrap();
        assert_eq!(game_state.board, Board::new(EXPECTED_BOARD));
        assert_eq!(game_state.empty, Vec::from(EXPECTED_EMPTY));
        assert_eq!(game_state.snake, VecDeque::from(EXPECTED_SNAKE));
    }
}
