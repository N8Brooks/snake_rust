use std::fmt::Debug;

use crate::data_transfer_objects as dto;

pub trait View: Debug {
    fn swap_cell(&mut self, position: &dto::Position, old: dto::Cell, new: dto::Cell);
}

#[derive(Default, Debug)]
pub struct MockView(pub Vec<(dto::Position, dto::Cell, dto::Cell)>);

impl View for MockView {
    fn swap_cell(&mut self, position: &dto::Position, old: dto::Cell, new: dto::Cell) {
        self.0.push((*position, old, new));
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn swap_cell() {
        let mut view = MockView::default();
        let position = (0, 1);
        let old = dto::Cell::Foods;
        let new = dto::Cell::Empty;
        view.swap_cell(&position, old, new);
        assert_eq!(view.0, [(position, old, new)]);
    }
}
