/*
To implement iterative deepening, we need a few things:
* SearchStack::search needs to return the PV
* To do that, as we do a regular search, we need to keep track of
the best line at each frame.
* Then, we need some way to sort the moves to prioritize PV moves
*/

use std::iter;

use crate::{
    alphabeta::{AlphaBetaOptions, AlphaBetaStack, LoopResult},
    bitboard::{warm_magic_cache, BoardIndex},
    game::Game,
    helpers::{ErrorResult, Joinable, OptionResult},
    move_ordering::capture_sort,
    moves::Move,
    traversal::null_move_sort,
    zobrist::BestMovesCache,
};

#[derive(Default, Debug, Clone)]
pub struct IterativeSearchOptions {
    skip_quiescence: bool,
    skip_cache_sort: bool,
    skip_capture_sort: bool,
    skip_aspiration_window: bool,
}

pub struct IterativeSearch {
    alpha_beta: AlphaBetaStack,
    start_game: Game,

    best_variations_per_depth: Vec<Vec<Move>>,
    best_moves_cache: BestMovesCache,

    options: IterativeSearchOptions,

    no_moves_found: bool,
}

impl IterativeSearch {
    pub fn new(game: Game, options: IterativeSearchOptions) -> ErrorResult<Self> {
        let best_moves_cache = BestMovesCache::new();
        let search_options = AlphaBetaOptions {
            skip_quiescence: options.skip_quiescence,
            aspiration_window: None,
            log_state_at_history: None,
        };
        let search = AlphaBetaStack::with(game, 1, search_options)?;
        Ok(Self {
            alpha_beta: search,
            start_game: game,
            best_variations_per_depth: vec![],
            best_moves_cache,
            options,
            no_moves_found: false,
        })
    }

    pub fn bestmove(&self) -> Option<(Move, Vec<Move>)> {
        let variation = self.best_variations_per_depth.last();
        match variation {
            None => None,
            Some(variation) => {
                let bestmove = variation[0];
                let response = variation[1..].into_iter().cloned().collect();

                Some((bestmove, response))
            }
        }
    }

    pub fn iterate<F: FnMut(&str)>(&mut self, log: &mut F) -> ErrorResult<()> {
        if self.no_moves_found {
            return Ok(());
        }

        let skip_cache_sort = self.options.skip_cache_sort;
        let skip_capture_sort = self.options.skip_capture_sort;
        let best_moves_cache = &self.best_moves_cache;

        let sorter = move |game: &Game, moves: &mut Vec<Move>| -> ErrorResult<()> {
            let mut unsorted: &mut [Move] = moves;
            if !skip_cache_sort {
                unsorted = best_moves_cache.sort(game, moves)?;
            }
            if !skip_capture_sort {
                capture_sort(unsorted)?;
            }
            Ok(())
        };

        match self.alpha_beta.iterate(sorter)? {
            LoopResult::Done => {
                let result = self.alpha_beta.bestmove();
                match result {
                    None => {
                        if self.alpha_beta.options.aspiration_window.is_some() {
                            log(&format!(
                                "no moves found at depth {} with aspiration window {:?}: trying again without aspiration window",
                                self.alpha_beta.evaluate_at_depth,
                                self.alpha_beta.options.aspiration_window,
                            ));
                            let mut alpha_beta_options = self.alpha_beta.options.clone();
                            alpha_beta_options.aspiration_window = None;

                            self.alpha_beta = AlphaBetaStack::with(
                                self.start_game.clone(),
                                self.alpha_beta.evaluate_at_depth,
                                alpha_beta_options,
                            )?;
                            return Ok(());
                        } else {
                            self.no_moves_found = true;
                            return Ok(());
                        }
                    }
                    Some((bestmove, response, score)) => {
                        let depth = self.alpha_beta.evaluate_at_depth;
                        log(&format!(
                            "at depth {}: bestmove {} ponder {} ({}), beta-cutoffs {}, evaluations {}, start moves searched {}",
                            depth,
                            bestmove.to_uci(),
                            response.iter().map(|m| m.to_uci()).collect::<Vec<_>>().join_vec(" "),
                            score,
                            self.alpha_beta.num_beta_cutoffs,
                            self.alpha_beta.num_evaluations,
                            self.alpha_beta.num_starting_moves_searched,
                        ));

                        let best_variation = iter::once(bestmove).chain(response).collect();
                        self.best_moves_cache
                            .update(&self.start_game, &best_variation)?;
                        self.best_variations_per_depth.push(best_variation);

                        let mut alpha_beta_options = self.alpha_beta.options.clone();
                        alpha_beta_options.aspiration_window =
                            if self.options.skip_aspiration_window {
                                None
                            } else {
                                Some(score.aspiration_window(self.start_game.player()))
                            };

                        self.alpha_beta = AlphaBetaStack::with(
                            self.start_game.clone(),
                            depth + 1,
                            alpha_beta_options,
                        )?;
                    }
                }
            }
            LoopResult::Continue => {}
        }

        Ok(())
    }
}

#[test]
fn test_iterative_deepening_for_depth() {
    // let fen = "startpos";
    // let max_depth = 7;

    // // mid-game fen
    // let fen = "r3k2r/1bq1bppp/pp2p3/2p1n3/P3PP2/2PBN3/1P1BQ1PP/R4RK1 b kq - 0 16";
    // let max_depth = 6;

    // late-game fen
    let fen = "6k1/8/4p3/3r4/5n2/1Q6/1K1R4/8 w";
    let max_depth = 8;

    // make sure any lazy-statics are generated
    IterativeSearch::new(
        Game::from_fen(fen).unwrap(),
        IterativeSearchOptions::default(),
    )
    .unwrap()
    .iterate(&mut |_| {})
    .unwrap();

    let skip_quiescence = false;

    let mut results: Vec<String> = vec![];

    for &skip_capture_sort in [false, true].iter() {
        for &skip_aspiration_window in [false, true].iter() {
            for &skip_cache_sort in [false, true].iter() {
                let options = IterativeSearchOptions {
                    skip_cache_sort,
                    skip_quiescence,
                    skip_aspiration_window,
                    skip_capture_sort,
                };
                let mut search =
                    IterativeSearch::new(Game::from_fen(fen).unwrap(), options.clone()).unwrap();

                let start_time = std::time::Instant::now();

                let mut log: Vec<String> = vec![];
                let mut last_log_time = std::time::Instant::now();
                let mut log_callback = |line: &str| {
                    log.push(format!(
                        "{} ms {}",
                        last_log_time.elapsed().as_millis(),
                        line.to_string()
                    ));
                    last_log_time = std::time::Instant::now();
                };

                loop {
                    search.iterate(&mut log_callback).unwrap();
                    if search.alpha_beta.evaluate_at_depth >= max_depth {
                        break;
                    }
                }

                let total_time = start_time.elapsed();
                log.push(format!("{} ms total", total_time.as_millis(),));

                println!("{:?}", options);
                println!("{:#?}\n", log);

                results.push(format!("{:?} => {} ms", options, total_time.as_millis()));
            }
        }
    }

    println!("{:#?}", results);
}
