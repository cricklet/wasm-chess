/*
To implement iterative deepening, we need a few things:
* SearchStack::search needs to return the PV
* To do that, as we do a regular search, we need to keep track of
the best line at each frame.
* Then, we need some way to sort the moves to prioritize PV moves
*/

use std::{fmt::Display, iter};

use num_format::{Locale, ToFormattedString};

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
    skip_killer_move_sort: bool,
    skip_aspiration_window: bool,
    skip_null_move_pruning: bool,
}

impl Display for IterativeSearchOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut options: Vec<String> = vec![];
        if !self.skip_quiescence {
            options.push("quiescence".to_string());
        }
        if !self.skip_cache_sort {
            options.push("cache_sort".to_string());
        }
        if !self.skip_capture_sort {
            options.push("capture_sort".to_string());
        }
        if !self.skip_killer_move_sort {
            options.push("killer_move_sort".to_string());
        }
        if !self.skip_null_move_pruning {
            options.push("null_move_pruning".to_string());
        }
        if !self.skip_aspiration_window {
            options.push("aspiration_window".to_string());
        }
        write!(f, "{{ {} }}", options.join_vec(", "))
    }
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
            skip_killer_move_sort: options.skip_killer_move_sort,
            skip_null_move_pruning: options.skip_null_move_pruning,

            aspiration_window: None,
            transposition_table: None,
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

        let sorter = move |game: &Game, moves: &mut [Move]| -> ErrorResult<()> {
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
                            bestmove.to_string(),
                            response.iter().map(|m| m.to_string()).collect::<Vec<_>>().join_vec(" "),
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

    // mid-game fen
    let fen = "r3k2r/1bq1bppp/pp2p3/2p1n3/P3PP2/2PBN3/1P1BQ1PP/R4RK1 b kq - 0 16";
    let max_depth = 6;

    // // late-game fen
    // let fen = "6k1/8/4p3/3r4/5n2/1Q6/1K1R4/8 w";
    // let max_depth = 8;

    // make sure any lazy-statics are generated
    IterativeSearch::new(
        Game::from_fen(fen).unwrap(),
        IterativeSearchOptions::default(),
    )
    .unwrap()
    .iterate(&mut |_| {})
    .unwrap();

    let mut results: Vec<(u128, String)> = vec![];

    let skip_all = IterativeSearchOptions {
        skip_aspiration_window: true,
        skip_cache_sort: true,
        skip_capture_sort: true,
        skip_killer_move_sort: true,
        skip_null_move_pruning: true,
        ..IterativeSearchOptions::default()
    };

    let options_to_try = vec![
        IterativeSearchOptions::default(),
        IterativeSearchOptions {
            skip_aspiration_window: false,
            ..skip_all.clone()
        },
        IterativeSearchOptions {
            skip_cache_sort: false,
            ..skip_all.clone()
        },
        IterativeSearchOptions {
            skip_capture_sort: false,
            ..skip_all.clone()
        },
        IterativeSearchOptions {
            skip_killer_move_sort: false,
            ..skip_all.clone()
        },
        IterativeSearchOptions {
            skip_null_move_pruning: false,
            ..skip_all.clone()
        },
        skip_all.clone(),
    ];

    println!("");

    for options in options_to_try.iter() {
        let mut search =
            IterativeSearch::new(Game::from_fen(fen).unwrap(), options.clone()).unwrap();

        println!("{}", options);

        let start_time = std::time::Instant::now();

        let mut last_log_time = std::time::Instant::now();
        let mut log_callback = |line: &str| {
            println!(
                "{:>5} ms {}",
                last_log_time
                    .elapsed()
                    .as_millis()
                    .to_formatted_string(&Locale::en),
                line.to_string()
            );
            last_log_time = std::time::Instant::now();
        };

        loop {
            search.iterate(&mut log_callback).unwrap();
            if search.alpha_beta.evaluate_at_depth >= max_depth {
                break;
            }
            // if start_time.elapsed() > std::time::Duration::from_secs(5) {
            //     break;
            // }
        }

        let total_time = start_time.elapsed();
        println!(
            "{:>5} ms total",
            total_time.as_millis().to_formatted_string(&Locale::en)
        );
        println!("");

        results.push((total_time.as_millis(), options.to_string()));
    }

    results.sort_by(|a, b| a.0.cmp(&b.0));
    for (time, options) in results {
        println!(
            "{:>5} ms {}",
            time.to_formatted_string(&Locale::en),
            options
        );
    }
}

#[test]
fn test_aspiration_window_deepening_should_give_pv() {
    let fen = "r3k2r/1bq1bppp/pp2p3/2p1n3/P3PP2/2PBN3/1P1BQ1PP/R4RK1 b kq - 0 16";

    let options = IterativeSearchOptions {
        skip_aspiration_window: false,
        ..IterativeSearchOptions::default()
    };

    let mut search = IterativeSearch::new(Game::from_fen(fen).unwrap(), options.clone()).unwrap();

    let start_time = std::time::Instant::now();
    println!("{}", options);

    let mut log: Vec<String> = vec![];
    let mut last_log_time = std::time::Instant::now();
    let mut log_callback = |line: &str| {
        println!(
            "{} ms {}",
            last_log_time.elapsed().as_millis(),
            line.to_string()
        );
        last_log_time = std::time::Instant::now();
    };

    loop {
        search.iterate(&mut log_callback).unwrap();
        if search.alpha_beta.evaluate_at_depth >= 4 {
            break;
        }
    }

    let total_time = start_time.elapsed();
    log.push(format!("{} ms total\n", total_time.as_millis(),));
}
