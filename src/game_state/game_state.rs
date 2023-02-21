use crate::controller::Controller;
use crate::data_transfer_objects as dto;
use crate::view::View;
use rand::Rng;
use rand_chacha::ChaCha8Rng;

use super::{
    options::Options,
    state::{board::Board, state::State, *},
};

// TODO: replace `view` with subscription model
// TODO: some testing for `iterate_turn` is redundant
// TODO: move `move_in` to position with `State` generic

#[derive(Debug)]
pub struct MaxFoods;

#[derive(Debug)]
pub struct GameState<'a, const N_ROWS: usize, const N_COLS: usize> {
    state: State<N_ROWS, N_COLS>,
    controller: &'a mut dyn Controller,
    view: &'a mut dyn View,
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
        GameState {
            state: State::new(board, rng),
            controller,
            view,
        }
    }

    fn get_next_head(&self, direction: &Direction) -> Position {
        self.state.board.move_in(self.get_last_head(), direction)
    }

    pub fn iterate_turn(&mut self) -> dto::Status {
        let direction = self.controller.get_direction();
        let next_head = self.get_next_head(&direction);
        match self.state.board.at(&next_head) {
            Cell::Empty(_) => {
                self.remove_last_tail();
                let entry = if self.state.snake.is_empty() {
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
        if self.state.foods.is_empty() && self.state.empty.is_empty() {
            dto::Status::Over { is_won: true }
        } else {
            dto::Status::Ongoing
        }
    }

    fn remove_last_tail(&mut self) {
        let last_tail = self
            .state
            .snake
            .pop_back()
            .expect("non empty snake last tail");
        let old = dto::Cell::from(self.state.board.at(&last_tail));
        *self.state.board.at_mut(&last_tail) = if let Cell::Snake(Path {
            entry: None,
            exit: _,
        }) = self.state.board.at(&last_tail)
        {
            Cell::Empty(self.state.empty.len())
        } else {
            panic!(
                "invariant invalid snake {:?}",
                self.state.board.at(&last_tail)
            )
        };
        self.state.empty.push(last_tail);
        self.view
            .swap_cell(&last_tail.into(), old, dto::Cell::Empty);
    }

    fn get_next_tail(&self) -> &Position {
        self.state.snake.back().expect("non empty snake next tail")
    }

    fn update_next_tail(&mut self) {
        let next_tail = *self.get_next_tail();
        let old = dto::Cell::from(self.state.board.at(&next_tail));
        *self.state.board.at_mut(&next_tail) =
            if let Cell::Snake(path) = self.state.board.at(&next_tail) {
                Cell::Snake(Path {
                    entry: None,
                    exit: path.exit,
                })
            } else {
                panic!("invariant not snake {:?}", self.state.board.at(&next_tail))
            };
        let new = dto::Cell::from(self.state.board.at(&next_tail));
        self.view.swap_cell(&next_tail.into(), old, new);
    }

    fn insert_snake_head(&mut self, next_head: Position, entry: Option<Direction>) {
        let old = dto::Cell::from(self.state.board.at(&next_head));
        match self.state.board.at(&next_head) {
            Cell::Empty(empty_index) => self.remove_empty(&next_head, empty_index),
            Cell::Foods(foods_index) => self.remove_foods(&next_head, foods_index),
            snake => panic!("unexpected snake {snake:?}"),
        }
        *self.state.board.at_mut(&next_head) = Cell::Snake(Path { entry, exit: None });
        self.state.snake.push_front(next_head);
        let new = dto::Cell::from(self.state.board.at(&next_head));
        self.view.swap_cell(&next_head.into(), old, new);
    }

    fn remove_empty(&mut self, next_head: &Position, empty_index: usize) {
        assert_eq!(&self.state.empty.swap_remove(empty_index), next_head);
        if empty_index < self.state.empty.len() {
            let position = self.state.empty[empty_index];
            *self.state.board.at_mut(&position) = Cell::Empty(empty_index);
        }
    }

    fn remove_foods(&mut self, next_head: &Position, foods_index: usize) {
        assert_eq!(&self.state.foods.swap_remove(foods_index), next_head);
        if foods_index < self.state.foods.len() {
            let position = self.state.foods[foods_index];
            *self.state.board.at_mut(&position) = Cell::Foods(foods_index);
        }
    }

    fn get_last_head(&self) -> &Position {
        self.state.snake.front().expect("non empty snake last head")
    }

    fn update_last_head(&mut self, direction: &Direction) {
        let last_head = *self.get_last_head();
        let old = dto::Cell::from(self.state.board.at(&last_head));
        *self.state.board.at_mut(&last_head) =
            if let Cell::Snake(Path { entry, exit: None }) = self.state.board.at(&last_head) {
                Cell::Snake(Path {
                    entry,
                    exit: Some(*direction),
                })
            } else {
                panic!(
                    "invariant invalid snake {:?}",
                    self.state.board.at(&last_head)
                )
            };
        let new = dto::Cell::from(self.state.board.at(&last_head));
        self.view.swap_cell(&last_head.into(), old, new);
    }

    fn insert_food(&mut self) -> Result<(), MaxFoods> {
        if self.state.empty.is_empty() {
            Err(MaxFoods)
        } else {
            let empty_index = self.state.rng.gen_range(0..self.state.empty.len());
            let position = self.state.empty.swap_remove(empty_index);
            if empty_index < self.state.empty.len() {
                let position = self.state.empty[empty_index];
                *self.state.board.at_mut(&position) = Cell::Empty(empty_index);
            }
            let foods_index = self.state.foods.len();
            *self.state.board.at_mut(&position) = Cell::Foods(foods_index);
            self.state.foods.push(position);
            self.view
                .swap_cell(&position.into(), dto::Cell::Empty, dto::Cell::Foods);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use rand::SeedableRng;

    use crate::{
        controller::mock_controller::MockController,
        seeder::{MockSeeder, Seeder},
        view::MockView,
    };

    use super::*;

    impl<'a, const N_ROWS: usize, const N_COLS: usize> GameState<'a, N_ROWS, N_COLS> {
        fn assert_is_empty(&self, position: &Position, empty_index: usize) {
            assert_eq!(Cell::Empty(empty_index), self.state.board.at(position));
            assert_eq!(self.state.empty[empty_index], *position);
            assert!(self.state.empty.contains(position));
            assert!(!self.state.foods.contains(position));
            assert!(!self.state.snake.contains(position));
        }

        fn assert_is_snake_with_path(&self, position: &Position, path: Path) {
            assert_eq!(self.state.board.at(position), Cell::Snake(path));
            assert!(!self.state.empty.contains(position));
            assert!(!self.state.foods.contains(position));
            assert!(self.state.snake.contains(position));
        }

        fn assert_is_foods(&self, position: &Position, foods_index: usize) {
            assert_eq!(self.state.board.at(position), Cell::Foods(foods_index));
            assert_eq!(self.state.foods[foods_index], *position);
            assert!(!self.state.empty.contains(position));
            assert!(self.state.foods.contains(position));
            assert!(!self.state.snake.contains(position));
        }
    }

    #[test]
    pub fn from_board() {
        let board = Board::new([[Cell::Snake(Path {
            entry: None,
            exit: None,
        })]]);
        let mut controller = MockController(Direction::Right);
        let mut view = MockView::default();
        let rng = ChaCha8Rng::seed_from_u64(0);
        let game_state = GameState::from_board(board, &mut controller, &mut view, rng);
        assert_eq!(game_state.state.empty, Vec::new());
        assert_eq!(game_state.state.snake, VecDeque::from([Position(0, 0)]));
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
        game_state.assert_is_snake_with_path(
            &Position(1, 2),
            Path {
                entry: None,
                exit: None,
            },
        );
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
        game_state.assert_is_snake_with_path(
            &Position(1, 1),
            Path {
                entry: None,
                exit: Some(Direction::Down),
            },
        );
        game_state.assert_is_snake_with_path(
            &Position(2, 1),
            Path {
                entry: Some(Direction::Up),
                exit: None,
            },
        );
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

    const BOARD: [[Cell; 3]; 2] = [
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

    fn setup_loosable_board<'a>(
        controller: &'a mut dyn Controller,
        view: &'a mut dyn View,
    ) -> GameState<'a, 2, 3> {
        let board = Board::new(BOARD);
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
        let old = dto::Cell::Snake(Path {
            entry: None,
            exit: Some(controller.0.opposite()),
        });
        assert_eq!(view.0, &[(position.into(), old, dto::Cell::Empty)])
    }

    #[test]
    fn update_next_tail() {
        let position = Position(0, 1);
        let mut controller = MockController(Direction::Right);
        let mut view = MockView::default();
        let mut game_state = setup_loosable_board(&mut controller, &mut view);
        game_state.remove_last_tail();
        game_state.update_next_tail();
        let new_path = Path {
            entry: None,
            exit: Some(Direction::Left),
        };
        game_state.assert_is_snake_with_path(&position, new_path);
        let old = dto::Cell::Snake(Path {
            entry: Some(Direction::Right),
            exit: Some(Direction::Left),
        });
        let new = dto::Cell::Snake(new_path);
        assert_eq!(view.0.last().unwrap(), &(position.into(), old, new));
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
        game_state.assert_is_snake_with_path(
            &position,
            Path {
                entry: Some(Direction::Left),
                exit: None,
            },
        );
        let new = dto::Cell::Snake(Path { entry, exit: None });
        assert_eq!(view.0, &[(position.into(), dto::Cell::Empty, new)]);
    }

    #[test]
    fn update_last_head() {
        let position = Position(1, 1);
        let direction = Direction::Right;
        let mut controller = MockController(direction);
        let mut view = MockView::default();
        let mut game_state = setup_loosable_board(&mut controller, &mut view);
        game_state.update_last_head(&Direction::Right);
        let new_path = Path {
            entry: Some(direction.opposite()),
            exit: Some(direction),
        };
        game_state.assert_is_snake_with_path(&position, new_path);
        let old = dto::Cell::Snake(Path {
            entry: Some(direction.opposite()),
            exit: None,
        });
        let new = dto::Cell::Snake(new_path);
        assert_eq!(view.0, &[(position.into(), old, new)]);
    }

    #[test]
    fn insert_food() {
        let position = Position(1, 2);
        let mut controller = MockController(Direction::Right);
        let mut view = MockView::default();
        let mut game_state = setup_loosable_board(&mut controller, &mut view);
        assert!(game_state.insert_food().is_ok());
        game_state.assert_is_foods(&position, 0);
        assert_eq!(
            view.0,
            &[(position.into(), dto::Cell::Empty, dto::Cell::Foods)]
        );
    }
}

impl<const N_ROWS: usize, const N_COLS: usize> Options<N_ROWS, N_COLS> {
    fn get_init_game_state<'a>(
        &self,
        board: Board<N_ROWS, N_COLS>,
        controller: &'a mut dyn Controller,
        view: &'a mut dyn View,
    ) -> GameState<'a, N_ROWS, N_COLS> {
        let state = State::new(board, self.seeder.get_rng());
        GameState {
            state,
            controller,
            view,
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
            Cell::Snake(Path {
                entry: None,
                exit: None,
            }),
            Cell::Empty(4),
        ],
        [Cell::Empty(5), Cell::Empty(6), Cell::Empty(0)],
    ];

    #[test]
    fn build_with_valid() {
        // TODO: this test is tightly coupled
        let options = Options::<3, 3>::with_seed(1, 0);
        let mut controller = MockController(Direction::Right);
        let mut view = MockView::default();
        let game_state = options.build(&mut controller, &mut view).unwrap();
        let board = Board::new(EXPECTED_BOARD);
        assert_eq!(game_state.state.board, board);
    }
}
