use crate::data_transfer::Cell;
use std::collections::{HashSet, VecDeque};

struct Position(usize, usize);

struct Velocity(isize, isize);

pub struct GameState<const N_ROWS: usize, const N_COLS: usize> {
    board: [[Cell; N_COLS]; N_ROWS],
    empty: HashSet<Position>,
    snake: VecDeque<Position>,
}
