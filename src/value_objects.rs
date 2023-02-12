use crate::data_transfer_objects as dto;

#[derive(PartialEq, Hash, Eq, Debug, Copy, Clone)]
pub struct Position(pub usize, pub usize);

#[derive(PartialEq, Debug)]
pub struct Velocity(pub isize, pub isize);

impl Velocity {
    pub const DEFAULT_MAGNITUDE: usize = 1;
}

impl dto::Direction {
    pub fn as_velocity(&self) -> Velocity {
        match self {
            dto::Direction::Right => Velocity(0, 1),
            dto::Direction::Up => Velocity(-1, 0),
            dto::Direction::Left => Velocity(0, -1),
            dto::Direction::Down => Velocity(1, 0),
        }
    }

    pub fn opposite(&self) -> dto::Direction {
        match self {
            dto::Direction::Right => dto::Direction::Left,
            dto::Direction::Up => dto::Direction::Down,
            dto::Direction::Left => dto::Direction::Right,
            dto::Direction::Down => dto::Direction::Up,
        }
    }
}

#[cfg(test)]
mod direction_tests {
    use super::*;

    #[test]
    fn as_velocity() {
        assert_eq!(dto::Direction::Right.as_velocity(), Velocity(0, 1));
        assert_eq!(dto::Direction::Up.as_velocity(), Velocity(-1, 0));
        assert_eq!(dto::Direction::Left.as_velocity(), Velocity(0, -1));
        assert_eq!(dto::Direction::Down.as_velocity(), Velocity(1, 0));
    }

    #[test]
    fn opposite() {
        assert_eq!(dto::Direction::Right.opposite(), dto::Direction::Left);
        assert_eq!(dto::Direction::Up.opposite(), dto::Direction::Down);
        assert_eq!(dto::Direction::Left.opposite(), dto::Direction::Right);
        assert_eq!(dto::Direction::Down.opposite(), dto::Direction::Up);
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Cell {
    Empty(usize),
    Foods(usize),
    /// A snake segment with an entra
    Snake {
        entry: Option<dto::Direction>,
        exit: Option<dto::Direction>,
    },
}

impl Cell {
    pub fn as_dto(&self) -> dto::Cell {
        match self {
            Cell::Empty(_) => dto::Cell::Empty,
            Cell::Foods(_) => dto::Cell::Foods,
            Cell::Snake { entry, exit } => dto::Cell::Snake {
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
        assert_eq!(Cell::Empty(0).as_dto(), dto::Cell::Empty);
    }

    #[test]
    fn foods_as_dto() {
        assert_eq!(Cell::Foods(0).as_dto(), dto::Cell::Foods);
    }

    #[test]
    fn snake_as_dto() {
        assert_eq!(
            Cell::Snake {
                entry: None,
                exit: None
            }
            .as_dto(),
            dto::Cell::Snake {
                entry: None,
                exit: None
            }
        );
    }
}
