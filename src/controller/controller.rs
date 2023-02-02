use crate::direction::Direction;
use dyn_clonable::*;
use std::fmt;

#[clonable]
pub trait Controller: Clone {
    fn get_direction(&mut self) -> Direction;
}

impl fmt::Debug for dyn Controller {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        panic!("expected debug")
    }
}
