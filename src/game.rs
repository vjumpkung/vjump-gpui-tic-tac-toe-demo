//! Pure Tic Tac Toe rules: marks, the 3x3 board and the outcome of a position.

/// A player's mark.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    X,
    O,
}

impl Mark {
    /// The opposing mark.
    pub fn other(self) -> Self {
        match self {
            Self::X => Self::O,
            Self::O => Self::X,
        }
    }

    /// The letter drawn on the board.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::X => "X",
            Self::O => "O",
        }
    }
}

/// Every winning triple of cell indices: rows, columns, then diagonals.
const LINES: [[usize; 3]; 8] = [
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8],
    [0, 3, 6],
    [1, 4, 7],
    [2, 5, 8],
    [0, 4, 8],
    [2, 4, 6],
];

/// The result of evaluating a position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    InProgress,
    Won { mark: Mark, line: [usize; 3] },
    Draw,
}

/// A 3x3 board, indexed row by row from 0 (top left) to 8 (bottom right).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Board([Option<Mark>; 9]);

impl Board {
    /// The mark at `ix`, or `None` for an empty or out-of-range cell.
    pub fn get(&self, ix: usize) -> Option<Mark> {
        self.0.get(ix).copied().flatten()
    }

    /// Whether `ix` is a real cell with no mark. False when out of range.
    pub fn is_empty_at(&self, ix: usize) -> bool {
        self.0.get(ix).is_some_and(Option::is_none)
    }

    /// Places `mark` at `ix`. Callers validate `ix` first.
    pub(crate) fn set(&mut self, ix: usize, mark: Mark) {
        self.0[ix] = Some(mark);
    }

    /// Indices of the cells with no mark, in ascending order.
    pub fn empty_cells(&self) -> impl Iterator<Item = usize> + '_ {
        self.0
            .iter()
            .enumerate()
            .filter_map(|(ix, cell)| cell.is_none().then_some(ix))
    }

    /// Whether no cell has a mark.
    pub fn is_empty(&self) -> bool {
        self.0.iter().all(Option::is_none)
    }

    /// Whether the position is won, drawn or still being played.
    pub fn outcome(&self) -> Outcome {
        for line in LINES {
            if let Some(mark) = self.get(line[0])
                && line.iter().all(|&ix| self.get(ix) == Some(mark))
            {
                return Outcome::Won { mark, line };
            }
        }
        if self.0.iter().all(Option::is_some) {
            Outcome::Draw
        } else {
            Outcome::InProgress
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Board, LINES, Mark, Outcome};

    fn board_with(marks: &[(usize, Mark)]) -> Board {
        let mut board = Board::default();
        for &(ix, mark) in marks {
            board.set(ix, mark);
        }
        board
    }

    #[test]
    fn every_line_is_detected_as_a_win() {
        for line in LINES {
            let marks: Vec<(usize, Mark)> = line.iter().map(|&ix| (ix, Mark::O)).collect();
            let board = board_with(&marks);
            assert_eq!(
                board.outcome(),
                Outcome::Won {
                    mark: Mark::O,
                    line
                },
                "line {line:?}"
            );
        }
    }

    #[test]
    fn a_win_is_reported_for_the_mark_that_owns_the_line() {
        // X: 0, 1, 2 across the top; O fills part of the middle row.
        let board = board_with(&[
            (0, Mark::X),
            (1, Mark::X),
            (2, Mark::X),
            (3, Mark::O),
            (4, Mark::O),
        ]);
        assert_eq!(
            board.outcome(),
            Outcome::Won {
                mark: Mark::X,
                line: [0, 1, 2]
            }
        );
    }

    #[test]
    fn a_full_board_without_a_line_is_a_draw() {
        // X O X
        // X O O
        // O X X
        let board = board_with(&[
            (0, Mark::X),
            (1, Mark::O),
            (2, Mark::X),
            (3, Mark::X),
            (4, Mark::O),
            (5, Mark::O),
            (6, Mark::O),
            (7, Mark::X),
            (8, Mark::X),
        ]);
        assert_eq!(board.outcome(), Outcome::Draw);
    }

    #[test]
    fn a_partial_board_without_a_line_is_in_progress() {
        assert_eq!(Board::default().outcome(), Outcome::InProgress);
        let board = board_with(&[(0, Mark::X), (4, Mark::O), (8, Mark::X)]);
        assert_eq!(board.outcome(), Outcome::InProgress);
    }

    #[test]
    fn out_of_range_cells_are_never_empty() {
        let board = Board::default();
        assert!(board.is_empty_at(0));
        assert!(board.is_empty_at(8));
        assert!(!board.is_empty_at(9));
        assert!(!board.is_empty_at(usize::MAX));
        assert_eq!(board.get(9), None);
    }

    #[test]
    fn empty_cells_skip_marked_cells_in_ascending_order() {
        let board = board_with(&[(1, Mark::X), (4, Mark::O)]);
        assert_eq!(
            board.empty_cells().collect::<Vec<_>>(),
            vec![0, 2, 3, 5, 6, 7, 8]
        );
        assert!(!board.is_empty());
        assert!(Board::default().is_empty());
    }

    #[test]
    fn marks_know_their_opponent_and_label() {
        assert_eq!(Mark::X.other(), Mark::O);
        assert_eq!(Mark::O.other(), Mark::X);
        assert_eq!(Mark::X.as_str(), "X");
        assert_eq!(Mark::O.as_str(), "O");
    }
}
