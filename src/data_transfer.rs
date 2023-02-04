#[derive(Copy, Clone)]
pub enum Direction {
    Right,
    Up,
    Left,
    Down,
}

#[derive(Copy, Clone)]
pub enum Cell {
    Empty,
    Foods,
    Snake(Direction),
}
