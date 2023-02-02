use rand::Rng;
use snake_rust::direction::Direction;
use snake_rust::Options;
use std::{thread, time};

fn main() {
    let mut rng = rand::thread_rng();
    let mut game_state = Options::default().build();
    while {
        thread::sleep(time::Duration::from_millis(30));
        let direction = [
            Direction::Up,
            Direction::Left,
            Direction::Right,
            Direction::Down,
        ][rng.gen_range(0..4)]
        .clone();
        clearscreen::clear().expect("failed to clear screen");
        let _ = game_state.set_direction(direction);
        println!("{game_state}");
        game_state.iterate_turn().is_ok()
    } {}
}
