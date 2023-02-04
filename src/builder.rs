pub struct Builder {
    /// Rows and columns for the game board
    pub shape: (usize, usize),
    /// Max food count at one time
    pub n_foods: usize,
    /// RNG seed to use for placing foods
    pub seed: u64,
}

#[cfg(test)]
mod tests {}
