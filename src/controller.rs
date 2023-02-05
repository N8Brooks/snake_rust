use crate::data_transfer::Direction;
use crate::seeder::Seeder;
use rand::distributions::Standard;
use rand::prelude::{Distribution, SeedableRng};
use rand_chacha::ChaCha8Rng;

pub trait Controller {
    fn get_direction(&mut self) -> Direction;
}

pub struct MockController {
    pub direction: Direction,
}

impl Controller for MockController {
    fn get_direction(&mut self) -> Direction {
        self.direction
    }
}

#[cfg(test)]
mod mock_controller_tests {
    use super::*;

    #[test]
    fn get_direction() {
        let direction = Direction::Up;
        let mut controller = MockController { direction };
        assert_eq!(controller.get_direction(), direction);
    }
}

pub struct RandomController {
    pub direction: Direction,
    rng: ChaCha8Rng,
}

impl RandomController {
    pub fn new(seeder: &mut dyn Seeder) -> RandomController {
        let mut rng = ChaCha8Rng::seed_from_u64(seeder.get_seed());
        let direction = Distribution::<Direction>::sample(&Standard, &mut rng);
        RandomController { direction, rng }
    }
}

impl Controller for RandomController {
    fn get_direction(&mut self) -> Direction {
        let direction: Direction = Distribution::sample(&Standard, &mut self.rng);
        if self.direction.get_plane() == direction.get_plane() {
            self.direction
        } else {
            direction
        }
    }
}

#[cfg(test)]
mod random_controller_tests {
    use super::*;
    use crate::seeder::MockSeeder;

    #[test]
    fn new() {
        let mut seeder = MockSeeder(0);
        let controller = RandomController::new(&mut seeder);
        assert_eq!(controller.direction, Direction::Left);
    }

    #[test]
    fn get_direction() {
        let mut seeder = MockSeeder(0);
        let mut controller = RandomController::new(&mut seeder);
        assert_eq!(controller.get_direction(), Direction::Left);
    }
}
