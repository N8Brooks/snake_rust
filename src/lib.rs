use controller::{Controller, MockController};
use direction::Direction;
use ndarray::Array2;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::fmt::Display;
use std::time::SystemTime;

pub mod controller;
pub mod direction;

pub enum Status {
    Ongoing,
    Over { is_won: bool },
}

/// A board reference to a cell referred to as `[i, j]`.
#[derive(Debug, PartialEq)]
struct Position([usize; 2]);

/// A 1 turn change in a `Position` referred to as `[di, dj]`.
#[derive(Default, Debug, PartialEq)]
struct Velocity([isize; 2]);

impl Velocity {
    /// The default magnitude for a velocity given by a direction.
    const DEFAULT_MAGNITUDE: usize = 1;

    fn is_vertical(&self) -> bool {
        self.0[0] != 0 && self.0[1] == 0
    }

    fn is_moving(&self) -> bool {
        self.0[0] != 0 || self.0[1] != 0
    }

    fn from_direction(direction: &Direction) -> Velocity {
        match direction {
            Direction::Up => Velocity([-(Velocity::DEFAULT_MAGNITUDE as isize), 0]),
            Direction::Left => Velocity([0, -(Velocity::DEFAULT_MAGNITUDE as isize)]),
            Direction::Right => Velocity([0, Velocity::DEFAULT_MAGNITUDE as isize]),
            Direction::Down => Velocity([Velocity::DEFAULT_MAGNITUDE as isize, 0]),
        }
    }
}

#[allow(dead_code)]
enum Cell {
    Snake,
    Empty,
    Food,
}

#[derive(Debug, PartialEq)]
enum CellWithMetadata {
    Snake,
    Empty(usize),
    Food(usize),
}

impl Display for CellWithMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let char = match self {
            CellWithMetadata::Empty(_) => "░░",
            CellWithMetadata::Food(_) => "▒▒",
            CellWithMetadata::Snake => "██",
        };
        write!(f, "{char}")
    }
}

#[derive(Debug)]
pub struct GameIsOver;

#[derive(Debug)]
pub struct InvalidDirection;

#[derive(Debug)]
struct FoodCannotBeAdded;

#[derive(Clone, Debug)]
pub struct Options {
    pub shape: (usize, usize),
    pub n_foods: usize,
    pub seed: Option<u64>,
    pub controller: Box<dyn Controller>,
}

impl Options {
    pub fn build(&self) -> GameState {
        if !self.is_valid_board_size() {
            panic!("insufficient board size");
        }
        let mut game_state = GameState {
            options: self.clone(),
            status: Status::Ongoing,
            velocity: Velocity::default(),
            board: self.get_board(),
            snake: self.get_snake(),
            empty: self.get_empty(),
            foods: self.get_foods(),
            rng: self.get_rng(),
            controller: Box::new(MockController::empty()),
        };
        for _ in 0..self.n_foods {
            game_state.add_food().expect("non-zero empty");
        }
        game_state
    }

    fn is_valid_board_size(&self) -> bool {
        let (n_rows, n_cols) = self.shape;
        let is_valid_n_rows = n_rows >= Velocity::DEFAULT_MAGNITUDE;
        let is_valid_n_cols = n_cols >= Velocity::DEFAULT_MAGNITUDE;
        let n_cells = n_rows * n_cols;
        let n_snake = 1;
        let n_non_empty = n_snake + self.n_foods;
        is_valid_n_rows && is_valid_n_cols && n_cells >= n_non_empty
    }

    fn get_head(&self) -> [usize; 2] {
        let (n_rows, n_cols) = self.shape;
        [n_rows / 2, n_cols / 2]
    }

    fn get_board(&self) -> Array2<CellWithMetadata> {
        let (n_rows, _n_cols) = self.shape;
        let [head_i, head_j] = self.get_head();
        let head_index = head_i * n_rows + head_j;
        Array2::from_shape_fn(self.shape, |(i, j)| {
            let index = i * n_rows + j;
            match index.cmp(&head_index) {
                Ordering::Less => CellWithMetadata::Empty(index),
                Ordering::Equal => CellWithMetadata::Snake,
                Ordering::Greater => CellWithMetadata::Empty(index - 1),
            }
        })
    }

    fn get_empty(&self) -> Vec<Position> {
        self.get_board()
            .indexed_iter()
            .filter(|(_, cell)| matches!(cell, CellWithMetadata::Empty(_)))
            .map(|(index, _)| Position([index.0, index.1]))
            .collect()
    }

    fn get_snake(&self) -> VecDeque<Position> {
        let (n_rows, n_cols) = self.shape;
        let mut snake = VecDeque::with_capacity(n_rows * n_cols);
        snake.push_front(Position(self.get_head()));
        snake
    }

    fn get_foods(&self) -> Vec<Position> {
        Vec::with_capacity(self.n_foods)
    }

    fn get_rng(&self) -> ChaCha8Rng {
        let seed = self
            .seed
            .unwrap_or_else(|| SystemTime::now().elapsed().expect("system time").as_secs());
        ChaCha8Rng::seed_from_u64(seed)
    }
}

impl Default for Options {
    fn default() -> Self {
        Options {
            shape: (20, 20),
            n_foods: 1,
            seed: None,
            controller: Box::new(MockController::empty()),
        }
    }
}

pub struct GameState {
    options: Options,
    status: Status,
    rng: ChaCha8Rng,
    velocity: Velocity,
    board: Array2<CellWithMetadata>,
    snake: VecDeque<Position>,
    empty: Vec<Position>,
    foods: Vec<Position>,
    #[allow(dead_code)]
    controller: Box<dyn Controller>,
}

impl GameState {
    pub fn set_direction(&mut self, direction: Direction) -> Result<(), InvalidDirection> {
        let velocity = Velocity::from_direction(&direction);
        if self.velocity.is_moving() && velocity.is_vertical() == self.velocity.is_vertical() {
            Err(InvalidDirection)
        } else {
            self.velocity = velocity;
            Ok(())
        }
    }

    pub fn iterate_turn(&mut self) -> Result<(), GameIsOver> {
        self.check_status()?;

        if !self.velocity.is_moving() {
            return Ok(());
        }

        let head = self.get_new_snake_head();
        match self.board[head.0] {
            CellWithMetadata::Snake => {
                self.end_game();
                return Ok(());
            }
            CellWithMetadata::Empty(empty_index) => {
                self.update_for_empty(empty_index);
            }
            CellWithMetadata::Food(foods_index) => {
                self.update_for_foods(foods_index);
            }
        }

        Ok(())
    }

    fn check_status(&self) -> Result<(), GameIsOver> {
        match self.status {
            Status::Ongoing => Ok(()),
            Status::Over { .. } => Err(GameIsOver),
        }
    }

    fn get_new_snake_head(&self) -> Position {
        let (n_rows, n_cols) = self.options.shape;
        let Position([i_0, j_0]) = self.get_current_snake_head();
        let Velocity([d_i, d_j]) = self.velocity;
        let i_1 = i_0
            .checked_add_signed(d_i)
            .unwrap_or(n_rows - Velocity::DEFAULT_MAGNITUDE)
            % n_rows;
        let j_1 = j_0
            .checked_add_signed(d_j)
            .unwrap_or(n_cols - Velocity::DEFAULT_MAGNITUDE)
            % n_cols;
        Position([i_1, j_1])
    }

    fn get_current_snake_head(&self) -> &Position {
        self.snake.front().expect("non-zero length snake")
    }

    fn get_current_snake_tail(&mut self) -> Position {
        self.snake.pop_back().expect("non-empty snake")
    }

    fn end_game(&mut self) {
        self.status = Status::Over {
            is_won: self.is_won(),
        };
    }

    fn is_won(&self) -> bool {
        self.empty.is_empty() && self.foods.is_empty()
    }

    fn update_for_empty(&mut self, empty_index: usize) {
        let head = self.get_new_snake_head();
        self.remove_empty(empty_index);
        self.pop_snake();
        self.push_snake(head);
    }

    /// Remove `CellWithMetadata::Empty` and update `food_index` for `board`
    fn remove_empty(&mut self, empty_index: usize) -> Position {
        let position = self.empty.swap_remove(empty_index);
        if empty_index < self.empty.len() {
            let position = &self.empty[empty_index];
            self.board[position.0] = CellWithMetadata::Empty(empty_index);
        }
        position
    }

    /// Remove `CellWithMetadata::Food` and update `food_index` for `board`
    fn remove_foods(&mut self, foods_index: usize) {
        self.foods.swap_remove(foods_index);
        if foods_index < self.foods.len() {
            let position = &self.foods[foods_index];
            self.board[position.0] = CellWithMetadata::Food(foods_index);
        }
    }

    fn pop_snake(&mut self) {
        let tail = self.get_current_snake_tail();
        let empty_index = self.empty.len();
        self.board[tail.0] = CellWithMetadata::Empty(empty_index);
        self.empty.push(tail);
    }

    fn push_snake(&mut self, head: Position) {
        self.board[head.0] = CellWithMetadata::Snake;
        self.snake.push_front(head);
    }

    fn push_foods(&mut self, position: Position) {
        let foods_index = self.foods.len();
        self.board[position.0] = CellWithMetadata::Food(foods_index);
        self.foods.push(position);
    }

    fn update_for_foods(&mut self, foods_index: usize) {
        self.remove_foods(foods_index);
        let _ = self.add_food();
        let head = self.get_new_snake_head();
        self.snake.push_front(head);
    }

    fn add_food(&mut self) -> Result<(), FoodCannotBeAdded> {
        if self.foods.len() >= self.options.n_foods {
            panic!("max foods");
        }

        if self.empty.is_empty() {
            return Err(FoodCannotBeAdded);
        }

        let empty_index = self.rng.gen_range(0..self.empty.len());
        let position = self.remove_empty(empty_index);
        self.push_foods(position);
        Ok(())
    }
}

impl Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = self
            .board
            .outer_iter()
            .map(|row| {
                row.iter()
                    .map(|cell_with_metadata| cell_with_metadata.to_string())
                    .collect::<String>()
                    + "\n"
            })
            .collect::<String>();
        write!(f, "{string}",)
    }
}

#[cfg(test)]
mod velocity_tests {
    use super::*;

    #[test]
    fn is_vertical() {
        assert!(Velocity([1, 0]).is_vertical());
    }

    #[test]
    fn is_not_vertical() {
        assert!(!Velocity([0, 1]).is_vertical());
    }

    #[test]
    fn is_moving() {
        assert!(Velocity([1, 0]).is_moving());
    }

    #[test]
    fn is_not_moving() {
        assert!(!Velocity([0, 0]).is_moving());
    }

    #[test]
    fn from_velocity_up() {
        assert_eq!(
            Velocity::from_direction(&Direction::Up),
            Velocity([-(Velocity::DEFAULT_MAGNITUDE as isize), 0])
        );
    }

    #[test]
    fn from_velocity_right() {
        assert_eq!(
            Velocity::from_direction(&Direction::Right),
            Velocity([0, Velocity::DEFAULT_MAGNITUDE as isize])
        );
    }

    #[test]
    fn from_velocity_left() {
        assert_eq!(
            Velocity::from_direction(&Direction::Left),
            Velocity([0, -(Velocity::DEFAULT_MAGNITUDE as isize)])
        );
    }

    #[test]
    fn from_velocity_down() {
        assert_eq!(
            Velocity::from_direction(&Direction::Down),
            Velocity([Velocity::DEFAULT_MAGNITUDE as isize, 0])
        );
    }
}

#[cfg(test)]
mod cell_with_metadata_tests {
    use super::*;

    #[test]
    fn to_string_empty() {
        let snake = CellWithMetadata::Empty(0);
        assert_eq!(snake.to_string(), "░░")
    }

    #[test]
    fn to_string_food() {
        let snake = CellWithMetadata::Food(0);
        assert_eq!(snake.to_string(), "▒▒")
    }

    #[test]
    fn to_string_snake() {
        let snake = CellWithMetadata::Snake;
        assert_eq!(snake.to_string(), "██")
    }
}

#[cfg(test)]
mod options_tests {
    use super::*;

    fn make_test_options() -> Options {
        Options {
            shape: (3, 3),
            n_foods: 1,
            seed: Some(0),
            controller: Box::new(MockController::empty()),
        }
    }

    #[test]
    fn build() {
        let options = make_test_options();
        let game_state = options.build();
        assert!(matches!(game_state.status, Status::Ongoing));
        assert_eq!(game_state.velocity, Velocity::default());
        assert_eq!(
            game_state.board,
            Array2::from_shape_vec(
                options.shape,
                vec![
                    CellWithMetadata::Food(0),
                    CellWithMetadata::Empty(1),
                    CellWithMetadata::Empty(2),
                    CellWithMetadata::Empty(3),
                    CellWithMetadata::Snake,
                    CellWithMetadata::Empty(4),
                    CellWithMetadata::Empty(5),
                    CellWithMetadata::Empty(6),
                    CellWithMetadata::Empty(0),
                ],
            )
            .unwrap()
        );
    }

    #[test]
    #[should_panic]
    fn insufficient_board_size() {
        Options {
            shape: (1, 1),
            n_foods: 1,
            seed: None,
            controller: Box::new(MockController::empty()),
        }
        .build();
    }

    #[test]
    fn is_valid_board_size() {
        assert_eq!(make_test_options().is_valid_board_size(), true);
    }

    #[test]
    fn get_head() {
        assert_eq!(make_test_options().get_head(), [1, 1]);
    }

    #[test]
    fn get_board() {
        let options = make_test_options();
        let actual = options.get_board();
        let expected = Array2::from_shape_vec(
            options.shape,
            vec![
                CellWithMetadata::Empty(0),
                CellWithMetadata::Empty(1),
                CellWithMetadata::Empty(2),
                CellWithMetadata::Empty(3),
                CellWithMetadata::Snake,
                CellWithMetadata::Empty(4),
                CellWithMetadata::Empty(5),
                CellWithMetadata::Empty(6),
                CellWithMetadata::Empty(7),
            ],
        )
        .unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn get_empty() {
        let actual = make_test_options().get_empty();
        let expected = vec![
            Position([0, 0]),
            Position([0, 1]),
            Position([0, 2]),
            Position([1, 0]),
            Position([1, 2]),
            Position([2, 0]),
            Position([2, 1]),
            Position([2, 2]),
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn get_snake() {
        let options = make_test_options();
        let actual = options.get_snake();
        let expected = vec![Position(options.get_head())];
        assert_eq!(actual, expected);
    }

    #[test]
    fn get_foods() {
        assert_eq!(make_test_options().get_foods(), Vec::new());
    }

    #[test]
    fn get_rng_some() {
        let options = make_test_options();
        let actual = options.get_rng();
        let expected = ChaCha8Rng::seed_from_u64(options.seed.unwrap());
        assert_eq!(actual, expected);
    }

    #[test]
    fn get_rng_none() {
        // This just asserts it doesn't `panic!`
        Options {
            shape: (3, 3),
            n_foods: 1,
            seed: None,
            controller: Box::new(MockController::empty()),
        }
        .get_rng();
    }
}

#[cfg(test)]
mod game_state_tests {
    use super::*;

    #[test]
    fn set_direction() {
        let mut game_state = Options {
            shape: (3, 3),
            n_foods: 1,
            seed: Some(0),
            controller: Box::new(MockController::empty()),
        }
        .build();
        assert!(game_state.set_direction(Direction::Up).is_ok());
        assert!(!game_state.set_direction(Direction::Up).is_ok());
    }

    #[test]
    fn iterate_turn_empty() {
        let mut game_state = Options {
            shape: (3, 3),
            n_foods: 0,
            seed: Some(0),
            controller: Box::new(MockController::empty()),
        }
        .build();
        assert!(game_state.set_direction(Direction::Right).is_ok());
        assert!(game_state.iterate_turn().is_ok());
    }

    #[test]
    fn iterate_turn_food() {
        let mut game_state = Options {
            shape: (3, 3),
            n_foods: 8,
            seed: Some(0),
            controller: Box::new(MockController::empty()),
        }
        .build();
        assert!(game_state.set_direction(Direction::Right).is_ok());
        assert!(game_state.iterate_turn().is_ok());
    }

    #[test]
    fn iterate_turn_snake() {
        let mut game_state = Options {
            shape: (3, 3),
            n_foods: 8,
            seed: Some(0),
            controller: Box::new(MockController::empty()),
        }
        .build();
        assert!(game_state.set_direction(Direction::Right).is_ok());
        assert!(game_state.iterate_turn().is_ok());
        assert!(game_state.iterate_turn().is_ok());
        assert!(game_state.iterate_turn().is_ok());
        assert!(matches!(game_state.iterate_turn(), Err(GameIsOver)));
    }
}
