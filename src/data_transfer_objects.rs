use rand::distributions::{Distribution, Standard};
use rand::Rng;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Direction {
    Right,
    Up,
    Left,
    Down,
}

impl Direction {
    pub fn get_plane(&self) -> Plane {
        match self {
            Direction::Right => Plane::Horizontal,
            Direction::Up => Plane::Vertical,
            Direction::Left => Plane::Horizontal,
            Direction::Down => Plane::Vertical,
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
}

#[derive(Debug, PartialEq)]
pub enum Plane {
    Horizontal,
    Vertical,
}

#[derive(Debug, PartialEq)]
pub enum Status {
    Ongoing,
    Over { is_won: bool },
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Cell {
    Empty,
    Foods,
    /// A snake segment with an entra
    Snake {
        entry: Option<Direction>,
        exit: Option<Direction>,
    },
}
