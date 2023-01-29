use ndarray::Array2;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::fmt::Display;
use std::time::SystemTime;

#[derive(Clone, Debug, PartialEq)]
pub struct Options {
    pub shape: (usize, usize),
    pub n_foods: usize,
    pub seed: Option<u64>,
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
        };
        for _ in 0..self.n_foods {
            game_state.add_food().expect("non-zero empty");
        }
        game_state
    }

    fn is_valid_board_size(&self) -> bool {
        let (n_rows, n_cols) = self.shape;
        let is_valid_n_rows = n_rows >= Direction::DEFAULT_MAGNITUDE;
        let is_valid_n_cols = n_cols >= Direction::DEFAULT_MAGNITUDE;
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
        }
    }
}

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
    fn is_vertical(&self) -> bool {
        self.0[0] != 0 && self.0[1] == 0
    }

    fn is_moving(&self) -> bool {
        self.0[0] != 0 || self.0[1] != 0
    }
}

#[derive(Clone)]
pub enum Direction {
    Up,
    Left,
    Right,
    Down,
}

impl Direction {
    /// The default magnitude for a velocity given by a direction.
    const DEFAULT_MAGNITUDE: usize = 1;

    fn as_velocity(&self) -> Velocity {
        match self {
            Direction::Up => Velocity([-(Direction::DEFAULT_MAGNITUDE as isize), 0]),
            Direction::Left => Velocity([0, -(Direction::DEFAULT_MAGNITUDE as isize)]),
            Direction::Right => Velocity([0, Direction::DEFAULT_MAGNITUDE as isize]),
            Direction::Down => Velocity([Direction::DEFAULT_MAGNITUDE as isize, 0]),
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
            CellWithMetadata::Snake => 'O',
            CellWithMetadata::Empty(_) => ' ',
            CellWithMetadata::Food(_) => 'X',
        };
        write!(f, "{char}")
    }
}

pub enum Command {
    SetDirection(Direction),
    IterateTurn,
}

pub enum Error {
    InvalidDirection,
    GameIsOver,
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
}

#[derive(Debug)]
struct FoodCannotBeAdded;

impl GameState {
    fn check_status(&self) -> Result<(), Error> {
        match self.status {
            Status::Ongoing => Ok(()),
            Status::Over { .. } => Err(Error::GameIsOver),
        }
    }

    pub fn set_direction(&mut self, direction: Direction) -> Result<(), Error> {
        self.check_status()?;
        let velocity = direction.as_velocity();
        if velocity.is_vertical() == self.velocity.is_vertical() {
            Err(Error::InvalidDirection)
        } else {
            self.velocity = velocity;
            Ok(())
        }
    }

    pub fn iterate_turn(&mut self) -> Result<(), Error> {
        self.check_status()?;
        if !self.velocity.is_moving() {
            return Ok(());
        }
        let head = self.compute_head();
        match self.board[head.0] {
            CellWithMetadata::Snake => {
                self.status = Status::Over {
                    is_won: self.empty.is_empty() && self.foods.is_empty(),
                };
                return Ok(());
            }
            CellWithMetadata::Empty(empty_index) => {
                // Remove `Entity::Empty` and update `empty_index` for `entities`
                self.empty.swap_remove(empty_index);
                if empty_index < self.empty.len() {
                    let position = &self.empty[empty_index];
                    self.board[position.0] = CellWithMetadata::Empty(empty_index);
                }

                // Remove the LRU `Entity::Snake`
                let tail = self.snake.pop_back().expect("non-empty snake");
                let empty_index = self.empty.len();
                self.board[tail.0] = CellWithMetadata::Empty(empty_index);
                self.empty.push(tail);
            }
            CellWithMetadata::Food(foods_index) => {
                // Remove `Entity::Food` and update `food_index` for `entities`
                self.foods.swap_remove(foods_index);
                if foods_index < self.foods.len() {
                    let position = &self.foods[foods_index];
                    self.board[position.0] = CellWithMetadata::Food(foods_index);
                }

                // Replace eaten `Entity::Food`
                let _ = self.add_food();
            }
        }

        // Update `entities` with the new `Location`, `head`
        self.board[head.0] = CellWithMetadata::Snake;
        self.snake.push_front(head);

        Ok(())
    }

    fn compute_head(&self) -> Position {
        let (n_rows, n_cols) = self.options.shape;
        let Position([i_0, j_0]) = self.snake.front().expect("non-zero length snake");
        let Velocity([d_i, d_j]) = self.velocity;
        let i_1 = i_0
            .checked_add_signed(d_i)
            .unwrap_or(n_rows - Direction::DEFAULT_MAGNITUDE)
            % n_rows;
        let j_1 = j_0
            .checked_add_signed(d_j)
            .unwrap_or(n_cols - Direction::DEFAULT_MAGNITUDE)
            % n_cols;
        Position([i_1, j_1])
    }

    fn add_food(&mut self) -> Result<(), FoodCannotBeAdded> {
        if self.foods.len() >= self.options.n_foods {
            panic!("max foods");
        }

        if self.empty.is_empty() {
            return Err(FoodCannotBeAdded);
        }

        // Remove a random instance of `CellWithMetadata::Empty` and get its `Position`
        let empty_index = self.rng.gen_range(0..self.empty.len());
        let position = self.empty.swap_remove(empty_index);

        // Update `self.board` reference to the `CellWithMetadata::Empty` that was swapped
        if empty_index < self.empty.len() {
            let position = &self.empty[empty_index];
            self.board[position.0] = CellWithMetadata::Empty(empty_index);
        }

        // Add new instance of `CellWithMetadata::Food`
        let foods_index = self.foods.len();
        self.board[position.0] = CellWithMetadata::Food(foods_index);
        self.foods.push(position);

        Ok(())
    }
}

impl Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.board)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod options {
        use super::*;

        const TEST_OPTIONS: Options = Options {
            shape: (3, 3),
            n_foods: 1,
            seed: Some(0),
        };

        #[test]
        fn build() {
            let game_state = TEST_OPTIONS.build();
            assert_eq!(game_state.options, TEST_OPTIONS);
            assert!(matches!(game_state.status, Status::Ongoing));
            assert_eq!(game_state.velocity, Velocity::default());
            assert_eq!(
                game_state.board,
                Array2::from_shape_vec(
                    TEST_OPTIONS.shape,
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
            }
            .build();
        }

        #[test]
        fn is_valid_board_size() {
            assert_eq!(TEST_OPTIONS.is_valid_board_size(), true);
        }

        #[test]
        fn get_head() {
            assert_eq!(TEST_OPTIONS.get_head(), [1, 1]);
        }

        #[test]
        fn get_board() {
            let actual = TEST_OPTIONS.get_board();
            let expected = Array2::from_shape_vec(
                TEST_OPTIONS.shape,
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
            let actual = TEST_OPTIONS.get_empty();
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
            let actual = TEST_OPTIONS.get_snake();
            let expected = vec![Position(TEST_OPTIONS.get_head())];
            assert_eq!(actual, expected);
        }

        #[test]
        fn get_foods() {
            assert_eq!(TEST_OPTIONS.get_foods(), Vec::new());
        }

        #[test]
        fn get_rng_some() {
            let actual = TEST_OPTIONS.get_rng();
            let expected = ChaCha8Rng::seed_from_u64(TEST_OPTIONS.seed.unwrap());
            assert_eq!(actual, expected);
        }

        #[test]
        fn get_rng_none() {
            // This just asserts it doesn't `panic!`
            Options {
                shape: (3, 3),
                n_foods: 1,
                seed: None,
            }
            .get_rng();
        }
    }

    #[cfg(test)]
    mod velocity {
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
    }
}
