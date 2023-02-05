use crate::data_transfer::Direction;

#[derive(PartialEq, Hash, Eq, Debug)]
pub struct Position(pub usize, pub usize);

#[cfg(test)]
mod position_tests {}

#[derive(PartialEq, Debug)]
pub struct Velocity(pub isize, pub isize);

#[derive(Debug)]
pub struct InvalidDirection;

impl Velocity {
    pub fn is_vertical(&self) -> bool {
        self.0 != 0 && self.1 == 0
    }

    pub fn from_direction(direction: &Direction) -> Velocity {
        match direction {
            Direction::Right => Velocity(0, 1),
            Direction::Up => Velocity(-1, 0),
            Direction::Left => Velocity(0, -1),
            Direction::Down => Velocity(1, 0),
        }
    }
}

#[cfg(test)]
mod velocity_tests {
    use super::*;

    #[test]
    fn is_vertical() {
        assert!(Velocity(-1, 0).is_vertical());
    }

    #[test]
    fn is_vertical_false() {
        assert!(!Velocity(0, -1).is_vertical());
    }

    #[test]
    fn from_direction() {
        assert_eq!(Velocity::from_direction(&Direction::Right), Velocity(0, 1));
        assert_eq!(Velocity::from_direction(&Direction::Up), Velocity(-1, 0));
        assert_eq!(Velocity::from_direction(&Direction::Down), Velocity(1, 0));
        assert_eq!(Velocity::from_direction(&Direction::Left), Velocity(0, -1));
    }
}
