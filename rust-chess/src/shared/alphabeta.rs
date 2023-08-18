use std::{collections::HashSet, fmt::Display, ptr::null};

use itertools::Itertools;

use crate::{
    defer,
    helpers::{err_result, Joinable, OptionResult},
    iterative_traversal::{null_move_sort, TraversalStack},
};

use super::{
    danger::Danger,
    evaluation::*,
    game::{Game, Legal},
    helpers::ErrorResult,
    moves::*,
    types::*,
};

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Score {
    Centipawns(Player, isize),
    WinInN(Player, usize),
    DrawInN(usize),
}

impl Default for Score {
    fn default() -> Self {
        Score::Centipawns(Player::White, 0)
    }
}

impl Display for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Score::Centipawns(player, score) => match player {
                Player::White => write!(f, "{}", score),
                Player::Black => write!(f, "{}", -score),
            },
            Score::WinInN(player, n) => write!(f, "{} wins +{} mate", player.to_fen(), n),
            Score::DrawInN(n) => write!(f, "draw +{}", n),
        }
    }
}

impl std::fmt::Debug for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Score {
    pub fn increment_turns(self) -> Self {
        let mut new_score = self;
        match new_score {
            Score::WinInN(_, ref mut i) => *i += 1,
            Score::DrawInN(ref mut i) => *i += 1,
            Score::Centipawns(..) => {}
        }
        new_score
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
    score: Score,
}

const PV_SIZE: usize = 4;

#[derive(Eq, PartialEq, Clone)]
struct BestMoveReturn {
    best_move: Move,
    response_moves: Vec<Move>, // store a relatively short PV
    score: Score,
}

impl std::fmt::Debug for BestMoveReturn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} ({})",
            self.best_move,
            self.response_moves.join_vec(" "),
            self.score
        )
    }
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

#[derive(Default, Debug, Eq, PartialEq)]
struct AlphaBetaFrame {
    alpha: Score,
    beta: Score,
    in_quiescence: InQuiescence,

    best_move: Option<BestMoveReturn>,
}

impl AlphaBetaFrame {
    pub fn for_player(player: Player) -> Self {
        Self {
            alpha: Score::WinInN(player.other(), 0),
            beta: Score::WinInN(player, 0),
            in_quiescence: InQuiescence::No,
            best_move: None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum LoopResult {
    Continue,
    Done,
}

#[derive(Debug)]
pub struct AlphaBetaStack {
    traversal: TraversalStack<AlphaBetaFrame>,
    best_move: Option<BestMoveReturn>,

    pub done: bool,
    pub max_depth: usize,

    pub skip_quiescence: bool,
    pub log_state_at_history: Option<String>,

    pub num_beta_cutoffs: usize,
    pub num_evaluations: usize,
    pub num_starting_moves_searched: usize,
}

impl AlphaBetaStack {
    pub fn default(game: Game) -> ErrorResult<Self> {
        Self::with(game, 4, false)
    }
    pub fn with(game: Game, max_depth: usize, skip_quiescence: bool) -> ErrorResult<Self> {
        Ok(Self {
            traversal: TraversalStack::<AlphaBetaFrame>::new(
                game,
                AlphaBetaFrame::for_player(game.player()),
            )?,
            best_move: None,
            max_depth,
            done: false,
            skip_quiescence,
            log_state_at_history: None,
            num_beta_cutoffs: 0,
            num_evaluations: 0,
            num_starting_moves_searched: 0,
        })
    }

    pub fn bestmove(&self) -> Option<(Move, Vec<Move>, Score)> {
        match self.best_move.as_ref() {
            Some(result) => Some((
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

        let score = Score::Centipawns(current.game.player(), evaluate(&current.game));

        self.num_evaluations += 1;
        return self.return_early(SearchResult::StaticEvaluation(StaticEvaluationReturn {
            score,
        }));
    }

    fn return_early(&mut self, child_result: SearchResult) -> ErrorResult<Option<LoopResult>> {
        if self.traversal.depth() == 0 {
            self.best_move = match child_result {
                SearchResult::BestMove(result) => Some(result),
                _ => None,
            };
            self.done = true;
            return Ok(Some(LoopResult::Done));
        }

        self.traversal.decrement_depth();
        let child_score = child_result.score().increment_turns();

        let (parent, _) = self.traversal.current_mut()?;
        let parent_to_child_move = parent.recent_move()?.as_result()?;

        if Score::compare(parent.game.player(), child_score, parent.data.beta).is_better_or_equal()
        {
            // The enemy can force a better score. Cutoff early.
            // Beta is the lower bound for the score we can get at this board state.
            self.num_beta_cutoffs += 1;

            return self.return_early(SearchResult::BetaCutoff(child_score));
        }

        if parent.data.best_move.is_none()
            || Score::compare(
                parent.game.player(),
                child_score,
                parent.data.best_move.as_ref().unwrap().score,
            )
            .is_better()
        {
            parent.data.best_move = Some(BestMoveReturn {
                best_move: *parent_to_child_move,
                score: child_score,
                response_moves: child_result.variation(),
            });

            if Score::compare(parent.game.player(), child_score, parent.data.alpha).is_better() {
                // Enemy won't prevent us from making this move. Keep searching
                parent.data.alpha = child_score;
            }
        }

        Ok(Some(LoopResult::Continue))
    }

    fn traverse_next<S>(&mut self, sorter: S) -> ErrorResult<Option<LoopResult>>
    where
        S: Fn(&Game, &mut Vec<Move>) -> ErrorResult<()>,
    {
        let (current, _) = self.traversal.current_mut()?;
        let current_options = current.data.in_quiescence.move_options();
        let current_game = &current.game;
        let current_moves = &mut current.moves;
        let next_move = current_moves.next(current_game, current_options, sorter)?;

        if let Some(next_move) = next_move {
            // If there are moves left at 'current', apply the move
            let (current, next) = self.traversal.current_and_next_mut()?;
            let result = next.setup(current, &next_move).unwrap();

            if result == Legal::No {
                return Ok(Some(LoopResult::Continue));
            }

            // Finish setting up the new data
            next.data.alpha = current.data.beta;
            next.data.beta = current.data.alpha;
            next.data.in_quiescence = current.data.in_quiescence;
            next.data.best_move = None;

            if self.traversal.depth() == 0 {
                self.num_starting_moves_searched += 1;
            }

            // Recurse into our newly applied move
            self.traversal.increment_depth();
            Ok(Some(LoopResult::Continue))
        } else {
            Ok(None)
        }
    }

    pub fn iterate<S>(&mut self, sorter: S) -> ErrorResult<LoopResult>
    where
        S: Fn(&Game, &mut Vec<Move>) -> ErrorResult<()>,
    {
        if self.max_depth == 0 {
            return err_result("max_depth must be > 0");
        }

        if self.done {
            return Ok(LoopResult::Done);
        }

        if let Some(log_state_at_history) = &self.log_state_at_history {
            if &self.traversal.history_string()? == log_state_at_history {
                let (current, _) = self.traversal.current()?;
                println!("logging state for: {}", log_state_at_history);
                println!("{:#?}", current.game);
                println!("{:#?}", self.traversal);
            }
        }

        let in_quiescence = {
            let (current, _) = self.traversal.current()?;
            current.data.in_quiescence == InQuiescence::Yes
        };

        if !in_quiescence {
            let (current, current_depth) = self.traversal.current_mut()?;
            if current_depth >= self.max_depth {
                let current_danger = current.danger()?;
                let current_recent_move = current.history_move.as_ref();
                if self.skip_quiescence || is_quiet_position(&current_danger, current_recent_move) {
                    return Ok(self.statically_evaluate_leaf()?.as_result()?);
                } else {
                    current.data.in_quiescence = InQuiescence::Yes;
                    return Ok(LoopResult::Continue);
                }
            }
        }

        if in_quiescence {
            let (current, _) = self.traversal.current_mut()?;
            let current_danger = current.danger()?;
            let current_game = &current.game;
            let current_player = current_game.player();
            let current_alpha = current.data.alpha;
            let current_beta = current.data.beta;
            if !current_danger.check {
                // Assume we can find a score better than stand-pat
                let stand_pat = Score::Centipawns(current_player, evaluate(current_game));

                if Score::compare(current_player, stand_pat, current_beta).is_better_or_equal() {
                    // The enemy will avoid this line
                    return Ok(self
                        .return_early(SearchResult::BetaCutoff(current_beta))?
                        .as_result()?);
                } else if Score::compare(current_player, stand_pat, current_alpha).is_better() {
                    // We should be able to find a move that is better than stand-pat
                    current.data.alpha = stand_pat;
                }
            }
        }

        // Apply some moves
        if let Some(result) = self.traverse_next(sorter)? {
            return Ok(result);
        }

        // If we're out of moves to traverse, evaluate and return.
        let (current, _) = self.traversal.current_mut()?;
        let result = {
            if let Some(best_move) = current.data.best_move.clone() {
                self.return_early(SearchResult::BestMove(best_move))
            } else {
                if !in_quiescence {
                    let current_enemy = current.game.player().other();
                    if current.danger()?.check {
                        self.return_early(SearchResult::StaticEvaluation(StaticEvaluationReturn {
                            score: Score::WinInN(current_enemy, 0),
                        }))
                    } else {
                        self.return_early(SearchResult::StaticEvaluation(StaticEvaluationReturn {
                            score: Score::DrawInN(0),
                        }))
                    }
                } else {
                    self.statically_evaluate_leaf()
                }
            }
        }?;

        if let Some(result) = result {
            Ok(result)
        } else {
            Ok(LoopResult::Continue)
        }
    }
}

// ************************************************************************************************* //

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InQuiescence {
    No,
    Yes,
}

fn is_quiet_position(danger: &Danger, last_move: Option<&Move>) -> bool {
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
    let mut search = AlphaBetaStack::with(Game::from_fen("startpos").unwrap(), 3, false).unwrap();
    loop {
        match search.iterate(null_move_sort).unwrap() {
            LoopResult::Continue => {}
            LoopResult::Done => break,
        }
    }

    // Calling `iterate()` should be idempotent
    search.iterate(null_move_sort).unwrap();
    println!("{:#?}", search.best_move);

    let potential_first_moves: HashSet<String> = HashSet::from_iter(
        vec!["e2e4", "d2d4"]
            .iter()
            .map(|s| s.to_string())
            .into_iter(),
    );

    let best_move = search.best_move.as_ref().unwrap();
    assert!(potential_first_moves.contains(&best_move.best_move.to_uci()));
}

#[test]
fn test_dont_capture() {
    let fen = "6k1/8/4p3/3r4/5n2/1Q6/1K1R4/8 w";
    let mut search = AlphaBetaStack::with(Game::from_fen(fen).unwrap(), 3, false).unwrap();
    search.log_state_at_history = Some("b3g3 g8f7 d2xd5".to_string());

    loop {
        match search.iterate(null_move_sort).unwrap() {
            LoopResult::Continue => {}
            LoopResult::Done => break,
        }
    }

    // Calling `iterate()` should be idempotent
    search.iterate(null_move_sort).unwrap();
    println!("{:#?}", search.best_move);
}
