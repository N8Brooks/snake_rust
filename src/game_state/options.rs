use crate::controller::Controller;
use crate::game_state::GameState;
use crate::seeder::*;
use crate::view::View;

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

    pub fn with_seed(n_foods: usize, seed: u64) -> Self {
        Options {
            n_foods,
            seeder: Box::new(MockSeeder(seed)),
        }
    }
}

impl<const N_ROWS: usize, const N_COLS: usize> Options<N_ROWS, N_COLS> {
    pub fn build<'a>(
        &self,
        controller: &'a mut dyn Controller,
        view: &'a mut dyn View,
    ) -> Result<GameState<'a, N_ROWS, N_COLS>, InvalidOptions> {
        if self.is_valid() {
            Ok(GameState::from_options(self, controller, view))
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
    use crate::controller::mock_controller::MockController;
    use crate::view::MockView;

    use super::super::value_objects::Direction;
    use super::*;

    #[test]
    fn build_with_invalid() {
        let options = Options::<3, 3>::with_seed(9, 0);
        let mut controller = MockController(Direction::Right);
        let mut view = MockView::default();
        let game_state = options.build(&mut controller, &mut view).unwrap_err();
        assert!(matches!(game_state, InvalidOptions));
    }

    #[test]
    fn is_valid_true() {
        let options = Options::<3, 3>::with_seed(8, 0);
        assert!(options.is_valid());
    }

    #[test]
    fn is_valid_false() {
        let options = Options::<3, 3>::with_seed(9, 0);
        assert!(!options.is_valid());
    }

    #[test]
    fn area() {
        let options = Options::<3, 4>::with_seed(1, 0);
        assert_eq!(options.area(), 12);
    }

    #[test]
    fn n_non_empty() {
        let options = Options::<3, 3>::with_seed(1, 0);
        assert_eq!(options.n_non_empty(), 2);
    }
}
