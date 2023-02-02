use super::Controller;
use crate::direction::Direction;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::time::SystemTime;

/// A `Controller` that randomly gives a `Direction`
#[derive(Clone, Debug)]
pub struct RandomController {
    rng: ChaCha8Rng,
}

impl Controller for RandomController {
    fn get_direction(&mut self) -> Direction {
        match self.rng.gen_range(0..4) {
            0 => Direction::Up,
            1 => Direction::Left,
            2 => Direction::Right,
            _ => Direction::Down,
        }
    }
}

impl RandomController {
    /// Creates a new `Controller` with the given seed
    fn new(seed: u64) -> RandomController {
        let rng = ChaCha8Rng::seed_from_u64(seed);
        RandomController { rng }
    }
}

impl Default for RandomController {
    /// Creates a new `Controller` seeded with system time
    fn default() -> Self {
        let seed = SystemTime::now().elapsed().expect("system time").as_secs();
        RandomController::new(seed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::direction::Direction;

    #[test]
    fn get_direction() {
        let mut controller = RandomController::new(42);
        let direction = controller.get_direction();
        assert!(matches!(direction, Direction::Down));
    }

    #[test]
    fn default() {
        // Asserting that is doesn't panic
        RandomController::default();
    }
}
