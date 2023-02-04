#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Direction {
    Right,
    Up,
    Left,
    Down,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Cell {
    Empty,
    Foods,
    Snake(Option<Direction>),
}
