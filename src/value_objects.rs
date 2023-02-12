use crate::data_transfer::{Cell as CellDto, Direction};

#[derive(PartialEq, Hash, Eq, Debug, Copy, Clone)]
pub struct Position(pub usize, pub usize);

#[derive(PartialEq, Debug)]
pub struct Velocity(pub isize, pub isize);

impl Velocity {
    pub const DEFAULT_MAGNITUDE: usize = 1;
}

impl Direction {
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

#[cfg(test)]
mod direction_tests {
    use super::*;

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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Cell {
    Empty(usize),
    Foods(usize),
    /// A snake segment with an entra
    Snake {
        entry: Option<Direction>,
        exit: Option<Direction>,
    },
}

impl Cell {
    pub fn as_dto(&self) -> CellDto {
        match self {
            Cell::Empty(_) => CellDto::Empty,
            Cell::Foods(_) => CellDto::Foods,
            Cell::Snake { entry, exit } => CellDto::Snake {
                entry: *entry,
                exit: *exit,
            },
        }
    }
}

#[cfg(test)]
mod cell_tests {
    use super::*;

    #[test]
    fn empty_as_dto() {
        assert_eq!(Cell::Empty(0).as_dto(), CellDto::Empty);
    }

    #[test]
    fn foods_as_dto() {
        assert_eq!(Cell::Foods(0).as_dto(), CellDto::Foods);
    }

    #[test]
    fn snake_as_dto() {
        assert_eq!(
            Cell::Snake {
                entry: None,
                exit: None
            }
            .as_dto(),
            CellDto::Snake {
                entry: None,
                exit: None
            }
        );
    }
}
