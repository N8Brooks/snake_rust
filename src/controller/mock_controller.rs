use super::Controller;
use crate::direction::Direction;
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct MockController {
    pub directions: VecDeque<Direction>,
}

impl Controller for MockController {
    fn get_direction(&mut self) -> Direction {
        self.directions.pop_front().expect("more directions")
    }
}

impl MockController {
    pub fn new(directions: VecDeque<Direction>) -> MockController {
        MockController { directions }
    }

    pub fn empty() -> MockController {
        MockController {
            directions: VecDeque::new(),
        }
    }
}

#[cfg(test)]
mod mock_controller_tests {
    use super::*;

    #[test]
    fn get_direction() {
        let directions = VecDeque::from([Direction::Up, Direction::Down]);
        let mut controller = MockController::new(directions);
        assert_eq!(controller.get_direction(), Direction::Up);
        assert_eq!(controller.directions, VecDeque::from([Direction::Down]));
    }
}
