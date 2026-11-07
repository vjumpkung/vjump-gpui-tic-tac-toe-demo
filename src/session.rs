//! A play session: the board, whose turn it is and the running score.

use crate::bot;
use crate::game::{Board, Mark, Outcome};

/// Who plays against whom.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// The human plays X against the bot, which plays O.
    OnePlayer,
    /// Two humans take turns on the same screen.
    TwoPlayers,
}

/// Finished rounds since the session started or the mode last changed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Score {
    x_wins: u32,
    o_wins: u32,
    draws: u32,
}

impl Score {
    pub fn x_wins(&self) -> u32 {
        self.x_wins
    }

    pub fn o_wins(&self) -> u32 {
        self.o_wins
    }

    pub fn draws(&self) -> u32 {
        self.draws
    }

    fn record(&mut self, outcome: Outcome) {
        match outcome {
            Outcome::Won { mark: Mark::X, .. } => self.x_wins += 1,
            Outcome::Won { mark: Mark::O, .. } => self.o_wins += 1,
            Outcome::Draw => self.draws += 1,
            Outcome::InProgress => {}
        }
    }
}

/// What the status line reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Turn(Mark),
    Won(Mark),
    Draw,
}

/// The state of play. All mutation goes through the methods below, which keep
/// the board, turn, outcome and score consistent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Session {
    mode: Mode,
    board: Board,
    turn: Mark,
    outcome: Outcome,
    score: Score,
}

impl Session {
    /// An empty board with X to move and a zero score.
    pub fn new(mode: Mode) -> Self {
        Self {
            mode,
            board: Board::default(),
            turn: Mark::X,
            outcome: Outcome::InProgress,
            score: Score::default(),
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn outcome(&self) -> Outcome {
        self.outcome
    }

    pub fn score(&self) -> Score {
        self.score
    }

    pub fn status(&self) -> Status {
        match self.outcome {
            Outcome::InProgress => Status::Turn(self.turn),
            Outcome::Won { mark, .. } => Status::Won(mark),
            Outcome::Draw => Status::Draw,
        }
    }

    /// Switches mode, starting over with an empty board and a zero score.
    /// Returns false, changing nothing, when `mode` is already active.
    pub fn set_mode(&mut self, mode: Mode) -> bool {
        if self.mode == mode {
            return false;
        }
        *self = Self::new(mode);
        true
    }

    /// Clears the board for another round, keeping the score and letting X
    /// start. Returns false, changing nothing, when the board is already empty.
    pub fn new_round(&mut self) -> bool {
        if self.board.is_empty() {
            return false;
        }
        self.board = Board::default();
        self.turn = Mark::X;
        self.outcome = Outcome::InProgress;
        true
    }

    /// Whether the human at the screen may mark cell `ix` right now.
    pub fn is_playable(&self, ix: usize) -> bool {
        ix < 9
            && self.board.is_empty_at(ix)
            && self.outcome == Outcome::InProgress
            && (self.mode == Mode::TwoPlayers || self.turn == Mark::X)
    }

    /// Marks cell `ix` for the player to move. In one-player mode the bot
    /// answers in the same call unless the round just ended. Returns false,
    /// changing nothing, when the cell is not playable.
    pub fn play(&mut self, ix: usize) -> bool {
        if !self.is_playable(ix) {
            return false;
        }
        self.place(ix);
        if self.mode == Mode::OnePlayer
            && self.outcome == Outcome::InProgress
            && let Some(reply_ix) = bot::best_move(&self.board, Mark::O)
        {
            self.place(reply_ix);
        }
        true
    }

    /// Marks `ix` for the player to move, then scores a finished round once
    /// and hands the turn over.
    fn place(&mut self, ix: usize) {
        self.board.set(ix, self.turn);
        self.outcome = self.board.outcome();
        self.score.record(self.outcome);
        self.turn = self.turn.other();
    }
}

#[cfg(test)]
mod tests {
    use super::{Mode, Session, Status};
    use crate::game::{Mark, Outcome};

    fn marks_on_board(session: &Session, mark: Mark) -> usize {
        (0..9)
            .filter(|&ix| session.board().get(ix) == Some(mark))
            .count()
    }

    #[test]
    fn play_rejects_out_of_range_occupied_and_finished_cells() {
        let mut session = Session::new(Mode::TwoPlayers);
        assert!(!session.play(9));
        assert!(!session.play(usize::MAX));
        assert_eq!(session, Session::new(Mode::TwoPlayers));

        assert!(session.play(4));
        let after_first_move = session;
        assert!(!session.play(4));
        assert_eq!(session, after_first_move);

        // X takes the top row while O plays 3 and 4... then the round is over.
        let mut session = Session::new(Mode::TwoPlayers);
        for ix in [0, 3, 1, 4, 2] {
            assert!(session.play(ix));
        }
        assert!(matches!(
            session.outcome(),
            Outcome::Won {
                mark: Mark::X,
                line: [0, 1, 2]
            }
        ));
        let finished = session;
        assert!(!session.play(8));
        assert_eq!(session, finished);
    }

    #[test]
    fn two_players_alternate_x_then_o() {
        let mut session = Session::new(Mode::TwoPlayers);
        assert_eq!(session.status(), Status::Turn(Mark::X));
        assert!(session.play(0));
        assert_eq!(session.board().get(0), Some(Mark::X));
        assert_eq!(session.status(), Status::Turn(Mark::O));
        assert!(session.play(1));
        assert_eq!(session.board().get(1), Some(Mark::O));
        assert_eq!(session.status(), Status::Turn(Mark::X));
    }

    #[test]
    fn one_player_gets_exactly_one_bot_reply_per_move() {
        let mut session = Session::new(Mode::OnePlayer);
        assert!(session.play(4));
        assert_eq!(session.board().get(4), Some(Mark::X));
        assert_eq!(marks_on_board(&session, Mark::X), 1);
        assert_eq!(marks_on_board(&session, Mark::O), 1);
        assert_eq!(session.status(), Status::Turn(Mark::X));
        assert!(
            session.is_playable(
                session
                    .board()
                    .empty_cells()
                    .next()
                    .expect("the board has empty cells")
            )
        );
    }

    #[test]
    fn score_counts_each_finished_round_once() {
        let mut session = Session::new(Mode::TwoPlayers);
        for ix in [0, 3, 1, 4, 2] {
            session.play(ix);
        }
        assert_eq!(session.status(), Status::Won(Mark::X));
        assert_eq!(session.score().x_wins(), 1);
        assert_eq!(session.score().o_wins(), 0);
        assert_eq!(session.score().draws(), 0);

        // Rejected moves after the round ended must not score again.
        assert!(!session.play(8));
        assert_eq!(session.score().x_wins(), 1);

        assert!(session.new_round());
        // O wins the next round: X 0, O 3, X 1, O 4, X 8, O 5.
        for ix in [0, 3, 1, 4, 8, 5] {
            session.play(ix);
        }
        assert_eq!(session.status(), Status::Won(Mark::O));
        assert_eq!(session.score().x_wins(), 1);
        assert_eq!(session.score().o_wins(), 1);

        assert!(session.new_round());
        // A draw: X O X / X O O / O X X in play order.
        for ix in [0, 1, 2, 4, 3, 5, 7, 6, 8] {
            session.play(ix);
        }
        assert_eq!(session.status(), Status::Draw);
        assert_eq!(session.score().draws(), 1);
        assert_eq!(session.score().x_wins(), 1);
        assert_eq!(session.score().o_wins(), 1);
    }

    #[test]
    fn new_round_keeps_the_score_and_is_a_no_op_on_an_empty_board() {
        let mut session = Session::new(Mode::TwoPlayers);
        assert!(!session.new_round());

        for ix in [0, 3, 1, 4, 2] {
            session.play(ix);
        }
        assert!(session.new_round());
        assert!(session.board().is_empty());
        assert_eq!(session.status(), Status::Turn(Mark::X));
        assert_eq!(session.score().x_wins(), 1);
        assert!(!session.new_round());
    }

    #[test]
    fn set_mode_resets_only_when_the_mode_changes() {
        let mut session = Session::new(Mode::TwoPlayers);
        session.play(0);
        let before = session;
        assert!(!session.set_mode(Mode::TwoPlayers));
        assert_eq!(session, before);

        for ix in [3, 1, 4, 2] {
            session.play(ix);
        }
        assert_eq!(session.score().x_wins(), 1);
        assert!(session.set_mode(Mode::OnePlayer));
        assert_eq!(session, Session::new(Mode::OnePlayer));
    }

    #[test]
    fn one_player_mode_never_lets_the_human_play_o() {
        let session = Session::new(Mode::OnePlayer);
        assert!((0..9).all(|ix| session.is_playable(ix)));
        assert!(!session.is_playable(9));
    }

    /// Plays every possible sequence of human (X) moves against the bot and
    /// checks that the bot never loses and every round is scored once.
    fn explore(session: &Session, finished_rounds: &mut u32) {
        for ix in 0..9 {
            if !session.is_playable(ix) {
                continue;
            }
            let mut next = *session;
            assert!(next.play(ix));
            match next.outcome() {
                Outcome::InProgress => explore(&next, finished_rounds),
                Outcome::Won {
                    mark: Mark::X,
                    line,
                } => panic!("the bot lost on line {line:?} after human move {ix}"),
                Outcome::Won { .. } | Outcome::Draw => {
                    let score = next.score();
                    assert_eq!(score.x_wins(), 0);
                    assert_eq!(score.o_wins() + score.draws(), 1);
                    *finished_rounds += 1;
                }
            }
        }
    }

    #[test]
    fn the_bot_never_loses_against_any_human_play() {
        let mut finished_rounds = 0;
        explore(&Session::new(Mode::OnePlayer), &mut finished_rounds);
        assert!(finished_rounds > 0);
    }
}
