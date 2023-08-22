use std::{cell::RefCell, collections::HashSet, fmt::Display, ptr::null, rc::Rc};

use itertools::Itertools;

use crate::{
    defer,
    helpers::{err_result, pad_left, Joinable, OptionResult},
    score::Score,
    transposition_table::{CacheEntry, CacheValue, TranspositionTable},
    traversal::{null_move_sort, TraversalStack, TraversalData},
};

use super::{
    danger::Danger,
    evaluation::*,
    game::{Game, Legal},
    helpers::ErrorResult,
    moves::*,
    types::*,
    zobrist::SimpleMove,
};

// ************************************************************************************************* //

type BestMoveReturn = (Vec<SimpleMove>, Score);

#[derive(Debug, Eq, PartialEq, Clone)]
enum SearchResult {
    // Returned if we pass both beta/alpha cut-offs
    BestMove(BestMoveReturn),

    // Leaf nodes
    StaticEvaluation(Score),

    // Returned if we fail beta cut-off
    BetaCutoff(Score, Option<SimpleMove>),

    // Returned if we fail to improve alpha. We don't have a
    // recommended move in this case because all failed low.
    AlphaMiss(Score),
}

impl Display for SearchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchResult::BestMove((variation, score)) => {
                write!(f, "({}) best {}", score, variation.join_vec(" "))
            }
            SearchResult::StaticEvaluation(e) => write!(f, "({}) static", e),
            SearchResult::BetaCutoff(e, _) => write!(f, "({}) beta cutoff", e),
            SearchResult::AlphaMiss(e) => write!(f, "({}) alpha miss", e),
        }
    }
}

impl SearchResult {
    fn score(&self) -> Score {
        match self {
            SearchResult::BestMove((_, score)) => *score,
            SearchResult::StaticEvaluation(e) => *e,
            SearchResult::BetaCutoff(e, _) => *e,
            SearchResult::AlphaMiss(e) => *e,
        }
    }

    fn variation(&self) -> Option<&Vec<SimpleMove>> {
        match self {
            SearchResult::BestMove((variation, _)) => {
                return Some(variation);
            }
            SearchResult::StaticEvaluation(_) => {}
            SearchResult::BetaCutoff(_, _) => {}
            SearchResult::AlphaMiss(_) => {}
        }
        None
    }

    fn to_cache_value(&self, depth_remaining: usize) -> Option<CacheValue> {
        match self {
            SearchResult::BestMove((variation, score)) => {
                Some(CacheValue::Exact(*score, variation[0]))
            }
            SearchResult::StaticEvaluation(score) => {
                if depth_remaining == 0 {
                    Some(CacheValue::Static(*score))
                } else {
                    None
                }
            }
            SearchResult::BetaCutoff(score, Some(cutoff_move)) => {
                Some(CacheValue::BetaCutoff(*score, *cutoff_move))
            }
            SearchResult::BetaCutoff(_, None) => None,
            SearchResult::AlphaMiss(score) => Some(CacheValue::AlphaMiss(*score)),
        }
    }

    fn from_cache_entry(
        entry: &CacheEntry,
        player: Player,
        alpha: Score,
        beta: Score,
        depth_remaining: usize,
    ) -> ErrorResult<Option<Self>> {
        if depth_remaining > entry.depth_remaining as usize {
            return Ok(None);
        }
        match entry.value {
            CacheValue::Static(score) => {
                if depth_remaining != 0 {
                    return err_result(
                        "static evaluations should only happen if there's 0 depth remaining",
                    );
                }
                Ok(Some(SearchResult::StaticEvaluation(score)))
            }
            CacheValue::Exact(score, best_move) => {
                if Score::compare(player, score, beta).is_better_or_equal() {
                    Ok(Some(Self::BetaCutoff(score, Some(best_move))))
                } else if Score::compare(player, score, alpha).is_better() {
                    Ok(Some(Self::BestMove((vec![best_move], score))))
                } else {
                    Ok(Some(Self::AlphaMiss(score)))
                }
            }
            CacheValue::BetaCutoff(score, cutoff_move) => {
                if Score::compare(player, score, beta).is_better_or_equal() {
                    Ok(Some(Self::BetaCutoff(score, Some(cutoff_move))))
                } else {
                    Ok(None)
                }
            }
            CacheValue::AlphaMiss(score) => {
                if Score::compare(player, score, alpha).is_better() {
                    Ok(None)
                } else {
                    Ok(Some(Self::AlphaMiss(score)))
                }
            }
            #[allow(unreachable_patterns)]
            _ => Ok(None),
        }
    }
}

// ************************************************************************************************* //

#[derive(Default, Debug, Eq, PartialEq)]
struct CachedKillerMoves {
    moves: [Option<SimpleMove>; 2],
}

impl CachedKillerMoves {
    pub fn add(&mut self, m: &SimpleMove) {
        let m = Some(*m);
        if self.moves[0] == m || self.moves[1] == m {
            return;
        }

        self.moves[1] = self.moves[0];
        self.moves[0] = m;
    }

    pub fn sort<'a>(&self, moves: &'a mut [Move]) -> &'a mut [Move] {
        let mut sorted_index = 0;

        for m in self.moves {
            if let Some(SimpleMove { start, end, .. }) = m {
                let matches = |m: &Move| -> bool { m.start_index == start && m.end_index == end };

                if let Some(i) = moves.iter().position(matches) {
                    moves.swap(sorted_index, i);
                    sorted_index += 1;
                }
            }
        }

        &mut moves[sorted_index..]
    }
}

#[derive(Default, Debug, Eq, PartialEq)]
struct HighPriorityMoves {
    moves: Vec<SimpleMove>,
    applied: HashSet<SimpleMove>,
}

impl HighPriorityMoves {
    pub fn add(&mut self, m: &SimpleMove) {
        self.moves.push(*m);
    }
    pub fn done(&self) -> bool {
        self.moves.is_empty()
    }
    pub fn next(&mut self) -> Option<SimpleMove> {
        let m = self.moves.pop();
        if let Some(m) = &m {
            if self.applied.contains(m) {
                return self.next();
            }
            self.applied.insert(*m);
            Some(*m)
        } else {
            None
        }
    }
    pub fn already_applied(&self, m: &SimpleMove) -> bool {
        self.applied.contains(m)
    }
}

#[derive(Default, Debug, Eq, PartialEq)]
struct AlphaBetaFrame {
    alpha: Score,
    beta: Score,
    in_quiescence: InQuiescence,

    alpha_move: Option<BestMoveReturn>,
    found_legal_moves: bool,

    high_priority_moves: HighPriorityMoves,
    cached_killer_moves: CachedKillerMoves,
}

impl TraversalData for AlphaBetaFrame {
    fn setup(&mut self, previous: &Self) {
        self.alpha = previous.beta;
        self.beta = previous.alpha;
        self.in_quiescence = previous.in_quiescence;
        self.alpha_move = None;
        self.found_legal_moves = false;
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum LoopResult {
    Continue,
    Done,
}

#[derive(Debug, Default, Clone)]
pub struct AlphaBetaOptions {
    pub skip_quiescence: bool,
    pub skip_killer_move_sort: bool,
    pub skip_null_move_pruning: bool,
    pub aspiration_window: Option<(Score, Score)>,
    pub transposition_table: Option<Rc<RefCell<TranspositionTable>>>,
    pub log_state_at_history: Option<String>,
}

#[derive(Debug)]
pub struct AlphaBetaStack {
    traversal: TraversalStack<AlphaBetaFrame>,
    best_move: Option<BestMoveReturn>,

    pub done: bool,
    pub evaluate_at_depth: usize,

    pub options: AlphaBetaOptions,

    pub num_beta_cutoffs: usize,
    pub num_evaluations: usize,
    pub num_starting_moves_searched: usize,
}

impl AlphaBetaStack {
    pub fn default(game: Game) -> ErrorResult<Self> {
        Self::with(game, 4, AlphaBetaOptions::default())
    }
    pub fn with(
        game: Game,
        evaluate_at_depth: usize,
        options: AlphaBetaOptions,
    ) -> ErrorResult<Self> {
        let (alpha, beta) = if let Some((alpha, beta)) = options.aspiration_window {
            (alpha, beta)
        } else {
            (
                Score::WinInN(game.player().other(), 0),
                Score::WinInN(game.player(), 0),
            )
        };

        Ok(Self {
            traversal: TraversalStack::<AlphaBetaFrame>::new(
                game,
                AlphaBetaFrame {
                    alpha,
                    beta,
                    in_quiescence: InQuiescence::No,
                    alpha_move: None,
                    found_legal_moves: false,
                    high_priority_moves: HighPriorityMoves::default(),
                    cached_killer_moves: CachedKillerMoves::default(),
                },
            )?,
            best_move: None,
            evaluate_at_depth,
            done: false,
            options,
            num_beta_cutoffs: 0,
            num_evaluations: 0,
            num_starting_moves_searched: 0,
        })
    }

    pub fn bestmove(&self) -> Option<(Vec<SimpleMove>, Score)> {
        match self.best_move.as_ref() {
            Some((variation, score)) => Some((variation.clone(), *score)),
            _ => None,
        }
    }

    fn transposition_table_entry(&self) -> ErrorResult<Option<CacheEntry>> {
        let (current, _) = self.traversal.current()?;
        if let Some(tt) = self.options.transposition_table.as_ref() {
            if let Some(entry) = tt.borrow().get(&current.game) {
                return Ok(Some(*entry));
            }
        }
        Ok(None)
    }

    fn statically_evaluate_leaf(&mut self) -> ErrorResult<Option<LoopResult>> {
        let (current, current_depth) = self.traversal.current()?;

        if current_depth < self.evaluate_at_depth {
            return Ok(None);
        }

        let score = Score::Centipawns(current.game.player(), evaluate(&current.game));

        self.num_evaluations += 1;
        return self.return_early(SearchResult::StaticEvaluation(score));
    }

    fn return_early(&mut self, child_result: SearchResult) -> ErrorResult<Option<LoopResult>> {
        {
            self.log_if_history_matches(|| format!("{}", child_result))?;
        }

        let (in_quiescence, depth_remaining) = {
            let (current, current_depth) = self.traversal.current()?;
            (
                current.data.in_quiescence == InQuiescence::Yes,
                self.depth_remaining(current_depth),
            )
        };

        if !in_quiescence {
            if self.options.transposition_table.is_some() {
                if let Some(cache_value) = child_result.to_cache_value(depth_remaining) {
                    let (current, _) = self.traversal.current()?;

                    let tt = &mut self
                        .options
                        .transposition_table
                        .as_ref()
                        .as_result()?
                        .borrow_mut();
                    tt.update(&current.game, cache_value, depth_remaining)?;
                }
            }
        }

        if self.traversal.depth() == 0 {
            // The root node is trying to return -- we're done
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
        let parent_to_child_move = SimpleMove::from(parent.recent_move()?.as_result()?);

        if Score::compare(parent.game.player(), child_score, parent.data.beta).is_better_or_equal()
        {
            // The enemy can force a better score. Cutoff early.
            // Beta is the lower bound for the score we can get at this board state.
            self.num_beta_cutoffs += 1;

            parent.data.cached_killer_moves.add(&parent_to_child_move);
            return self.return_early(SearchResult::BetaCutoff(
                child_score,
                Some(parent_to_child_move),
            ));
        }

        if Score::compare(parent.game.player(), child_score, parent.data.alpha).is_better() {
            let mut variation = vec![parent_to_child_move];
            if let Some(child_variation) = child_result.variation() {
                variation.extend(child_variation);
            }
            parent.data.alpha_move = Some((variation, child_score));
            parent.data.alpha = child_score;
        }

        Ok(Some(LoopResult::Continue))
    }

    fn traverse_next<S>(&mut self, sorter: S) -> ErrorResult<Option<LoopResult>>
    where
        S: Fn(&Game, &mut [Move]) -> ErrorResult<()>,
    {
        let skip_killer_move_sort = self.options.skip_killer_move_sort;

        let (current, _) = self.traversal.current_mut()?;
        let current_options = current.data.in_quiescence.move_options();
        let current_game = &current.game;
        let current_moves = &mut current.moves;
        let current_killer_moves = &current.data.cached_killer_moves;

        // if !current.data.high_priority_moves.done() {
        //     if let Some(next_move) = current.data.high_priority_moves.next() {
        //         let (current, next) = self.traversal.current_and_next_mut()?;
        //         let result = next.setup(current, next_move).unwrap();

        //         if result == Legal::No {
        //             return Ok(Some(LoopResult::Continue));
        //         }
        //     }
        // }

        let next_move = current_moves.next(current_game, current_options, |game, moves| {
            let moves = if skip_killer_move_sort {
                moves
            } else {
                current_killer_moves.sort(moves)
            };
            sorter(game, moves)
        })?;

        if let Some(next_move) = next_move {
            // If there are moves left at 'current', apply the move
            let (current, next) = self.traversal.current_and_next_mut()?;
            let result = next.setup(current, &next_move).unwrap();

            if result == Legal::No {
                return Ok(Some(LoopResult::Continue));
            }

            current.data.found_legal_moves = true;

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

    fn should_log_history(&self) -> ErrorResult<Option<String>> {
        if let Some(log_state_at_history) = &self.options.log_state_at_history {
            let history = self.traversal.history_display_string()?;
            if history.starts_with(log_state_at_history) {
                return Ok(Some(history));
            }
            let history = self.traversal.history_uci_string()?;
            if history.starts_with(log_state_at_history) {
                return Ok(Some(history));
            }
        }
        Ok(None)
    }

    fn log_if_history_matches<S: FnOnce() -> String>(&self, suffix: S) -> ErrorResult<()> {
        if let Some(history) = &self.should_log_history()? {
            let (current, _) = self.traversal.current()?;
            println!(
                "<{}, {}>   {}   {}",
                pad_left(&current.data.alpha.to_string(), " ", 5),
                pad_left(&current.data.beta.to_string(), " ", 5),
                history,
                suffix(),
            );
        }
        Ok(())
    }

    pub fn depth_remaining(&self, current_depth: usize) -> usize {
        self.evaluate_at_depth - current_depth
    }

    pub fn iterate<S>(&mut self, sorter: S) -> ErrorResult<LoopResult>
    where
        S: Fn(&Game, &mut [Move]) -> ErrorResult<()>,
    {
        if self.evaluate_at_depth == 0 {
            return err_result("max_depth must be > 0");
        }

        if self.done {
            return Ok(LoopResult::Done);
        }

        {
            self.log_if_history_matches(|| "".to_string())?;
        }

        let in_quiescence = {
            let (current, _) = self.traversal.current()?;
            current.data.in_quiescence == InQuiescence::Yes
        };

        if !in_quiescence {
            if let Some(entry) = self.transposition_table_entry()? {
                let (current, _) = self.traversal.current()?;
                if let Some(early_return) = SearchResult::from_cache_entry(
                    &entry,
                    current.game.player(),
                    current.data.alpha,
                    current.data.beta,
                    self.depth_remaining(self.traversal.depth()),
                )? {
                    return Ok(self.return_early(early_return)?.as_result()?);
                }
            }

            let (current, current_depth) = self.traversal.current_mut()?;
            if current_depth >= self.evaluate_at_depth {
                let current_danger = current.danger()?;
                let current_recent_move = current.history_move.as_ref();
                if self.options.skip_quiescence
                    || is_quiet_position(&current_danger, current_recent_move)
                {
                    return Ok(self.statically_evaluate_leaf()?.as_result()?);
                } else {
                    current.data.in_quiescence = InQuiescence::Yes;
                    return Ok(LoopResult::Continue);
                }
            }

            if !self.options.skip_null_move_pruning {
                let (current, _) = self.traversal.current_mut()?;
                let current_danger = current.danger()?;
                let current_game = &current.game;
                let current_player = current_game.player();
                let current_beta = current.data.beta;
                if !current_danger.check {
                    // If the null evaluation is much better than beta, cutoff early
                    let null_move_score =
                        Score::Centipawns(current_player, evaluate(current_game) - 300);

                    if Score::compare(current_player, null_move_score, current_beta)
                        .is_better_or_equal()
                    {
                        return Ok(self
                            .return_early(SearchResult::BetaCutoff(current_beta, None))?
                            .as_result()?);
                    }
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
                        .return_early(SearchResult::BetaCutoff(current_beta, None))?
                        .as_result()?);
                } else if Score::compare(current_player, stand_pat, current_alpha).is_better() {
                    // If no capture is better than stand-pat, we should probably not capture in this situation!
                    // If this were not during quiescence, we would probably find a quiet move that is much better
                    current.data.alpha = stand_pat;
                }
            }
        }

        // Apply some moves
        if let Some(result) = self.traverse_next(sorter)? {
            return Ok(result);
        }

        // If we're out of moves to traverse, evaluate and return.
        let (current, _) = self.traversal.current()?;
        let result = {
            if current.data.found_legal_moves {
                if let Some(alpha_move) = current.data.alpha_move.clone() {
                    self.return_early(SearchResult::BestMove(alpha_move))
                } else {
                    self.return_early(SearchResult::AlphaMiss(current.data.alpha))
                }
            } else if !in_quiescence {
                let current_enemy = current.game.player().other();
                let (current, _) = self.traversal.current_mut()?;
                if current.danger()?.check {
                    self.return_early(SearchResult::StaticEvaluation(Score::WinInN(
                        current_enemy,
                        0,
                    )))
                } else {
                    self.return_early(SearchResult::StaticEvaluation(Score::DrawInN(0)))
                }
            } else {
                self.statically_evaluate_leaf()
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
    let mut search = AlphaBetaStack::with(
        Game::from_fen("startpos").unwrap(),
        3,
        AlphaBetaOptions::default(),
    )
    .unwrap();
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

    let (variation, _) = search.best_move.as_ref().unwrap();
    assert!(potential_first_moves.contains(&variation[0].to_string()));
}

#[test]
fn test_dont_capture() {
    let fen = "6k1/8/4p3/3r4/5n2/1Q6/1K1R4/8 w";
    let mut options = AlphaBetaOptions::default();
    options.log_state_at_history = Some("b3g3 g8f7 d2d5".to_string());
    let mut search = AlphaBetaStack::with(Game::from_fen(fen).unwrap(), 3, options).unwrap();

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

#[test]
fn test_alpha_beta_weird() {
    let fen = "r3k2r/1bq1bppp/pp2p3/2p1n3/P3PP2/2PBN3/1P1BQ1PP/R4RK1 b kq - 0 16";

    let options = AlphaBetaOptions {
        log_state_at_history: Some("nxB-e5d3 Qxn-e2d3".to_string()),
        ..Default::default()
    };
    {
        let mut search =
            AlphaBetaStack::with(Game::from_fen(fen).unwrap(), 2, options.clone()).unwrap();
        loop {
            match search.iterate(null_move_sort).unwrap() {
                LoopResult::Continue => {}
                LoopResult::Done => break,
            }
        }
        println!("{:#?}", search.best_move);
    }
    {
        let mut search =
            AlphaBetaStack::with(Game::from_fen(fen).unwrap(), 3, options.clone()).unwrap();
        loop {
            match search.iterate(null_move_sort).unwrap() {
                LoopResult::Continue => {}
                LoopResult::Done => break,
            }
        }
        println!("{:#?}", search.best_move);
    }
}
