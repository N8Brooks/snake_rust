#[derive(PartialEq, Hash, Eq, Debug)]
pub struct Position(pub usize, pub usize);

#[cfg(test)]
mod position_tests {}

#[derive(PartialEq, Debug)]
pub struct Velocity(pub isize, pub isize);
