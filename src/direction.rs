/// Data transfer object for game_state-controller interfacing.
#[derive(Clone, Debug, PartialEq)]
pub enum Direction {
    Up,
    Left,
    Right,
    Down,
}
