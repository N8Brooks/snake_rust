use crate::data_transfer_objects as _dto; // Limited usage in `from` only

// TODO: separate state
pub use board::Board as State;
pub use value_objects::*;

mod board;
mod value_objects;

// #[derive(Debug, Clone, PartialEq)]
// pub struct State<const N_ROWS: usize, const N_COLS: usize> {
//     board: Board<N_ROWS, N_COLS>,
//     empty: Vec<Position>,
//     foods: Vec<Position>,
//     snake: VecDeque<Position>,
// }
//
// impl<const N_ROWS: usize, const N_COLS: usize> State<N_ROWS, N_COLS> {
//     fn new(board: &[[dto::Cell; N_COLS]; N_ROWS]) -> State<N_ROWS, N_COLS> {
//         let board = Board::from(board);
//         let empty = board.get_empty();
//         let foods = board.get_foods();
//
//         State {
//             board,
//             empty: board.get_empty(),
//             foods: board.get_foods(),
//             snake: board.get_snake(),
//         }
//     }
// }
//
// #[cfg(test)]
// mod tests {}

impl<const N_ROWS: usize, const N_COLS: usize> From<[[_dto::Cell; N_COLS]; N_ROWS]>
    for State<N_ROWS, N_COLS>
{
    fn from(board: [[_dto::Cell; N_COLS]; N_ROWS]) -> Self {
        let mut empty_count = 0;
        let mut foods_count = 0;
        let board = board.map(|row| {
            row.map(|cell| match cell {
                _dto::Cell::Empty => {
                    let empty_index = empty_count;
                    empty_count += 1;
                    Cell::Empty(empty_index)
                }
                _dto::Cell::Foods => {
                    let foods_index = foods_count;
                    foods_count += 1;
                    Cell::Foods(foods_index)
                }
                _dto::Cell::Snake(path) => Cell::Snake(path),
            })
        });
        State::from(board)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DTO_BOARD: [[_dto::Cell; 3]; 3] = [
        [_dto::Cell::Empty, _dto::Cell::Foods, _dto::Cell::Empty],
        [
            _dto::Cell::Empty,
            _dto::Cell::Snake(Path {
                entry: Some(Direction::Down),
                exit: None,
            }),
            _dto::Cell::Empty,
        ],
        [
            _dto::Cell::Snake(Path {
                entry: None,
                exit: Some(Direction::Up),
            }),
            _dto::Cell::Snake(Path {
                entry: Some(Direction::Left),
                exit: Some(Direction::Right),
            }),
            _dto::Cell::Empty,
        ],
    ];

    const INPUT_BOARD: [[Cell; 3]; 3] = [
        [Cell::Empty(0), Cell::Foods(0), Cell::Empty(1)],
        [
            Cell::Empty(2),
            Cell::Snake(Path {
                entry: Some(Direction::Down),
                exit: None,
            }),
            Cell::Empty(3),
        ],
        [
            Cell::Snake(Path {
                entry: None,
                exit: Some(Direction::Up),
            }),
            Cell::Snake(Path {
                entry: Some(Direction::Left),
                exit: Some(Direction::Right),
            }),
            Cell::Empty(4),
        ],
    ];

    #[test]
    fn from_dto() {
        let board: State<3, 3> = DTO_BOARD.into();
        assert_eq!(board, State::from(INPUT_BOARD));
    }
}
