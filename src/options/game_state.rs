use super::seeder::{SecondsSeeder, Seeder};
use crate::{data_transfer::Cell, value_objects::Position};
use std::collections::{HashSet, VecDeque};

pub struct Options<const N_ROWS: usize, const N_COLS: usize, T: Seeder> {
    n_foods: usize,
    seeder: T,
}

impl Default for Options<10, 10, SecondsSeeder> {
    fn default() -> Self {
        Options {
            n_foods: 1,
            seeder: SecondsSeeder::SECONDS_SEEDER,
        }
    }
}

impl<const N_ROWS: usize, const N_COLS: usize, T: Seeder> Options<N_ROWS, N_COLS, T> {
    fn is_valid(&self) -> bool {
        println!("{}", self.n_non_empty());
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
    use super::*;

    #[test]
    fn is_valid_true() {
        let mut options = Options::default();
        options.n_foods = 99;
        assert!(options.is_valid());
    }

    #[test]
    fn is_valid_false() {
        let mut options = Options::default();
        options.n_foods = 100;
        assert!(!options.is_valid());
    }
}

pub struct GameState<const N_ROWS: usize, const N_COLS: usize> {
    board: [[Cell; N_COLS]; N_ROWS],
    empty: HashSet<Position>,
    snake: VecDeque<Position>,
}
