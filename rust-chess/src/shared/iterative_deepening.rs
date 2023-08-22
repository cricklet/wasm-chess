/*
To implement iterative deepening, we need a few things:
* SearchStack::search needs to return the PV
* To do that, as we do a regular search, we need to keep track of
the best line at each frame.
* Then, we need some way to sort the moves to prioritize PV moves
*/

use std::{cell::RefCell, fmt::Display, iter, rc::Rc};

use num_format::{Locale, ToFormattedString};

use crate::{
    alphabeta::{AlphaBetaOptions, AlphaBetaStack, LoopResult},
    bitboard::BoardIndex,
    game::Game,
    helpers::{ErrorResult, Joinable, OptionResult},
    move_ordering::capture_sort,
    moves::Move,
    transposition_table::TranspositionTable,
    traversal::null_move_sort,
    zobrist::{BestMovesCache, SimpleMove},
};

#[derive(Debug, Clone)]
pub struct IterativeSearchOptions {
    pub skip_quiescence: bool,
    pub skip_cache_sort: bool,
    pub skip_capture_sort: bool,
    pub skip_sibling_beta_cutoff_sort: bool,
    pub skip_aspiration_window: bool,
    pub skip_null_move_pruning: bool,
    pub transposition_table: Option<Rc<RefCell<TranspositionTable>>>,
}

impl Default for IterativeSearchOptions {
    fn default() -> Self {
        Self {
            skip_quiescence: false,
            skip_cache_sort: false,
            skip_capture_sort: false,
            skip_sibling_beta_cutoff_sort: false,
            skip_aspiration_window: false,
            skip_null_move_pruning: false,
            transposition_table: None,
        }
    }
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
        if !self.skip_sibling_beta_cutoff_sort {
            options.push("sibling_beta_cutoff_sort".to_string());
        }
        if !self.skip_null_move_pruning {
            options.push("null_move_pruning".to_string());
        }
        if !self.skip_aspiration_window {
            options.push("aspiration_window".to_string());
        }
        if self.transposition_table.is_some() {
            options.push("transposition_table".to_string());
        }
        write!(f, "{{ {} }}", options.join_vec(", "))
    }
}
pub struct IterativeSearch {
    alpha_beta: AlphaBetaStack,
    start_game: Game,

    best_variations_per_depth: Vec<Vec<SimpleMove>>,
    best_moves_cache: BestMovesCache,

    options: IterativeSearchOptions,

    no_moves_found: bool,
}

impl IterativeSearch {
    pub fn new(game: Game, options: IterativeSearchOptions) -> ErrorResult<Self> {
        let best_moves_cache = BestMovesCache::new();
        let search_options = AlphaBetaOptions {
            skip_quiescence: options.skip_quiescence,
            skip_sibling_beta_cutoff_sort: options.skip_sibling_beta_cutoff_sort,
            skip_null_move_pruning: options.skip_null_move_pruning,
            transposition_table: options.transposition_table.clone(),

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

    pub fn max_depth(&self) -> usize {
        self.alpha_beta.evaluate_at_depth
    }

    pub fn bestmove(&self) -> Option<(SimpleMove, Vec<SimpleMove>)> {
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
                    Some((variation, score)) => {
                        let depth = self.alpha_beta.evaluate_at_depth;
                        log(&format!(
                            "at depth {}: bestmove {} ponder {} ({}), beta-cutoffs {}, evaluations {}, start moves searched {}",
                            depth,
                            variation[0].to_string(),
                            variation[1..].iter().map(|m| m.to_string()).collect::<Vec<_>>().join_vec(" "),
                            score,
                            self.alpha_beta.num_beta_cutoffs,
                            self.alpha_beta.num_evaluations,
                            self.alpha_beta.num_starting_moves_searched,
                        ));

                        self.best_moves_cache.update(&self.start_game, &variation)?;
                        self.best_variations_per_depth.push(variation);

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

pub fn warmup_iterative_deepening() {
    // make sure any lazy-statics are generated
    IterativeSearch::new(
        Game::from_fen("startpos").unwrap(),
        IterativeSearchOptions::default(),
    )
    .unwrap()
    .iterate(&mut |_| {})
    .unwrap();
}

#[allow(unused)]
#[test]
fn test_iterative_deepening_for_depth() {
    // let fen = "startpos";
    // let max_depth = 7;

    // mid-game fen
    let fen = "r3k2r/1bq1bppp/pp2p3/2p1n3/P3PP2/2PBN3/1P1BQ1PP/R4RK1 b kq - 0 16";
    let max_depth = 8;

    // // late-game fen
    // let fen = "6k1/8/4p3/3r4/5n2/1Q6/1K1R4/8 w";
    // let max_depth = 8;

    warmup_iterative_deepening();

    let mut results: Vec<(u128, String)> = vec![];

    let skip_all = IterativeSearchOptions {
        skip_aspiration_window: true,
        skip_cache_sort: true,
        skip_capture_sort: true,
        skip_sibling_beta_cutoff_sort: true,
        skip_null_move_pruning: true,
        transposition_table: None,
        ..IterativeSearchOptions::default()
    };

    let transposition_table = Some(Rc::new(RefCell::new(TranspositionTable::new())));

    let options_to_try = vec![
        IterativeSearchOptions {
            transposition_table: transposition_table.clone(),
            ..IterativeSearchOptions::default()
        },
        IterativeSearchOptions {
            transposition_table: transposition_table.clone(),
            ..IterativeSearchOptions::default()
        },
        // IterativeSearchOptions {
        //     skip_aspiration_window: false,
        //     ..skip_all.clone()
        // },
        // IterativeSearchOptions {
        //     skip_cache_sort: false,
        //     ..skip_all.clone()
        // },
        // IterativeSearchOptions {
        //     skip_capture_sort: false,
        //     ..skip_all.clone()
        // },
        IterativeSearchOptions {
            skip_sibling_beta_cutoff_sort: false,
            ..skip_all.clone()
        },
        // IterativeSearchOptions {
        //     skip_null_move_pruning: false,
        //     ..skip_all.clone()
        // },
        // skip_all.clone(),
        IterativeSearchOptions::default(),
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

        if options.transposition_table.is_some() {
            let tt = options.transposition_table.as_ref().unwrap().borrow();
            let stats = tt.stats.borrow();
            println!(
                "hits: {}, misses: {}, collisions: {}, updates: {}, size: {} gb",
                stats.hits.to_formatted_string(&Locale::en),
                stats.misses.to_formatted_string(&Locale::en),
                stats.collisions.to_formatted_string(&Locale::en),
                stats.updates.to_formatted_string(&Locale::en),
                (stats.size_in_bytes / 1024 / 1024 / 1024).to_formatted_string(&Locale::en),
            );
        }

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

fn warmup() {
    // make sure any lazy-statics are generated
    IterativeSearch::new(
        Game::from_fen("startpos").unwrap(),
        IterativeSearchOptions::default(),
    )
    .unwrap()
    .iterate(&mut |_| {})
    .unwrap();
}

#[test]
fn test_iterative_deepening_transposition_table() {
    // mid-game fen
    let fen = "r3k2r/1bq1bppp/pp2p3/2p1n3/P3PP2/2PBN3/1P1BQ1PP/R4RK1 b kq - 0 16";
    let max_depth = 6;

    warmup();

    let options = IterativeSearchOptions {
        transposition_table: Some(Rc::new(RefCell::new(TranspositionTable::new()))),
        ..IterativeSearchOptions::default()
    };

    let mut search = IterativeSearch::new(Game::from_fen(fen).unwrap(), options.clone()).unwrap();

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

    (move || loop {
        search.iterate(&mut log_callback).unwrap();
        if search.max_depth() >= max_depth {
            break;
        }
        if start_time.elapsed() > std::time::Duration::from_secs(2) {
            break;
        }
    })();

    let total_time = start_time.elapsed();
    println!(
        "{:>5} ms total",
        total_time.as_millis().to_formatted_string(&Locale::en)
    );
}
