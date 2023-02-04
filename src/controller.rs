use crate::data_transfer::Direction;

trait Controller {
    fn get_direction(&mut self) -> Direction;
}
