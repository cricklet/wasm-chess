use std::{collections::HashSet, fmt::Display};

use itertools::Itertools;

use crate::{
    defer,
    helpers::{err_result, OptionResult},
    iterative_traversal::{IndexedMoveBuffer, TraversalStack},
};

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
pub enum Score {
    Unknown,
    Centipawns(Player, isize),
    WinInN(Player, usize),
    DrawInN(usize),
}

impl Display for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Score::Unknown => write!(f, "?"),
            Score::Centipawns(player, score) => match player {
                Player::White => write!(f, "{}", score),
                Player::Black => write!(f, "{}", -score),
            },
            Score::WinInN(player, n) => write!(f, "{} wins +{} mate", player.to_fen(), n),
            Score::DrawInN(n) => write!(f, "draw +{}", n),
        }
    }
}

impl Score {
    pub fn increment_turns(self) -> Self {
        let mut new_score = self;
        match new_score {
            Score::WinInN(_, ref mut i) => *i += 1,
            Score::DrawInN(ref mut i) => *i += 1,
            Score::Centipawns(..) => {}
            Score::Unknown => {}
        }
        new_score
    }
}

impl Default for Score {
    fn default() -> Self {
        Score::Unknown
    }
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

impl Score {
    fn comparison_points(&self, current_player: Player) -> Option<(isize, isize)> {
        match self {
            Score::Centipawns(player, score) => {
                if *player == current_player {
                    Some((0, *score))
                } else {
                    Some((0, -*score))
                }
            }
            Score::WinInN(player, n) => {
                if *player == current_player {
                    Some((999999 - *n as isize, 0))
                } else {
                    Some((-99999 + *n as isize, 0))
                }
            }
            Score::DrawInN(_) => Some((0, 0)),
            Score::Unknown => None,
        }
    }

    fn is_draw(&self) -> bool {
        match self {
            Score::DrawInN(_) => true,
            _ => false,
        }
    }

    pub fn compare(current_player: Player, left: Score, right: Score) -> Comparison {
        let left_points = left.comparison_points(current_player);
        let right_points = right.comparison_points(current_player);

        if left_points.is_none() || right_points.is_none() {
            return Comparison::Unknown;
        }
        let (left_mate, mut left_eval) = left_points.unwrap();
        let (right_mate, mut right_eval) = right_points.unwrap();

        if left.is_draw() && !right.is_draw() {
            right_eval += 400;
        }

        if right.is_draw() && !left.is_draw() {
            left_eval += 400;
        }

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
        Score::compare(
            Player::White,
            Score::Centipawns(Player::White, 0),
            Score::Centipawns(Player::White, 0)
        ),
        Comparison::Equal
    );
    assert_eq!(
        Score::compare(
            Player::White,
            Score::Centipawns(Player::White, 100),
            Score::Centipawns(Player::White, 0)
        ),
        Comparison::Better
    );
    assert_eq!(
        Score::compare(
            Player::White,
            Score::Centipawns(Player::White, 100),
            Score::Centipawns(Player::White, 200)
        ),
        Comparison::Worse
    );
    assert_eq!(
        Score::compare(
            Player::White,
            Score::Centipawns(Player::Black, 100),
            Score::Centipawns(Player::White, 0)
        ),
        Comparison::Worse
    );
    assert_eq!(
        Score::compare(
            Player::White,
            Score::Centipawns(Player::Black, -300),
            Score::Centipawns(Player::White, 200)
        ),
        Comparison::Better
    );
    assert_eq!(
        Score::compare(
            Player::White,
            Score::WinInN(Player::White, 0),
            Score::WinInN(Player::White, 1),
        ),
        Comparison::Better
    );
    assert_eq!(
        Score::compare(
            Player::Black,
            Score::WinInN(Player::White, 0),
            Score::WinInN(Player::White, 1),
        ),
        Comparison::Worse
    );
    assert_eq!(
        Score::compare(
            Player::White,
            Score::WinInN(Player::White, 1),
            Score::WinInN(Player::Black, 1),
        ),
        Comparison::Better
    );
    assert_eq!(
        Score::compare(
            Player::Black,
            Score::WinInN(Player::White, 1),
            Score::WinInN(Player::Black, 1),
        ),
        Comparison::Worse
    );

    // Don't prefer a draw if you're only losing by a little
    assert_eq!(
        Score::compare(
            Player::White,
            Score::DrawInN(0),
            Score::Centipawns(Player::Black, 50),
        ),
        Comparison::Worse
    );
    assert_eq!(
        Score::compare(
            Player::White,
            Score::DrawInN(0),
            Score::Centipawns(Player::Black, 500),
        ),
        Comparison::Better
    );
    assert_eq!(
        Score::compare(
            Player::White,
            Score::DrawInN(0),
            Score::Centipawns(Player::White, -500),
        ),
        Comparison::Better
    );

    // Don't let them draw if you're winning
    assert_eq!(
        Score::compare(
            Player::Black,
            Score::Centipawns(Player::Black, 100),
            Score::DrawInN(0),
        ),
        Comparison::Better
    );
    assert_eq!(
        Score::compare(
            Player::Black,
            Score::Centipawns(Player::Black, 500),
            Score::DrawInN(0),
        ),
        Comparison::Better
    );
    assert_eq!(
        Score::compare(
            Player::Black,
            Score::DrawInN(0),
            Score::Centipawns(Player::Black, 500),
        ),
        Comparison::Worse
    );

    // Mates beat draws
    assert_eq!(
        Score::compare(
            Player::Black,
            Score::DrawInN(0),
            Score::WinInN(Player::Black, 1),
        ),
        Comparison::Worse
    );
    assert_eq!(
        Score::compare(
            Player::Black,
            Score::DrawInN(0),
            Score::WinInN(Player::White, 1),
        ),
        Comparison::Better
    );
}

#[test]
fn test_evaluation_increment() {
    // It is better to win sooner
    assert_eq!(
        Score::compare(
            Player::White,
            Score::WinInN(Player::White, 0),
            Score::WinInN(Player::White, 1),
        ),
        Comparison::Better
    );
    assert_eq!(
        Score::compare(
            Player::White,
            Score::WinInN(Player::White, 0).increment_turns(),
            Score::WinInN(Player::White, 1),
        ),
        Comparison::Equal
    );
    assert_eq!(
        Score::compare(
            Player::White,
            Score::WinInN(Player::White, 0)
                .increment_turns()
                .increment_turns(),
            Score::WinInN(Player::White, 1),
        ),
        Comparison::Worse
    );
}
// ************************************************************************************************* //

#[derive(Debug, Eq, PartialEq, Clone)]
struct StaticEvaluationReturn {
    // eg for a search of depth 1 from startpos
    //  previous_move: e2e4
    //  score: +x
    //  depth: 1
    //  previous_previous_move: None
    previous_move: Move,
    score: Score,

    // extra data for error checking
    depth: usize,
    previous_previous_move: Option<Move>,
}

const PV_SIZE: usize = 4;

#[derive(Debug, Eq, PartialEq, Clone)]
struct BestMoveReturn {
    best_move: Move,
    response_moves: Vec<Move>, // store a relatively short PV
    score: Score,
}

#[derive(Debug, Eq, PartialEq, Clone)]
enum SearchResult {
    BestMove(BestMoveReturn),
    StaticEvaluation(StaticEvaluationReturn),
    BetaCutoff(Score),
}

impl SearchResult {
    fn score(&self) -> Score {
        match self {
            SearchResult::BestMove(result) => result.score,
            SearchResult::StaticEvaluation(result) => result.score,
            SearchResult::BetaCutoff(e) => *e,
        }
    }

    fn variation(&self) -> Vec<Move> {
        let mut variation = vec![];
        match self {
            SearchResult::BestMove(result) => {
                variation.push(result.best_move);
                for m in result.response_moves.iter() {
                    variation.push(*m);
                }
            }
            SearchResult::StaticEvaluation(_) => {}
            SearchResult::BetaCutoff(_) => {}
        }
        variation
    }
}

// ************************************************************************************************* //

const MAX_ALPHA_BETA_DEPTH: usize = 40;

#[derive(Default, Debug, Eq, PartialEq)]
struct SearchFrameData {
    alpha: Score,
    beta: Score,
    in_quiescence: InQuiescence,

    best_move: Option<BestMoveReturn>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LoopResult {
    Continue,
    Done,
}

#[derive(Debug)]
pub struct SearchStack {
    traversal: TraversalStack<SearchFrameData, MAX_ALPHA_BETA_DEPTH>,
    returned_evaluation: Option<SearchResult>,

    pub done: bool,
    pub max_depth: usize,
}

impl SearchStack {
    pub fn new(game: Game) -> ErrorResult<Self> {
        Ok(Self {
            traversal: TraversalStack::<SearchFrameData, MAX_ALPHA_BETA_DEPTH>::new(game, || {
                SearchFrameData::default()
            })?,
            returned_evaluation: None,
            max_depth: 3,
            done: false,
        })
    }
    pub fn with_max_depth(game: Game, max_depth: usize) -> ErrorResult<Self> {
        Ok(Self {
            traversal: TraversalStack::<SearchFrameData, MAX_ALPHA_BETA_DEPTH>::new(game, || {
                SearchFrameData::default()
            })?,
            returned_evaluation: None,
            max_depth,
            done: false,
        })
    }

    pub fn bestmove(&self) -> Option<(Move, Vec<Move>, Score)> {
        match self.returned_evaluation.as_ref() {
            Some(SearchResult::BestMove(result)) => Some((
                result.best_move,
                result.response_moves.clone(),
                result.score,
            )),
            _ => None,
        }
    }

    fn statically_evaluate_leaf(&mut self) -> ErrorResult<Option<LoopResult>> {
        let (current, current_depth) = self.traversal.current()?;

        if current_depth < self.max_depth {
            return Ok(None);
        }

        let score = Score::Centipawns(current.game.player, evaluate(&current.game));

        self.returned_evaluation = Some(SearchResult::StaticEvaluation(StaticEvaluationReturn {
            previous_move: self
                .traversal
                .move_applied_before_depth(current_depth)?
                .as_result()?,
            score,
            depth: current_depth,
            previous_previous_move: self
                .traversal
                .move_applied_before_depth(current_depth - 1)?,
        }));

        // Return early (pop up the stack)
        self.traversal.depth -= 1;
        Ok(Some(LoopResult::Continue))
    }

    fn process_next_move_evaluation(&mut self) -> ErrorResult<Option<LoopResult>> {
        // The previously searched child might have a return value. If so, check against alpha/beta & clear the return value.
        let current_depth = self.traversal.current_depth();
        let next_depth = current_depth + 1;

        if self.returned_evaluation.is_none() {
            return Ok(None);
        }

        let next_move = self
            .traversal
            .move_applied_before_depth(next_depth)?
            .expect_ok(|| format!("{:#?}", self))?;

        let next_evaluation = self.returned_evaluation.as_ref().unwrap();
        let next_score = next_evaluation.score().increment_turns();

        let (current, _) = self.traversal.current_mut()?;
        if Score::compare(current.game.player, next_score, current.data.beta).is_better_or_equal() {
            // The enemy can force a better score. Cutoff early.
            // Beta is the lower bound for the score we can get at this board state.
            self.returned_evaluation = Some(SearchResult::BetaCutoff(next_score));

            // Return early (pop up the stack)
            self.traversal.depth -= 1;
            return Ok(Some(LoopResult::Continue));
        }

        if current.data.best_move.is_none()
            || Score::compare(
                current.game.player,
                next_score,
                current.data.best_move.as_ref().unwrap().score,
            )
            .is_better()
        {
            current.data.best_move = Some(BestMoveReturn {
                best_move: next_move,
                score: next_score,
                response_moves: next_evaluation.variation(),
            });

            if Score::compare(current.game.player, next_score, current.data.alpha).is_better() {
                // Enemy won't prevent us from making this move. Keep searching
                current.data.alpha = next_score;
            }
        }

        self.returned_evaluation = None;
        Ok(Some(LoopResult::Continue))
    }

    fn apply_next_move_or_return(&mut self) -> ErrorResult<Option<LoopResult>> {
        let next_move = self.traversal.get_and_increment_move()?;
        if let Some(next_move) = next_move {
            // If there are moves left at 'current', apply the move
            let (current, next) = self.traversal.current_and_next_mut()?;
            let result = next.setup_from_move(current, &next_move).unwrap();

            if result == Legal::No {
                return Ok(Some(LoopResult::Continue));
            }

            // Finish setting up the new data
            next.data.alpha = current.data.beta;
            next.data.beta = current.data.alpha;
            next.data.in_quiescence = current.data.in_quiescence;
            next.data.best_move = None;

            // Recurse into our newly applied move
            self.traversal.depth += 1;
            return Ok(Some(LoopResult::Continue));
        }

        // We're out of moves to traverse, return the best move and pop up the stack.
        let (current, current_depth) = self.traversal.current_mut()?;

        if let Some(best_move) = &current.data.best_move {
            self.returned_evaluation = Some(SearchResult::BestMove(best_move.clone()));
        } else {
            current.lazily_generate_danger()?;

            if current.danger.unwrap().check {
                self.returned_evaluation =
                    Some(SearchResult::StaticEvaluation(StaticEvaluationReturn {
                        score: Score::WinInN(current.game.player.other(), 0),
                        previous_move: self
                            .traversal
                            .move_applied_before_depth(current_depth)?
                            .as_result()?,
                        depth: current_depth,
                        previous_previous_move: self
                            .traversal
                            .move_applied_before_depth(current_depth - 1)?,
                    }));
            } else {
                self.returned_evaluation =
                    Some(SearchResult::StaticEvaluation(StaticEvaluationReturn {
                        score: Score::DrawInN(0),
                        previous_move: self
                            .traversal
                            .move_applied_before_depth(current_depth)?
                            .as_result()?,
                        depth: current_depth,
                        previous_previous_move: self
                            .traversal
                            .move_applied_before_depth(current_depth - 1)?,
                    }));
            }
        }

        if self.traversal.depth == 0 {
            // If there are no more moves at 'current' and we're at the root node, we've exhaustively searched
            self.done = true;
            Ok(Some(LoopResult::Done))
        } else {
            self.traversal.depth -= 1;
            Ok(Some(LoopResult::Continue))
        }
    }

    pub fn iterate(&mut self) -> ErrorResult<LoopResult> {
        // eg for the first move
        //  previous (startpos) --> last_move (e2e4) --> current (startpos moves e2e4)

        if self.max_depth == 0 {
            return err_result("max_depth must be > 0");
        }

        if self.done {
            return Ok(LoopResult::Done);
        }

        // If we're at a leaf, statically evaluate
        if let Some(result) = self.statically_evaluate_leaf()? {
            return Ok(result);
        }

        // If we have a return value from a move we applied, process w.r.t. alpha/beta
        #[cfg(test)]
        let current_return = self.returned_evaluation.clone();

        if let Some(result) = self.process_next_move_evaluation()? {
            #[cfg(test)]
            if current_return == self.returned_evaluation {
                return err_result(
                    "process_next_move_evaluation() did not change the returned_evaluation",
                );
            }

            return Ok(result);
        }

        // Otherwise, continue traversing or return the best move we've found so far
        if let Some(result) = self.apply_next_move_or_return()? {
            return Ok(result);
        }

        return Ok(LoopResult::Continue);
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

impl Default for InQuiescence {
    fn default() -> Self {
        InQuiescence::No
    }
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

#[test]
fn test_start_search() {
    let mut search = SearchStack::with_max_depth(Game::from_fen("startpos").unwrap(), 4).unwrap();
    loop {
        match search.iterate().unwrap() {
            LoopResult::Continue => {}
            LoopResult::Done => break,
        }
    }

    // Calling `iterate()` should be idempotent
    search.iterate().unwrap();
    println!("{:#?}", search.returned_evaluation);

    let potential_first_moves: HashSet<String> = HashSet::from_iter(
        vec!["e2e4", "d2d4"]
            .iter()
            .map(|s| s.to_string())
            .into_iter(),
    );

    match search.returned_evaluation.as_ref().unwrap() {
        SearchResult::BestMove(best_move) => {
            // We play conservatively because we don't support quiescence yet
            assert!(!potential_first_moves.contains(&best_move.best_move.to_uci()));
        }
        _ => panic!("unexpected {:?}", search.returned_evaluation.as_ref()),
    }
}
