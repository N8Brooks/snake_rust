use crate::data_transfer_objects as dto;
use rand::distributions::{Distribution, Standard};
use rand::Rng;

pub use dto::{Direction, Path}; // Re-implementation not deemed worthwhile

impl Direction {
    pub fn get_plane(&self) -> Plane {
        match self {
            Direction::Right => Plane::Horizontal,
            Direction::Up => Plane::Vertical,
            Direction::Left => Plane::Horizontal,
            Direction::Down => Plane::Vertical,
        }
    }

    pub fn as_velocity(&self) -> Velocity {
        match self {
            Direction::Right => Velocity(0, 1),
            Direction::Up => Velocity(-1, 0),
            Direction::Left => Velocity(0, -1),
            Direction::Down => Velocity(1, 0),
        }
    }

    pub fn opposite(&self) -> Direction {
        match self {
            Direction::Right => Direction::Left,
            Direction::Up => Direction::Down,
            Direction::Left => Direction::Right,
            Direction::Down => Direction::Up,
        }
    }
}

impl Distribution<Direction> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        match rng.gen_range(0..4) {
            0 => Direction::Right,
            1 => Direction::Up,
            2 => Direction::Left,
            _ => Direction::Down,
        }
    }
}

#[cfg(test)]
mod direction_tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn as_plane() {
        assert_eq!(Direction::Right.get_plane(), Plane::Horizontal);
        assert_eq!(Direction::Up.get_plane(), Plane::Vertical);
        assert_eq!(Direction::Left.get_plane(), Plane::Horizontal);
        assert_eq!(Direction::Down.get_plane(), Plane::Vertical);
    }

    #[test]
    fn sample() {
        let mut rng = ChaCha8Rng::seed_from_u64(0);
        let direction: Direction = Distribution::sample(&Standard, &mut rng);
        assert_eq!(direction, Direction::Left);
    }

    #[test]
    fn as_velocity() {
        assert_eq!(Direction::Right.as_velocity(), Velocity(0, 1));
        assert_eq!(Direction::Up.as_velocity(), Velocity(-1, 0));
        assert_eq!(Direction::Left.as_velocity(), Velocity(0, -1));
        assert_eq!(Direction::Down.as_velocity(), Velocity(1, 0));
    }

    #[test]
    fn opposite() {
        assert_eq!(Direction::Right.opposite(), Direction::Left);
        assert_eq!(Direction::Up.opposite(), Direction::Down);
        assert_eq!(Direction::Left.opposite(), Direction::Right);
        assert_eq!(Direction::Down.opposite(), Direction::Up);
    }
}

#[derive(Debug, PartialEq)]
pub enum Plane {
    Horizontal,
    Vertical,
}

#[derive(PartialEq, Hash, Eq, Debug, Copy, Clone)]
pub struct Position(pub usize, pub usize);

impl From<Position> for dto::Position {
    fn from(position: Position) -> Self {
        (position.0, position.1)
    }
}

#[cfg(test)]
mod position_tests {
    use super::Position;
    use crate::data_transfer_objects::Position as DtoPosition;

    #[test]
    fn from() {
        let position = Position(0, 1);
        let actual = DtoPosition::from(position);
        assert_eq!(actual, (0, 1));
    }
}

#[derive(PartialEq, Debug)]
pub struct Velocity(pub isize, pub isize);

impl Velocity {
    pub const DEFAULT_MAGNITUDE: usize = 1;
}

impl Direction {}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Cell {
    Empty(usize),
    Foods(usize),
    Snake(Path),
}

impl From<Cell> for dto::Cell {
    fn from(cell: Cell) -> Self {
        match cell {
            Cell::Empty(_) => dto::Cell::Empty,
            Cell::Foods(_) => dto::Cell::Foods,
            Cell::Snake(path) => dto::Cell::Snake(path),
        }
    }
}

#[cfg(test)]
mod cell_tests {
    use super::*;

    #[test]
    fn empty_into() {
        let actual = dto::Cell::from(Cell::Empty(0));
        assert_eq!(actual, dto::Cell::Empty);
    }

    #[test]
    fn foods_from_into() {
        let actual: dto::Cell = Cell::Foods(0).into();
        assert_eq!(actual, dto::Cell::Foods);
    }

    #[test]
    fn snake_from_into() {
        let actual: dto::Cell = Cell::Snake(Path {
            entry: None,
            exit: None,
        })
        .into();
        assert_eq!(
            actual,
            dto::Cell::Snake(dto::Path {
                entry: None,
                exit: None
            })
        );
    }
}
