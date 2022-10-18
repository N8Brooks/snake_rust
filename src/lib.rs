use rand::Rng;
use std::collections::{HashSet, VecDeque};
use std::fmt::Display;

/// Player command for controlling the `Gamestate`.
#[derive(Clone, Debug)]
pub enum Direction {
    Up,
    Left,
    Right,
    Down,
}

/// Current status of `Gamestate`.
#[derive(Clone, Debug)]
pub enum Status {
    Turn,
    Over { is_winner: bool },
}

/// Row-index, col-index pair; usually de-structured as `(i, j)`.
type Location = (usize, usize);

/// Change in `Location`.
#[derive(Clone, Debug)]
struct LocationDelta(isize, isize);

impl LocationDelta {
    /// The corresponding `LocationDelta` for a `Direction`.
    fn from_direction(direction: Direction) -> LocationDelta {
        match direction {
            Direction::Up => LocationDelta(1, 0),
            Direction::Left => LocationDelta(0, -1),
            Direction::Right => LocationDelta(0, 1),
            Direction::Down => LocationDelta(-1, 0),
        }
    }

    /// Whether this `LocationDelta` goes `Direction::Up` or `Direction::Down`.
    fn is_vertical(&self) -> bool {
        self.0 != 0
    }
}

/// Possible states of a cell in `Gamestate`.
#[derive(Clone, Debug)]
enum Entity {
    Snake,
    Empty { empty_index: usize },
    Food { foods_index: usize },
}

/// Contains pieces of state for representing the game 'Snake'.
#[derive(Clone, Debug)]
pub struct Gamestate<const N_ROWS: usize, const N_COLS: usize, const N_FOODS: usize> {
    heading: LocationDelta,
    snake: VecDeque<Location>,
    empty: Vec<Location>,
    foods: Vec<Location>,
    entities: [[Entity; N_COLS]; N_ROWS],
}

impl<const N_ROWS: usize, const N_COLS: usize, const N_FOODS: usize>
    Gamestate<N_ROWS, N_COLS, N_FOODS>
{
    /// Generate a new `Gamestate` given appropriate const generics.
    pub fn new(
        initial_direction: Direction,
        snake_head_location: Location,
        n_foods: usize,
    ) -> Self {
        // Protect `Gamestate` from invariant options
        assert!(N_ROWS > 0, "`N_ROWS` must be non-zero");
        assert!(N_COLS > 0, "`N_COLS` must be non-zero");

        // Generate `entities` without `Entity::Food`
        let mut snake = VecDeque::with_capacity(N_ROWS * N_COLS);
        let mut empty = Vec::with_capacity(N_ROWS * N_COLS);
        let entities: [[Entity; N_COLS]; N_ROWS] = (0..N_ROWS)
            .map(|i| {
                (0..N_COLS)
                    .map(|j| {
                        let location: Location = (i, j);
                        if location == snake_head_location {
                            snake.push_front(location);
                            Entity::Snake
                        } else {
                            empty.push(location);
                            Entity::Empty {
                                empty_index: empty.len() - 1,
                            }
                        }
                    })
                    .collect::<Vec<Entity>>()
                    .try_into()
                    .expect("width equals N_COLS")
            })
            .collect::<Vec<[Entity; N_COLS]>>()
            .try_into()
            .expect("height equals N_ROWS");

        // Produce `Gamestate` instance and add requied `Entity::Food`s
        let foods = Vec::with_capacity(n_foods);
        let mut gamestate = Gamestate {
            heading: LocationDelta::from_direction(initial_direction),
            snake,
            empty,
            foods,
            entities,
        };
        gamestate.place_foods();

        gamestate
    }

    /// Takes an optional `Direction` and returns the new `Gamestate` `Status`.
    pub fn update_service(&mut self, direction: Option<Direction>) -> Status {
        // Update heading for strictly left or right turns
        if let Some(direction) = direction {
            let location_delta = LocationDelta::from_direction(direction);
            if self.heading.is_vertical() != location_delta.is_vertical() {
                self.heading = location_delta;
            }
        }

        // Find the new `head` from the current `Snake` + `heading` mod `SIZE`
        let head = (
            checked_add_signed(self.snake[0].0, self.heading.0).unwrap_or(N_ROWS - 1) % N_ROWS,
            checked_add_signed(self.snake[0].1, self.heading.1).unwrap_or(N_COLS - 1) % N_COLS,
        );

        match self.entities[head.0][head.1] {
            Entity::Snake => {
                // Player wins if all `entities` are `Entity::Snake`
                return Status::Over {
                    is_winner: self.empty.is_empty() && self.foods.is_empty(),
                };
            }
            Entity::Empty { empty_index } => {
                // Remove `Entity::Empty` and update `empty_index` for `entities`
                self.empty.swap_remove(empty_index);
                if empty_index < self.empty.len() {
                    let (i, j) = self.empty[empty_index];
                    self.entities[i][j] = Entity::Empty { empty_index };
                }

                // Remove the LRU `Entity::Snake`
                let tail = self.snake.pop_back().expect("non-empty snake");
                let empty_index = self.empty.len();
                self.entities[tail.0][tail.1] = Entity::Empty { empty_index };
                self.empty.push(tail);
            }
            Entity::Food { foods_index } => {
                // Remove `Entity::Food` and update `food_index` for `entities`
                self.foods.swap_remove(foods_index);
                if foods_index < self.foods.len() {
                    let (i, j) = self.foods[foods_index];
                    self.entities[i][j] = Entity::Food { foods_index };
                }

                // Replace eaten `Entity::Food`
                self.place_foods();
            }
        }

        // Update `entities` with the new `Location`, `head`
        self.snake.push_front(head);
        self.entities[head.0][head.1] = Entity::Snake;

        // At this point, the player has not lost or won
        Status::Turn
    }

    /// Replace necessary `Entity::Empty`s with `Entity::Food`, if possible.
    fn place_foods(&mut self) {
        let mut rng = rand::thread_rng();
        while !self.empty.is_empty() && self.foods.len() < N_FOODS {
            // Remove a random instance of `Entity::Empty` and get its `Location`
            let empty_index = rng.gen_range(0..self.empty.len());
            let loc = self.empty.swap_remove(empty_index);

            // Update `entities` reference to the `Entity::Empty` that was swapped
            if empty_index < self.empty.len() {
                let (i, j) = self.empty[empty_index];
                self.entities[i][j] = Entity::Empty { empty_index };
            }

            // Add new instance of `Entity::food`
            let foods_index = self.foods.len();
            self.entities[loc.0][loc.1] = Entity::Food { foods_index };
            self.foods.push(loc);
        }
    }

    /// Method to check `Gamestate` for an invariant/invalid `Gamestate`.
    #[allow(unused_variables)]
    pub fn is_valid(&self) -> bool {
        let mut locations: HashSet<_> = (0..N_ROWS)
            .flat_map(|i| (0..N_COLS).map(move |j| (i, j)))
            .collect();
        let is_snake_valid = self.snake.iter().all(|loc| {
            let actual = self.entities[loc.0][loc.1].clone();
            let expected = Entity::Snake;
            locations.remove(loc) && matches!(actual, expected)
        });
        let is_empty_valid = self.empty.iter().enumerate().all(|(empty_index, loc)| {
            let actual = self.entities[loc.0][loc.1].clone();
            let expected = Entity::Empty { empty_index };
            locations.remove(loc) && matches!(actual, expected)
        });
        let is_foods_valid = self.foods.iter().enumerate().all(|(foods_index, loc)| {
            let actual = self.entities[loc.0][loc.1].clone();
            let expected = Entity::Food { foods_index };
            locations.remove(loc) && matches!(actual, expected)
        });
        is_snake_valid && is_empty_valid && is_foods_valid && locations.is_empty()
    }
}

impl<const N_ROWS: usize, const N_COLS: usize, const N_FOODS: usize> Default
    for Gamestate<N_ROWS, N_COLS, N_FOODS>
{
    /// A `Gamestate` with reasonable default options.
    fn default() -> Self {
        Gamestate::new(Direction::Right, (N_ROWS / 2, N_COLS / 2), 1)
    }
}

impl<const N_ROWS: usize, const N_COLS: usize, const N_FOODS: usize> Display
    for Gamestate<N_ROWS, N_COLS, N_FOODS>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x: String = self
            .entities
            .clone()
            .map(|row| {
                row.map(|entity| match entity {
                    Entity::Snake => "░░",
                    Entity::Food { .. } => "▒▒",
                    Entity::Empty { .. } => "██",
                })
                .join("")
            })
            .join("\n");

        write!(f, "{}", x)
    }
}

fn checked_add_signed(a: usize, b: isize) -> Option<usize> {
    let total = a as isize + b;
    if total < 0 {
        None
    } else {
        Some(total as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::seq::SliceRandom;

    const DIRECTIONS: [Direction; 4] = [
        Direction::Right,
        Direction::Down,
        Direction::Left,
        Direction::Up,
    ];

    #[test]
    fn is_winner_false() {
        let mut rng = rand::thread_rng();
        let mut gamestate = Gamestate::<20, 20, 3>::default();
        let mut status: Status;
        while {
            assert!(gamestate.is_valid());
            // Go around randomly until the game is lost
            let direction = DIRECTIONS.choose(&mut rng).unwrap().clone();
            status = gamestate.update_service(Some(direction));
            matches!(status, Status::Turn)
        } {}
        // A random playthrough win on a 20x20 board is nigh impossible
        assert!(matches!(status, Status::Over { is_winner: false }));
    }

    #[test]
    fn is_winner_true() {
        let mut gamestate = Gamestate::<2, 2, 1>::default();
        let mut status: Status;
        let mut directions = DIRECTIONS.iter().cycle();
        while {
            assert!(gamestate.is_valid());
            // Go in a circle until the game is won
            let direction = directions.next().unwrap().clone();
            status = gamestate.update_service(Some(direction));
            matches!(status, Status::Turn)
        } {}
        // A random playthrough win on a 20x20 board is nigh impossible
        assert!(matches!(status, Status::Over { is_winner: true }));
    }
}
