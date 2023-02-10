use std::collections::VecDeque;

use crate::controller::Controller;
use crate::data_transfer::{Cell, Status};
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
        let head = self.get_next_head();
        match self.board.at(&head) {
            Cell::Empty(_empty_index) => todo!(),
            Cell::Foods(_foods_index) => todo!(),
            Cell::Snake { .. } => Status::Over { is_won: false },
        }
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
            let empty_index = self.rng.gen_range(0..self.empty.len());
            let position = self.empty.swap_remove(empty_index);
            if empty_index != self.empty.len() {
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
        fn is_empty(&self, position: &Position, empty_index: usize) -> bool {
            self.board.at(position) == Cell::Empty(empty_index)
                && self.empty[empty_index] == *position
                && self.empty.contains(position)
                && !self.foods.contains(position)
                && !self.snake.contains(position)
        }

        fn is_snake_with_directions(
            &self,
            position: &Position,
            entry: Option<Direction>,
            exit: Option<Direction>,
        ) -> bool {
            self.board.at(position) == Cell::Snake { entry, exit }
                && !self.empty.contains(position)
                && !self.foods.contains(position)
                && self.snake.contains(position)
        }

        fn is_foods(&self, position: &Position, foods_index: usize) -> bool {
            self.board.at(position) == Cell::Empty(foods_index)
                && self.empty[foods_index] == *position
                && !self.empty.contains(position)
                && self.foods.contains(position)
                && !self.snake.contains(position)
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
        let options = Options::<3, 3>::with_mock_seeder(1, 0);
        let controller = Box::new(MockController(Direction::Right));
        let mut game_state = options.build(controller).unwrap();
        assert_eq!(game_state.get_next_head(), Position(1, 2));
    }

    #[test]
    pub fn get_head() {
        let options = Options::<3, 3>::with_mock_seeder(1, 0);
        let controller = Box::new(MockController(Direction::Right));
        let game_state = options.build(controller).unwrap();
        assert_eq!(*game_state.get_head(), Position(1, 1));
    }

    #[test]
    fn iterate_turn_empty() {
        let controller = Box::new(MockController(Direction::Right));
        let mut game_state = Options::<3, 3>::new(0).build(controller).unwrap();
        assert_eq!(game_state.iterate_turn(), Status::Ongoing);
        assert!(game_state.is_empty(&Position(1, 1), 7));
        assert!(game_state.is_snake_with_directions(&Position(1, 2), None, None));
    }

    #[test]
    fn iterate_turn_foods() {
        let controller = Box::new(MockController(Direction::Down));
        let mut game_state = Options::<3, 3>::new(3).build(controller).unwrap();
        let new_foods_position = Position(0, 1);
        assert!(game_state.is_empty(&new_foods_position, 7));
        assert_eq!(game_state.iterate_turn(), Status::Ongoing);
        assert!(game_state.is_snake_with_directions(&Position(1, 1), None, Some(Direction::Down)));
        assert!(game_state.is_snake_with_directions(&Position(1, 2), Some(Direction::Up), None));
        assert!(game_state.is_foods(&new_foods_position, 7));
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
        let options = Options::<3, 3>::with_mock_seeder(1, 0);
        let controller = Box::new(MockController(Direction::Right));
        let game_state = options.build(controller).unwrap();
        assert_eq!(game_state.board, Board::new(EXPECTED_BOARD));
        assert_eq!(game_state.empty, Vec::from(EXPECTED_EMPTY));
        assert_eq!(game_state.snake, VecDeque::from(EXPECTED_SNAKE));
    }
}
