use strum::IntoEnumIterator;

use crate::{
    danger::Danger,
    evaluation::{centipawn_evaluation, development_evaluation},
    game::Game,
    helpers::ErrorResult,
    moves::*,
    types::*,
};

struct AlphaBeta {
    pub evaluator: Box<dyn Evaluator>,
    pub move_generator: Box<dyn MoveGenerator>,
    max_depth: usize,
}

impl AlphaBeta {
    pub fn alpha_beta(
        &self,
        game: &Game,
        alpha: isize,
        beta: isize,
        ply: usize,
    ) -> ErrorResult<isize> {
        let mut best_score: Option<isize> = None;

        if ply >= self.max_depth {
            return self.evaluator.evaluate(game);
        }

        for m in self.move_generator.moves(game, ply) {
            let (next_game, _) = m?;
            let score = -self.alpha_beta(&next_game, -beta, -alpha, ply + 1)?;
            if score >= beta {
                // enemy is can force a better score. cutoff early.
                // beta is the lower bound for the score we can get at this board state.
                return Ok(beta);
            }
            if score > best_score.unwrap_or(alpha) {
                // enemy won't prevent us from making this move. keep searching.
                best_score = Some(score);
            }
        }

        // if we have a best score, that's 100% the best score we can get from a move at this board state.
        // if we never found a best score, alpha is the upper bound for the score we can get at this board state.
        return Ok(best_score.unwrap_or(alpha));
    }
}

// ************************************************************************************************* //

trait Evaluator {
    fn evaluate(&self, game: &Game) -> ErrorResult<isize>;
}

struct PointEvaluator {}

impl Evaluator for PointEvaluator {
    fn evaluate(&self, game: &Game) -> ErrorResult<isize> {
        Ok(centipawn_evaluation(game) + development_evaluation(game))
    }
}

// ************************************************************************************************* //

trait MoveGenerator {
    fn moves<'t>(
        &self,
        game: &'t Game,
        ply: usize,
    ) -> Box<dyn Iterator<Item = ErrorResult<(Game, Move)>> + 't>;
}

struct AllMovesGenerator {}

impl MoveGenerator for AllMovesGenerator {
    fn moves<'t>(
        &self,
        game: &'t Game,
        _: usize,
    ) -> Box<dyn Iterator<Item = ErrorResult<(Game, Move)>> + 't> {
        game.for_each_legal_move()
    }
}
