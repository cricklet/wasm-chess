/*
To implement iterative deepening, we need a few things:
* SearchStack::search needs to return the PV
* To do that, as we do a regular search, we need to keep track of
the best line at each frame.
* Then, we need some way to sort the moves to prioritize PV moves
*/

use std::iter;

use crate::{
    alphabeta::{AlphaBetaStack, LoopResult},
    bitboard::{warm_magic_cache, BoardIndex},
    game::Game,
    helpers::{ErrorResult, Joinable, OptionResult},
    moves::Move,
    traversal::null_move_sort, zobrist::BestMovesCache,
};

#[derive(Default)]
pub struct IterativeSearchOptions {
    skip_quiescence: bool,
    skip_cache_sort: bool,
}

pub struct IterativeSearch {
    search: AlphaBetaStack,
    start_game: Game,

    best_variations_per_depth: Vec<Vec<Move>>,
    best_moves_cache: BestMovesCache,

    options: IterativeSearchOptions,

    no_moves_found: bool,
}

impl IterativeSearch {
    pub fn new(game: Game, options: IterativeSearchOptions) -> ErrorResult<Self> {
        let best_moves_cache = BestMovesCache::new();
        let search = AlphaBetaStack::with(game, 1, options.skip_quiescence)?;
        Ok(Self {
            search,
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
        let best_moves_cache = &self.best_moves_cache;

        let sorter = move |game: &Game, moves: &mut Vec<Move>| -> ErrorResult<()> {
            if skip_cache_sort {
                null_move_sort(game, moves)
            } else {
                best_moves_cache.sort(game, moves)
            }
        };

        match self.search.iterate(sorter)? {
            LoopResult::Done => {
                let result = self.search.bestmove();
                match result {
                    None => {
                        self.no_moves_found = true;
                        return Ok(());
                    }
                    Some((bestmove, response, score)) => {
                        let depth = self.search.max_depth;
                        log(&format!(
                            "at depth {}: bestmove {} ponder {} ({}), beta-cutoffs {}, evaluations {}, start moves searched {}",
                            depth,
                            bestmove.to_uci(),
                            response.iter().map(|m| m.to_uci()).collect::<Vec<_>>().join_vec(" "),
                            score,
                            self.search.num_beta_cutoffs,
                            self.search.num_evaluations,
                            self.search.num_starting_moves_searched,
                        ));

                        let best_variation = iter::once(bestmove).chain(response).collect();
                        self.best_moves_cache
                            .update(&self.start_game, &best_variation)?;
                        self.best_variations_per_depth.push(best_variation);

                        self.search = AlphaBetaStack::with(
                            self.start_game.clone(),
                            depth + 1,
                            self.options.skip_quiescence,
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

    // mid-game fen
    // let fen = "r3k2r/1bq1bppp/pp2p3/2p1n3/P3PP2/2PBN3/1P1BQ1PP/R4RK1 b kq - 0 16";

    // late-game fen
    let fen = "6k1/8/4p3/3r4/5n2/1Q6/1K1R4/8 w";

    // make sure any lazy-statics are generated
    IterativeSearch::new(
        Game::from_fen(fen).unwrap(),
        IterativeSearchOptions::default(),
    )
    .unwrap()
    .iterate(&mut |_| {})
    .unwrap();

    let skip_quiescence = false;

    // for &skip_quiescence in [false, true].iter() {
    for &skip_cache_sort in [false, true].iter() {
        let options = IterativeSearchOptions {
            skip_cache_sort,
            skip_quiescence,
        };
        let mut search = IterativeSearch::new(Game::from_fen(fen).unwrap(), options).unwrap();

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
            if search.search.max_depth >= 7 {
                break;
            }
        }

        log.push(format!("{} ms total", start_time.elapsed().as_millis(),));

        println!(
            "skip_cache_sort {}, skip_quiescence {}",
            skip_cache_sort, skip_quiescence
        );
        println!("{:#?}\n", log);
    }
    // }
}
