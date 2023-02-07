use crate::controller::Controller;
use crate::game_state::GameState;
use crate::seeder::*;

#[derive(Debug)]
pub struct InvalidOptions;

pub struct Options<const N_ROWS: usize, const N_COLS: usize> {
    pub n_foods: usize,
    pub seeder: Box<dyn Seeder>,
}

impl<const N_ROWS: usize, const N_COLS: usize> Options<N_ROWS, N_COLS> {
    pub fn new(n_foods: usize) -> Self {
        Options {
            n_foods,
            seeder: Box::new(SecondsSeeder::SECONDS_SEEDER),
        }
    }

    pub fn with_mock_seeder(n_foods: usize, seed: u64) -> Self {
        Options {
            n_foods,
            seeder: Box::new(MockSeeder(seed)),
        }
    }
}

impl<const N_ROWS: usize, const N_COLS: usize> Options<N_ROWS, N_COLS> {
    pub fn build(
        &self,
        controller: Box<dyn Controller>,
    ) -> Result<GameState<N_ROWS, N_COLS>, InvalidOptions> {
        if self.is_valid() {
            Ok(GameState::from_options(self, controller))
        } else {
            Err(InvalidOptions)
        }
    }

    fn is_valid(&self) -> bool {
        self.area() >= self.n_non_empty()
    }

    fn area(&self) -> usize {
        N_ROWS * N_COLS
    }

    fn n_non_empty(&self) -> usize {
        let n_snake = 1;
        self.n_foods + n_snake
    }
}

#[cfg(test)]
mod options_tests {
    use super::*;
    use crate::controller::MockController;
    use crate::data_transfer::*;

    #[test]
    fn build_with_invalid() {
        let options = Options::<3, 3>::with_mock_seeder(9, 0);
        let controller = Box::new(MockController(Direction::Right));
        let game_state = options.build(controller).unwrap_err();
        assert!(matches!(game_state, InvalidOptions));
    }

    #[test]
    fn is_valid_true() {
        let options = Options::<3, 3>::with_mock_seeder(8, 0);
        assert!(options.is_valid());
    }

    #[test]
    fn is_valid_false() {
        let options = Options::<3, 3>::with_mock_seeder(9, 0);
        assert!(!options.is_valid());
    }

    #[test]
    fn area() {
        let options = Options::<3, 4>::with_mock_seeder(1, 0);
        assert_eq!(options.area(), 12);
    }

    #[test]
    fn n_non_empty() {
        let options = Options::<3, 3>::with_mock_seeder(1, 0);
        assert_eq!(options.n_non_empty(), 2);
    }
}
