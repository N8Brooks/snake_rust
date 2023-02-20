#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Direction {
    Right,
    Up,
    Left,
    Down,
}

#[derive(Debug, PartialEq)]
pub enum Status {
    Ongoing,
    Over { is_won: bool },
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Cell {
    Empty,
    Foods,
    /// A snake segment with an entra
    Snake(Path),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Path {
    pub entry: Option<Direction>,
    pub exit: Option<Direction>,
}
