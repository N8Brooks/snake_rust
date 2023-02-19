use std::collections::VecDeque;

use crate::controller::Controller;
use crate::data_transfer_objects as dto;
use crate::value_objects::*;
use crate::view::View;
use rand::Rng;
use rand_chacha::ChaCha8Rng;

use super::board::Board;
use super::Options;

// TODO: make a `SnakeDirections` struct
// TODO: add other structures to `Board`?
// TODO: replace `view` with subscription model
// TODO: some testing for `iterate_turn` is redundant
// TODO: move `move_in` to position with `Board` generic

#[derive(Debug)]
pub struct MaxFoods;

#[derive(Debug)]
pub struct GameState<'a, const N_ROWS: usize, const N_COLS: usize> {
    board: Board<N_ROWS, N_COLS>,
    empty: Vec<Position>,
    foods: Vec<Position>,
    snake: VecDeque<Position>,
    controller: &'a mut dyn Controller,
    view: &'a mut dyn View,
    rng: ChaCha8Rng,
}

impl<'a, const N_ROWS: usize, const N_COLS: usize> GameState<'a, N_ROWS, N_COLS> {
    pub fn from_options(
        options: &Options<N_ROWS, N_COLS>,
        controller: &'a mut dyn Controller,
        view: &'a mut dyn View,
    ) -> GameState<'a, N_ROWS, N_COLS> {
        let board = Board::<N_ROWS, N_COLS>::default();
        let mut game_state = options.get_init_game_state(board, controller, view);
        options.add_foods(&mut game_state);
        game_state
    }

    /// This builds a `GameState` from a board without checking for invariants
    pub fn from_board(
        board: Board<N_ROWS, N_COLS>,
        controller: &'a mut dyn Controller,
        view: &'a mut dyn View,
        rng: ChaCha8Rng,
    ) -> GameState<'a, N_ROWS, N_COLS> {
        let empty = board.get_empty();
        let foods = board.get_foods();
        let snake = board.get_snake();
        GameState {
            board,
            empty,
            foods,
            snake,
            controller,
            view,
            rng,
        }
    }

    fn get_next_head(&self, direction: &Direction) -> Position {
        self.board.move_in(self.get_last_head(), direction)
    }

    pub fn iterate_turn(&mut self) -> dto::Status {
        let direction = self.controller.get_direction();
        let next_head = self.get_next_head(&direction);
        match self.board.at(&next_head) {
            Cell::Empty(_) => {
                self.remove_last_tail();
                let entry = if self.snake.is_empty() {
                    None
                } else {
                    self.update_next_tail();
                    self.update_last_head(&direction);
                    Some(direction.opposite())
                };
                self.insert_snake_head(next_head, entry);
                dto::Status::Ongoing
            }
            Cell::Foods(_) => {
                self.update_last_head(&direction);
                self.insert_snake_head(next_head, Some(direction.opposite()));
                let _ = self.insert_food();
                self.check_is_won_status()
            }
            Cell::Snake { .. } => dto::Status::Over { is_won: false },
        }
    }

    fn check_is_won_status(&self) -> dto::Status {
        if self.foods.is_empty() && self.empty.is_empty() {
            dto::Status::Over { is_won: true }
        } else {
            dto::Status::Ongoing
        }
    }

    fn remove_last_tail(&mut self) {
        let last_tail = self.snake.pop_back().expect("non empty snake last tail");
        let old = self.board.at(&last_tail).as_dto();
        *self.board.at_mut(&last_tail) =
            if let Cell::Snake { entry: None, .. } = self.board.at(&last_tail) {
                Cell::Empty(self.empty.len())
            } else {
                panic!("invariant not snake {:?}", self.board.at(&last_tail))
            };
        self.empty.push(last_tail);
        self.view.swap_cell(&last_tail, old, dto::Cell::Empty);
    }

    fn get_next_tail(&self) -> &Position {
        self.snake.back().expect("non empty snake next tail")
    }

    fn update_next_tail(&mut self) {
        let next_tail = *self.get_next_tail();
        let old = self.board.at(&next_tail).as_dto();
        *self.board.at_mut(&next_tail) =
            if let Cell::Snake { entry: _, exit } = self.board.at(&next_tail) {
                Cell::Snake { entry: None, exit }
            } else {
                panic!("invariant not snake {:?}", self.board.at(&next_tail))
            };
        let new = self.board.at(&next_tail).as_dto();
        self.view.swap_cell(&next_tail, old, new);
    }

    fn insert_snake_head(&mut self, next_head: Position, entry: Option<Direction>) {
        let old = self.board.at(&next_head).as_dto();
        match self.board.at(&next_head) {
            Cell::Empty(empty_index) => self.remove_empty(&next_head, empty_index),
            Cell::Foods(foods_index) => self.remove_foods(&next_head, foods_index),
            snake => panic!("unexpected snake {snake:?}"),
        }
        *self.board.at_mut(&next_head) = Cell::Snake { entry, exit: None };
        self.snake.push_front(next_head);
        let new = self.board.at(&next_head).as_dto();
        self.view.swap_cell(&next_head, old, new);
    }

    fn remove_empty(&mut self, next_head: &Position, empty_index: usize) {
        assert_eq!(&self.empty.swap_remove(empty_index), next_head);
        if empty_index < self.empty.len() {
            let position = self.empty[empty_index];
            *self.board.at_mut(&position) = Cell::Empty(empty_index);
        }
    }

    fn remove_foods(&mut self, next_head: &Position, foods_index: usize) {
        assert_eq!(&self.foods.swap_remove(foods_index), next_head);
        if foods_index < self.foods.len() {
            let position = self.foods[foods_index];
            *self.board.at_mut(&position) = Cell::Foods(foods_index);
        }
    }

    fn get_last_head(&self) -> &Position {
        self.snake.front().expect("non empty snake last head")
    }

    fn update_last_head(&mut self, direction: &Direction) {
        let last_head = *self.get_last_head();
        let old = self.board.at(&last_head).as_dto();
        *self.board.at_mut(&last_head) =
            if let Cell::Snake { entry, exit: None } = self.board.at(&last_head) {
                Cell::Snake {
                    entry,
                    exit: Some(*direction),
                }
            } else {
                panic!("invariant not snake {:?}", self.board.at(&last_head))
            };
        let new = self.board.at(&last_head).as_dto();
        self.view.swap_cell(&last_head, old, new);
    }

    fn insert_food(&mut self) -> Result<(), MaxFoods> {
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
            self.view
                .swap_cell(&position, dto::Cell::Empty, dto::Cell::Foods);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;

    use crate::{
        controller::mock_controller::MockController,
        data_transfer_objects as dto,
        seeder::{MockSeeder, Seeder},
        view::MockView,
    };

    use super::*;

    impl<'a, const N_ROWS: usize, const N_COLS: usize> GameState<'a, N_ROWS, N_COLS> {
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
        let mut controller = MockController(Direction::Right);
        let mut view = MockView::default();
        let rng = ChaCha8Rng::seed_from_u64(0);
        let game_state = GameState::from_board(board, &mut controller, &mut view, rng);
        assert_eq!(game_state.empty, Vec::new());
        assert_eq!(game_state.snake, VecDeque::from([Position(0, 0)]));
    }

    #[test]
    pub fn get_next_head() {
        let options = Options::<3, 3>::with_seed(1, 0);
        let mut controller = MockController(Direction::Right);
        let mut view = MockView::default();
        let game_state = options.build(&mut controller, &mut view).unwrap();
        assert_eq!(game_state.get_next_head(&Direction::Right), Position(1, 2));
    }

    #[test]
    pub fn get_last_head() {
        let options = Options::<3, 3>::with_seed(1, 0);
        let mut controller = MockController(Direction::Right);
        let mut view = MockView::default();
        let game_state = options.build(&mut controller, &mut view).unwrap();
        assert_eq!(*game_state.get_last_head(), Position(1, 1));
    }

    #[test]
    fn iterate_turn_empty() {
        let mut controller = MockController(Direction::Right);
        let mut view = MockView::default();
        let mut game_state = Options::<3, 3>::with_seed(0, 0)
            .build(&mut controller, &mut view)
            .unwrap();
        assert_eq!(game_state.iterate_turn(), dto::Status::Ongoing);
        game_state.assert_is_empty(&Position(1, 1), 4);
        game_state.assert_is_snake_with_directions(&Position(1, 2), None, None);
    }

    #[test]
    fn iterate_turn_foods() {
        let new_foods_position = Position(1, 2);
        let mut controller = MockController(Direction::Down);
        let mut view = MockView::default();
        let mut game_state = Options::<3, 3>::with_seed(3, 0)
            .build(&mut controller, &mut view)
            .unwrap();
        game_state.assert_is_empty(&new_foods_position, 4);
        assert_eq!(game_state.iterate_turn(), dto::Status::Ongoing);
        game_state.assert_is_snake_with_directions(&Position(1, 1), None, Some(Direction::Down));
        game_state.assert_is_snake_with_directions(&Position(2, 1), Some(Direction::Up), None);
        game_state.assert_is_foods(&new_foods_position, 2);
    }

    #[test]
    fn iterate_turn_snake_is_won_true() {
        let mut controller = MockController(Direction::Right);
        let mut view = MockView::default();
        let mut game_state = Options::<1, 2>::new(1)
            .build(&mut controller, &mut view)
            .unwrap();
        assert_eq!(
            game_state.iterate_turn(),
            dto::Status::Over { is_won: true }
        );
    }

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

    fn setup_loosable_board<'a>(
        controller: &'a mut dyn Controller,
        view: &'a mut dyn View,
    ) -> GameState<'a, 2, 3> {
        let board = Board::new(LOOSABLE_BOARD);
        let rng = MockSeeder(0).get_rng();
        GameState::from_board(board, controller, view, rng)
    }

    #[test]
    fn iterate_turn_snake_is_won_false() {
        let mut controller = MockController(Direction::Up);
        let mut view = MockView::default();
        let mut game_state = setup_loosable_board(&mut controller, &mut view);
        assert_eq!(
            game_state.iterate_turn(),
            dto::Status::Over { is_won: false }
        );
    }

    #[test]
    fn remove_last_tail() {
        let position = Position(0, 2);
        let mut controller = MockController(Direction::Right);
        let mut view = MockView::default();
        let mut game_state = setup_loosable_board(&mut controller, &mut view);
        game_state.remove_last_tail();
        game_state.assert_is_empty(&position, 1);
        let old = dto::Cell::Snake {
            entry: None,
            exit: Some(controller.0.opposite()),
        };
        assert_eq!(view.0, &[(position, old, dto::Cell::Empty)])
    }

    #[test]
    fn update_next_tail() {
        let position = Position(0, 1);
        let mut controller = MockController(Direction::Right);
        let mut view = MockView::default();
        let mut game_state = setup_loosable_board(&mut controller, &mut view);
        game_state.remove_last_tail();
        game_state.update_next_tail();
        game_state.assert_is_snake_with_directions(&position, None, Some(Direction::Left));
        let old = dto::Cell::Snake {
            entry: Some(Direction::Right),
            exit: Some(Direction::Left),
        };
        let new = dto::Cell::Snake {
            entry: None,
            exit: Some(Direction::Left),
        };
        assert_eq!(view.0.last().unwrap(), &(position, old, new));
    }

    #[test]
    fn insert_snake_head() {
        let position = Position(1, 2);
        let mut controller = MockController(Direction::Right);
        let mut view = MockView::default();
        let mut game_state = setup_loosable_board(&mut controller, &mut view);
        let next_head = game_state.get_next_head(&Direction::Right);
        let entry = Some(Direction::Left);
        game_state.insert_snake_head(next_head, entry);
        game_state.assert_is_snake_with_directions(&position, Some(Direction::Left), None);
        let new = dto::Cell::Snake { entry, exit: None };
        assert_eq!(view.0, &[(position, dto::Cell::Empty, new)]);
    }

    #[test]
    fn update_last_head() {
        let position = Position(1, 1);
        let direction = Direction::Right;
        let mut controller = MockController(direction);
        let mut view = MockView::default();
        let mut game_state = setup_loosable_board(&mut controller, &mut view);
        game_state.update_last_head(&Direction::Right);
        game_state.assert_is_snake_with_directions(
            &position,
            Some(direction.opposite()),
            Some(direction),
        );
        let old = dto::Cell::Snake {
            entry: Some(direction.opposite()),
            exit: None,
        };
        let new = dto::Cell::Snake {
            entry: Some(direction.opposite()),
            exit: Some(direction),
        };
        assert_eq!(view.0, &[(Position(1, 1), old, new)]);
    }

    #[test]
    fn insert_food() {
        let position = Position(1, 2);
        let mut controller = MockController(Direction::Right);
        let mut view = MockView::default();
        let mut game_state = setup_loosable_board(&mut controller, &mut view);
        assert!(game_state.insert_food().is_ok());
        game_state.assert_is_foods(&position, 0);
        assert_eq!(view.0, &[(position, dto::Cell::Empty, dto::Cell::Foods)]);
    }
}

impl<const N_ROWS: usize, const N_COLS: usize> Options<N_ROWS, N_COLS> {
    fn get_init_game_state<'a>(
        &self,
        board: Board<N_ROWS, N_COLS>,
        controller: &'a mut dyn Controller,
        view: &'a mut dyn View,
    ) -> GameState<'a, N_ROWS, N_COLS> {
        let empty = board.get_empty();
        let foods = board.get_foods();
        let snake = board.get_snake();
        GameState {
            board,
            empty,
            foods,
            snake,
            controller,
            view,
            rng: self.seeder.get_rng(),
        }
    }

    fn add_foods(&self, game_state: &mut GameState<N_ROWS, N_COLS>) {
        for _ in 0..self.n_foods {
            game_state.insert_food().expect("room for foods");
        }
    }
}

#[cfg(test)]
mod options_tests {
    use super::*;
    use crate::controller::mock_controller::MockController;
    use crate::view::MockView;

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
        let mut controller = MockController(Direction::Right);
        let mut view = MockView::default();
        let game_state = options.build(&mut controller, &mut view).unwrap();
        assert_eq!(game_state.board, Board::new(EXPECTED_BOARD));
        assert_eq!(game_state.empty, Vec::from(EXPECTED_EMPTY));
        assert_eq!(game_state.snake, VecDeque::from(EXPECTED_SNAKE));
    }
}
