use crate::data_transfer::Direction;

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
