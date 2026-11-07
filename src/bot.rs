//! The computer opponent: an exhaustive minimax search that never loses.

use crate::game::{Board, Mark, Outcome};

/// The best cell for `bot` to play, or `None` when the round is already over.
///
/// A win scores `10 - depth` and a loss `depth - 10`, so the bot prefers the
/// fastest win and the slowest loss. Ties keep the lowest index, which makes
/// the bot deterministic.
pub fn best_move(board: &Board, bot: Mark) -> Option<usize> {
    if board.outcome() != Outcome::InProgress {
        return None;
    }
    let mut best: Option<(usize, i32)> = None;
    for ix in board.empty_cells() {
        let mut next = *board;
        next.set(ix, bot);
        let score = score_position(&next, bot, bot.other(), 1);
        if best.is_none_or(|(_, best_score)| score > best_score) {
            best = Some((ix, score));
        }
    }
    best.map(|(ix, _)| ix)
}

/// Scores `board` from the bot's point of view with `to_move` about to play.
fn score_position(board: &Board, bot: Mark, to_move: Mark, depth: i32) -> i32 {
    match board.outcome() {
        Outcome::Won { mark, .. } if mark == bot => 10 - depth,
        Outcome::Won { .. } => depth - 10,
        Outcome::Draw => 0,
        Outcome::InProgress => {
            let scores = board.empty_cells().map(|ix| {
                let mut next = *board;
                next.set(ix, to_move);
                score_position(&next, bot, to_move.other(), depth + 1)
            });
            let best = if to_move == bot {
                scores.max()
            } else {
                scores.min()
            };
            best.expect("an in-progress board has an empty cell")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::best_move;
    use crate::game::{Board, Mark};

    fn board_with(marks: &[(usize, Mark)]) -> Board {
        let mut board = Board::default();
        for &(ix, mark) in marks {
            board.set(ix, mark);
        }
        board
    }

    #[test]
    fn returns_none_when_the_round_is_over() {
        let won = board_with(&[(0, Mark::X), (1, Mark::X), (2, Mark::X)]);
        assert_eq!(best_move(&won, Mark::O), None);

        let full = board_with(&[
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
        assert_eq!(best_move(&full, Mark::O), None);
    }

    #[test]
    fn takes_an_immediate_win() {
        // O can finish the middle row at 5; X threatens the top row at 2.
        let board = board_with(&[
            (0, Mark::X),
            (1, Mark::X),
            (3, Mark::O),
            (4, Mark::O),
            (8, Mark::X),
        ]);
        assert_eq!(best_move(&board, Mark::O), Some(5));
    }

    #[test]
    fn blocks_an_immediate_loss() {
        // X threatens the top row at 2 and O has no winning move of its own.
        let board = board_with(&[(0, Mark::X), (1, Mark::X), (4, Mark::O)]);
        assert_eq!(best_move(&board, Mark::O), Some(2));
    }

    #[test]
    fn breaks_ties_with_the_lowest_index() {
        // Every reply to a lone corner except the centre loses against best
        // play, so the bot must pick the centre; a lone centre is answered by
        // the lowest-index corner.
        let corner = board_with(&[(0, Mark::X)]);
        assert_eq!(best_move(&corner, Mark::O), Some(4));
        let centre = board_with(&[(4, Mark::X)]);
        assert_eq!(best_move(&centre, Mark::O), Some(0));
    }
}
