#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Direction {
    Right,
    Up,
    Left,
    Down,
}

impl Direction {
    fn as_plane(&self) -> Plane {
        match self {
            Direction::Right => Plane::Horizontal,
            Direction::Up => Plane::Vertical,
            Direction::Left => Plane::Horizontal,
            Direction::Down => Plane::Vertical,
        }
    }
}

#[cfg(test)]
mod direction_tests {
    use super::*;

    #[test]
    fn as_plane() {
        assert_eq!(Direction::Right.as_plane(), Plane::Horizontal);
        assert_eq!(Direction::Up.as_plane(), Plane::Vertical);
        assert_eq!(Direction::Left.as_plane(), Plane::Horizontal);
        assert_eq!(Direction::Down.as_plane(), Plane::Vertical);
    }
}

#[derive(Debug, PartialEq)]
pub enum Plane {
    Horizontal,
    Vertical,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Cell {
    Empty,
    Foods,
    Snake(Option<Direction>),
}
