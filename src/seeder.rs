use std::time::{SystemTime, UNIX_EPOCH};

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

pub trait Seeder {
    fn get_seed(&self) -> u64;

    fn get_rng(&self) -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(self.get_seed())
    }
}

#[derive(Default)]
pub struct SecondsSeeder;

impl Seeder for SecondsSeeder {
    fn get_seed(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_secs()
    }
}

impl SecondsSeeder {
    pub const SECONDS_SEEDER: SecondsSeeder = SecondsSeeder {};
}

#[derive(Default)]
pub struct MockSeeder(pub u64);

impl Seeder for MockSeeder {
    fn get_seed(&self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seconds_seeder_get_secs() {
        SecondsSeeder::SECONDS_SEEDER.get_seed();
    }

    #[test]
    fn mock_seeder_get_secs() {
        assert_eq!(MockSeeder(0).get_seed(), 0);
    }
}
