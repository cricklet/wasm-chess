use crate::{helpers::OptionResult, iterative_traversal::TraversalStack};

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SearchResult {
    None,
    BestMove(Evaluation, Move),
    BetaCutoff(Evaluation),
    StaticEvaluation(Evaluation),
}

impl Default for SearchResult {
    fn default() -> Self {
        SearchResult::None
    }
}

impl SearchResult {
    fn score(&self) -> Option<Evaluation> {
        match self {
            SearchResult::None => None,
            SearchResult::BestMove(e, _) => Some(*e),
            SearchResult::BetaCutoff(e) => Some(*e),
            SearchResult::StaticEvaluation(e) => Some(*e),
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

    result: SearchResult, // the 'return' value of this function
}

#[derive(Debug, PartialEq, Eq)]
pub enum LoopResult {
    Continue,
    Done,
}
pub struct Search {
    traversal: TraversalStack<SearchStackData, MAX_ALPHA_BETA_DEPTH>,
    pub max_depth: usize,
}

impl Search {
    pub fn new(game: Game) -> ErrorResult<Self> {
        Ok(Self {
            traversal: TraversalStack::<SearchStackData, MAX_ALPHA_BETA_DEPTH>::new(game)?,
            max_depth: 3,
        })
    }
    pub fn with_max_depth(game: Game, max_depth: usize) -> ErrorResult<Self> {
        Ok(Self {
            traversal: TraversalStack::<SearchStackData, MAX_ALPHA_BETA_DEPTH>::new(game)?,
            max_depth,
        })
    }

    pub fn bestmove(&self) -> Option<(Move, Evaluation)> {
        match self.traversal.root().data.result {
            SearchResult::BestMove(e, m) => Some((m, e)),
            _ => None,
        }
    }

    pub fn iterate(&mut self) -> ErrorResult<LoopResult> {
        {
            // Are we about to search a leaf node?
            let (current, current_depth) = self.traversal.current_mut()?;

            if current_depth >= self.max_depth {
                let score = Evaluation::Centipawns(current.game.player, evaluate(&current.game));
                current.data.result = SearchResult::StaticEvaluation(score);

                // Return early (pop up the stack)
                self.traversal.depth -= 1;
            }
        }

        {
            // The previously searched child might have a return value. If so, check against alpha/beta and clear the value to
            // mark it as consumed.
            let (current, next) = self.traversal.current_and_next_mut()?;

            let next_evaluation = next.data.result;
            next.data.result = SearchResult::None;

            let next_score = next_evaluation.score();
            if let Some(next_score) = next_score {
                let next_move = current
                    .previously_applied_move()
                    .expect_ok("we should only have a next-evaluation if a move has been applied")?;

                if Evaluation::compare(current.game.player, next_score, current.data.beta)
                    .is_better_or_equal()
                {
                    // The enemy can force a better score. Cutoff early.
                    // Beta is the lower bound for the score we can get at this board state.
                    current.data.result = SearchResult::BetaCutoff(current.data.beta);

                    // Return early (pop up the stack)
                    self.traversal.depth -= 1;
                } else {
                    let current_score = current.data.result.score();
                    if current_score.is_none()
                        || Evaluation::compare(
                            current.game.player,
                            next_score,
                            current_score.unwrap(),
                        )
                        .is_better()
                    {
                        current.data.result =
                            SearchResult::BestMove(next_score, next_move.unwrap());

                        if Evaluation::compare(current.game.player, next_score, current.data.alpha)
                            .is_better()
                        {
                            // Enemy won't prevent us from making this move. Keep searching
                            current.data.alpha = next_score;
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
                    next.data.result = SearchResult::None;

                    // Recurse into our newly applied move
                    self.traversal.depth += 1;
                    return Ok(LoopResult::Continue);
                }
            } else if self.traversal.depth == 0 {
                // If there are no more moves at 'current' and we're at the root node, we've exhaustively searched
                return Ok(LoopResult::Done);
            } else {
                // We're out of moves to traverse, pop up the stack. We don't have to specifically 'return' the
                // evaluation because we've already been accumulating it in 'current.data.bestmove'.
                self.traversal.depth -= 1;
            }

            return Ok(LoopResult::Continue);
        }
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
    let mut search = Search::with_max_depth(Game::from_fen("startpos").unwrap(), 4).unwrap();
    loop {
        match search.iterate().unwrap() {
            LoopResult::Continue => {}
            LoopResult::Done => break,
        }
    }
}
