use std::fmt::Debug;

use crate::{data_transfer_objects::Cell, value_objects::Position};

pub trait View: Debug {
    fn swap_cell(&mut self, position: &Position, old: Cell, new: Cell);
}

#[derive(Default, Debug)]
pub struct MockView(pub Vec<(Position, Cell, Cell)>);

impl View for MockView {
    fn swap_cell(&mut self, position: &Position, old: Cell, new: Cell) {
        self.0.push((*position, old, new));
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn swap_cell() {
        let mut view = MockView::default();
        let position = Position(0, 1);
        let old = Cell::Foods;
        let new = Cell::Empty;
        view.swap_cell(&position, old, new);
        assert_eq!(view.0, [(position, old, new)]);
    }
}
