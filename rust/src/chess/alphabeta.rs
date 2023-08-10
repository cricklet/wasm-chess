use strum::IntoEnumIterator;

use super::{
    danger::Danger,
    evaluation::*,
    game::{Game, Legal},
    helpers::ErrorResult,
    moves::*,
    types::*,
};

struct MoveHistory<'h> {
    history: &'h mut Vec<Move>,
}

impl<'h> MoveHistory<'h> {
    pub fn track(history: &'h mut Vec<Move>, m: Move) -> Self {
        history.push(m);
        MoveHistory { history }
    }
}

impl<'h> Drop for MoveHistory<'h> {
    fn drop(&mut self) {
        self.history.pop();
    }
}
// ************************************************************************************************* //

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Evaluation {
    Centipawns(Player, isize),
    WinInN(Player, usize),
    EarlyExit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Comparison {
    Better,
    Equal,
    Worse,
    Unknown,
}

impl Comparison {
    pub fn is_better_or_equal(self) -> bool {
        self == Comparison::Better || self == Comparison::Equal
    }
    pub fn is_better(self) -> bool {
        self == Comparison::Better
    }
}

impl Evaluation {
    fn comparison_points(&self, current_player: Player) -> Option<(isize, isize)> {
        match self {
            Evaluation::Centipawns(player, score) => {
                if *player == current_player {
                    Some((0, *score))
                } else {
                    Some((0, -*score))
                }
            }
            Evaluation::WinInN(player, n) => {
                if *player == current_player {
                    Some((1000 - *n as isize, 0))
                } else {
                    Some((-1000 + *n as isize, 0))
                }
            }
            Evaluation::EarlyExit => None,
        }
    }

    pub fn compare(current_player: Player, left: Evaluation, right: Evaluation) -> Comparison {
        let left_points = left.comparison_points(current_player);
        let right_points = right.comparison_points(current_player);

        if left_points.is_none() || right_points.is_none() {
            return Comparison::Unknown;
        }
        let (left_mate, left_eval) = left_points.unwrap();
        let (right_mate, right_eval) = right_points.unwrap();

        if left_mate > right_mate {
            Comparison::Better
        } else if left_mate < right_mate {
            Comparison::Worse
        } else if left_eval > right_eval {
            Comparison::Better
        } else if left_eval < right_eval {
            Comparison::Worse
        } else {
            Comparison::Equal
        }
    }
}

#[test]
fn test_evaluation_comparison() {
    assert_eq!(
        Evaluation::compare(
            Player::White,
            Evaluation::Centipawns(Player::White, 0),
            Evaluation::Centipawns(Player::White, 0)
        ),
        Comparison::Equal
    );
    assert_eq!(
        Evaluation::compare(
            Player::White,
            Evaluation::Centipawns(Player::White, 100),
            Evaluation::Centipawns(Player::White, 0)
        ),
        Comparison::Better
    );
    assert_eq!(
        Evaluation::compare(
            Player::White,
            Evaluation::Centipawns(Player::White, 100),
            Evaluation::Centipawns(Player::White, 200)
        ),
        Comparison::Worse
    );
    assert_eq!(
        Evaluation::compare(
            Player::White,
            Evaluation::Centipawns(Player::Black, 100),
            Evaluation::Centipawns(Player::White, 0)
        ),
        Comparison::Worse
    );
    assert_eq!(
        Evaluation::compare(
            Player::White,
            Evaluation::Centipawns(Player::Black, -300),
            Evaluation::Centipawns(Player::White, 200)
        ),
        Comparison::Better
    );
    assert_eq!(
        Evaluation::compare(
            Player::White,
            Evaluation::WinInN(Player::White, 0),
            Evaluation::WinInN(Player::White, 1),
        ),
        Comparison::Better
    );
    assert_eq!(
        Evaluation::compare(
            Player::Black,
            Evaluation::WinInN(Player::White, 0),
            Evaluation::WinInN(Player::White, 1),
        ),
        Comparison::Worse
    );
    assert_eq!(
        Evaluation::compare(
            Player::White,
            Evaluation::WinInN(Player::White, 1),
            Evaluation::WinInN(Player::Black, 1),
        ),
        Comparison::Better
    );
    assert_eq!(
        Evaluation::compare(
            Player::Black,
            Evaluation::WinInN(Player::White, 1),
            Evaluation::WinInN(Player::Black, 1),
        ),
        Comparison::Worse
    );
}

// ************************************************************************************************* //

struct AlphaBeta {
    pub max_depth: usize,
    pub move_history: Vec<Move>,
}

impl AlphaBeta {
    pub fn alpha_beta(
        &mut self,
        game: &Game,
        alpha: Evaluation,
        beta: Evaluation,
        ply: usize,
        in_quiescence: InQuiescence,
    ) -> ErrorResult<Evaluation> {
        let mut alpha = alpha;
        let mut best_score: Option<Evaluation> = None;

        let player = game.player;
        let danger = Danger::from(player, &game.board)?;

        if in_quiescence == InQuiescence::No && ply >= self.max_depth {
            if self.is_quiet_position(&danger, self.move_history.last()) {
                return Ok(Evaluation::Centipawns(player, evaluate(game, player)));
            } else {
                return self.alpha_beta(game, alpha, beta, ply, InQuiescence::Yes);
            }
        }

        if in_quiescence == InQuiescence::Yes {
            if danger.check {
                // assume we can find a score better than stand-pat
                let stand_pat = Evaluation::Centipawns(player, evaluate(game, player));
                if Evaluation::compare(player, stand_pat, beta).is_better_or_equal() {
                    // the enemy will avoid this line
                    return Ok(beta);
                } else if Evaluation::compare(player, stand_pat, alpha).is_better() {
                    // we should be able to find a move that is better than stand-pat
                    best_score = Some(stand_pat);
                }
            }
        }

        let mut moves = MoveBuffer::default();
        game.fill_pseudo_move_buffer(&mut moves, in_quiescence.move_options())?;

        for &m in moves.iter() {
            let mut next_game = game.clone();
            next_game.make_move(m)?;

            if next_game.move_legality(&m, &danger) == Legal::No {
                continue;
            }

            let _ = MoveHistory::track(&mut self.move_history, m);

            let score = self.alpha_beta(&next_game, beta, alpha, ply + 1, in_quiescence)?;

            if Evaluation::compare(player, score, beta).is_better_or_equal() {
                // enemy is can force a better score. cutoff early.
                // beta is the lower bound for the score we can get at this board state.
                return Ok(beta);
            }

            if best_score.is_none()
                || Evaluation::compare(player, score, best_score.unwrap()).is_better()
            {
                best_score = Some(score);
                if Evaluation::compare(player, best_score.unwrap(), alpha).is_better() {
                    // enemy won't prevent us from making this move. keep searching.
                    alpha = best_score.unwrap();
                }
            }
        }

        if best_score.is_none() {
            // no legal moves
            if danger.check {
                // lost to checkmate
                return Ok(Evaluation::WinInN(player.other(), 0));
            } else {
                // stalemate
                return Ok(Evaluation::Centipawns(player, 0));
            }
        }

        return Ok(best_score.unwrap_or(alpha));
    }

    fn is_quiet_position(&self, danger: &Danger, last_move: Option<&Move>) -> bool {
        if danger.check {
            return false;
        }

        if let Some(last_move) = last_move {
            if !last_move.is_quiet() {
                return false;
            }
        }

        true
    }
}

// ************************************************************************************************* //

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InQuiescence {
    No,
    Yes,
}

impl InQuiescence {
    fn move_options(self) -> MoveOptions {
        match self {
            InQuiescence::No => MoveOptions {
                only_captures: OnlyCaptures::No,
                only_queen_promotion: OnlyQueenPromotion::No,
            },
            InQuiescence::Yes => MoveOptions {
                only_captures: OnlyCaptures::Yes,
                only_queen_promotion: OnlyQueenPromotion::No,
            },
        }
    }
}
