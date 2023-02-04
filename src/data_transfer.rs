pub enum Direction {
    Right,
    Up,
    Left,
    Down,
}

pub enum Cell {
    Empty,
    Foods,
    Snake(Direction),
}
