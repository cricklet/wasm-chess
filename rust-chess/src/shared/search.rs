use crate::{
    helpers::{err_result, OptionResult},
    iterative_traversal::TraversalStack,
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
pub enum Evaluation {
    Unknown,
    Centipawns(Player, isize),
    WinInN(Player, usize),
}

impl Default for Evaluation {
    fn default() -> Self {
        Evaluation::Unknown
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
            Evaluation::Unknown => None,
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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct StaticEvaluationReturn {
    // eg for a search of depth 1 from startpos
    //  best_move: e2e4
    //  evaluation: +x
    //  depth: 1
    //  last_move: None
    current_move: Move,
    evaluation: Evaluation,

    // extra data for error checking
    depth: usize,
    previous_move: Option<Move>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct BestMoveReturn {
    best_move: Move,
    evaluation: Evaluation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SearchResult {
    BestMove(BestMoveReturn),
    StaticEvaluation(StaticEvaluationReturn),
    BetaCutoff(Evaluation),
}

impl SearchResult {
    fn score(&self) -> Evaluation {
        match self {
            SearchResult::BestMove(result) => result.evaluation,
            SearchResult::StaticEvaluation(result) => result.evaluation,
            SearchResult::BetaCutoff(e) => *e,
        }
    }
}

// ************************************************************************************************* //

const MAX_ALPHA_BETA_DEPTH: usize = 40;

#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
struct SearchStackData {
    alpha: Evaluation,
    beta: Evaluation,
    in_quiescence: InQuiescence,

    best_move: Option<BestMoveReturn>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LoopResult {
    Continue,
    Done,
}

#[derive(Debug)]
pub struct Search {
    traversal: TraversalStack<SearchStackData, MAX_ALPHA_BETA_DEPTH>,
    returned_evaluation: Option<SearchResult>,
    pub max_depth: usize,
}

impl Search {
    pub fn new(game: Game) -> ErrorResult<Self> {
        Ok(Self {
            traversal: TraversalStack::<SearchStackData, MAX_ALPHA_BETA_DEPTH>::new(game)?,
            returned_evaluation: None,
            max_depth: 3,
        })
    }
    pub fn with_max_depth(game: Game, max_depth: usize) -> ErrorResult<Self> {
        Ok(Self {
            traversal: TraversalStack::<SearchStackData, MAX_ALPHA_BETA_DEPTH>::new(game)?,
            returned_evaluation: None,
            max_depth,
        })
    }

    pub fn bestmove(&self) -> Option<(Move, Evaluation)> {
        match self.returned_evaluation {
            Some(SearchResult::BestMove(result)) => Some((result.best_move, result.evaluation)),
            _ => None,
        }
    }

    pub fn iterate(&mut self) -> ErrorResult<LoopResult> {
        // eg for the first move
        //  previous (startpos) --> last_move (e2e4) --> current (startpos moves e2e4)

        if self.max_depth == 0 {
            return err_result("max_depth must be > 0");
        }

        {
            let (current, current_depth) = self.traversal.current()?;

            if current_depth >= self.max_depth {
                let score = Evaluation::Centipawns(current.game.player, evaluate(&current.game));

                self.returned_evaluation =
                    Some(SearchResult::StaticEvaluation(StaticEvaluationReturn {
                        current_move: self
                            .traversal
                            .move_applied_before_depth(current_depth)?
                            .expect_ok(&format!("invalid move at current_depth {}, {:#?}", current_depth, self))?,
                        evaluation: score,
                        depth: current_depth,
                        previous_move: self
                            .traversal
                            .move_applied_before_depth(current_depth - 1)?,
                    }));

                // Return early (pop up the stack)
                self.traversal.depth -= 1;
                return Ok(LoopResult::Continue);
            }
        }

        {
            // The previously searched child might have a return value. If so, check against alpha/beta & clear the return value.
            let current_depth = self.traversal.current_depth();
            let next_depth = current_depth + 1;

            if let Some(next_evaluation) = self.returned_evaluation {
                self.returned_evaluation = None;

                let next_evaluation = next_evaluation.score();
                let next_move = self
                    .traversal
                    .move_applied_before_depth(next_depth)?
                    .expect_ok(&format!("{:#?}", self))?;

                let (current, _) = self.traversal.current_mut()?;
                if Evaluation::compare(current.game.player, next_evaluation, current.data.beta)
                    .is_better_or_equal()
                {
                    // The enemy can force a better score. Cutoff early.
                    // Beta is the lower bound for the score we can get at this board state.
                    self.returned_evaluation = Some(SearchResult::BetaCutoff(next_evaluation));

                    // Return early (pop up the stack)
                    self.traversal.depth -= 1;
                    return Ok(LoopResult::Continue);
                } else {
                    let current_evaluation = current.data.best_move;
                    if current_evaluation.is_none()
                        || Evaluation::compare(
                            current.game.player,
                            next_evaluation,
                            current_evaluation.unwrap().evaluation,
                        )
                        .is_better()
                    {
                        current.data.best_move = Some(BestMoveReturn {
                            best_move: next_move,
                            evaluation: next_evaluation,
                        });

                        if Evaluation::compare(
                            current.game.player,
                            next_evaluation,
                            current.data.alpha,
                        )
                        .is_better()
                        {
                            // Enemy won't prevent us from making this move. Keep searching
                            current.data.alpha = next_evaluation;
                        }
                    }
                }
            }
        }

        let next_move = self.traversal.get_and_increment_move().unwrap();
        {
            if let Some(next_move) = next_move {
                // If there are moves left at 'current', apply the move
                let (current, next) = self.traversal.current_and_next_mut()?;
                let result = next.setup_from_move(current, &next_move).unwrap();
                if result == Legal::Yes {
                    // Finish setting up the new data
                    next.data.alpha = current.data.beta;
                    next.data.beta = current.data.alpha;
                    next.data.in_quiescence = current.data.in_quiescence;
                    next.data.best_move = None;

                    // Recurse into our newly applied move
                    self.traversal.depth += 1;
                    return Ok(LoopResult::Continue);
                }
            } else {
                // We're out of moves to traverse, return the best move and pop up the stack.
                let (current, _) = self.traversal.current_mut()?;

                if let Some(best_move) = current.data.best_move {
                    self.returned_evaluation = Some(SearchResult::BestMove(best_move));
                } else {
                    todo!("checkmate or draw");
                }

                if self.traversal.depth == 0 {
                    // If there are no more moves at 'current' and we're at the root node, we've exhaustively searched
                    return Ok(LoopResult::Done);
                } else {
                    self.traversal.depth -= 1;
                    return Ok(LoopResult::Continue);
                }
            }
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
    let mut search = Search::with_max_depth(Game::from_fen("startpos").unwrap(), 2).unwrap();
    loop {
        match search.iterate().unwrap() {
            LoopResult::Continue => {}
            LoopResult::Done => break,
        }
    }

    println!("{:#?}", search);
}
