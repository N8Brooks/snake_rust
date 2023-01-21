use ndarray::Array2;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::time::SystemTime;

/// The default magnitude for a velocity given by a direction.
const DEFAULT_MAGNITUDE: isize = 1;

pub struct Options {
    shape: (usize, usize),
    n_foods: usize,
    seed: Option<u64>,
}

impl Options {
    fn is_valid(&self) -> bool {
        self.shape.0 > 0 && self.shape.1 > 0 && self.n_foods > 0
    }
}

enum Status {
    Ongoing,
    Over { is_won: bool },
}

/// A board reference to a cell referred to as `[i, j]`.
struct Position([usize; 2]);

/// A 1 turn change in a `Position` referred to as `[di, dj]`.
#[derive(Default)]
struct Velocity([isize; 2]);

impl Velocity {
    fn is_vertical(&self) -> bool {
        self.0[0] != 0 && self.0[1] == 0
    }
}

enum Direction {
    Up,
    Left,
    Right,
    Down,
}

impl Direction {
    fn as_velocity(&self) -> Velocity {
        match self {
            Direction::Up => Velocity([-DEFAULT_MAGNITUDE, 0]),
            Direction::Left => Velocity([0, -DEFAULT_MAGNITUDE]),
            Direction::Right => Velocity([0, DEFAULT_MAGNITUDE]),
            Direction::Down => Velocity([DEFAULT_MAGNITUDE, 0]),
        }
    }
}

enum Cell {
    Snake,
    Empty,
    Food,
}

enum CellWithMetadata {
    Snake,
    Empty(usize),
    Food(usize),
}

enum Command {
    SetDirection(Direction),
    IterateTurn,
}

enum Error {
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

impl GameState {
    fn new(options: Options) -> Self {
        if !options.is_valid() {
            panic!("invalid options");
        }
        let (n_rows, n_cols) = options.shape;
        let head = [n_rows / 2, n_cols / 2];
        let head_index = head[0] * n_rows + head[1] * n_cols;
        let board = Array2::from_shape_fn((n_rows, n_cols), |(i, j)| {
            let index = i * n_rows + j * n_cols;
            match index.cmp(&head_index) {
                Ordering::Less => CellWithMetadata::Empty(index),
                Ordering::Equal => CellWithMetadata::Snake,
                Ordering::Greater => CellWithMetadata::Empty(index - 1),
            }
        });
        let empty = board
            .indexed_iter()
            .filter(|(_, cell)| matches!(cell, CellWithMetadata::Empty(_)))
            .map(|(index, _)| Position([index.0, index.1]))
            .collect();
        let foods = Vec::with_capacity(options.n_foods);
        let seed = options
            .seed
            .unwrap_or_else(|| SystemTime::now().elapsed().expect("system time").as_secs());
        let mut game_state = GameState {
            options,
            status: Status::Ongoing,
            velocity: Velocity::default(),
            board,
            snake: {
                let mut snake = VecDeque::with_capacity(n_rows * n_cols);
                snake.push_front(Position(head));
                snake
            },
            empty,
            foods,
            rng: ChaCha8Rng::seed_from_u64(seed),
        };
        game_state.add_foods();
        game_state
    }

    fn handle(&mut self, command: Command) -> Result<(), Error> {
        if matches!(self.status, Status::Over { .. }) {
            return Err(Error::GameIsOver);
        }
        match command {
            Command::SetDirection(direction) => {
                let velocity = direction.as_velocity();
                if velocity.is_vertical() == self.velocity.is_vertical() {
                    Err(Error::InvalidDirection)
                } else {
                    self.velocity = velocity;
                    Ok(())
                }
            }
            Command::IterateTurn => {
                let head = self.snake.front().expect("non-zero length snake");
                match self.board[head.0] {
                    CellWithMetadata::Snake => {
                        self.status = Status::Over {
                            is_won: self.empty.is_empty() && self.foods.is_empty(),
                        };
                    }
                    CellWithMetadata::Empty(position) => {}
                    CellWithMetadata::Food(position) => {}
                }
                Ok(())
            }
        }
    }

    fn add_foods(&mut self) {
        while !self.empty.is_empty() && self.foods.len() < self.options.n_foods {
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
        }
    }
}
