use rand::Rng;
use snake_rust::{Direction, Gamestate};
use std::{thread, time};

fn main() {
    let mut rng = rand::thread_rng();
    let mut gamestate = Gamestate::<20, 20, 3>::default();
    loop {
        assert!(gamestate.is_valid());
        thread::sleep(time::Duration::from_millis(30));
        let direction = [
            Direction::Up,
            Direction::Left,
            Direction::Right,
            Direction::Down,
        ][rng.gen_range(0..4)]
        .clone();
        clearscreen::clear().expect("failed to clear screen");
        gamestate.update_service(Some(direction));
        println!("{gamestate}\n");
    }
}
