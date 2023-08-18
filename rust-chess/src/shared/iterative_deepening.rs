/*
To implement iterative deepening, we need a few things:
* SearchStack::search needs to return the PV
* To do that, as we do a regular search, we need to keep track of
the best line at each frame.
* Then, we need some way to sort the moves to prioritize PV moves
*/

use std::iter;

use crate::{
    bitboard::{warm_magic_cache, BoardIndex},
    game::Game,
    helpers::{ErrorResult, Joinable, OptionResult},
    iterative_traversal::null_move_sort,
    moves::Move,
    search::{LoopResult, SearchStack},
};

pub struct BestMovesCache {
    best_moves: Vec<Option<(BoardIndex, BoardIndex)>>,
    bits: usize,
    mask: u64,
}

impl BestMovesCache {
    pub fn new() -> Self {
        Self {
            best_moves: vec![None; (2 as usize).pow(20)],
            bits: 20, // 1 mb
            mask: (2 as u64).pow(20) - 1,
        }
    }

    pub fn add(&mut self, game: &Game, m: Move) {
        let hash = game.zobrist().value();
        let masked = hash & self.mask;

        self.best_moves[masked as usize] = Some((m.start_index, m.end_index));
    }

    pub fn update(&mut self, game: &Game, moves: &Vec<Move>) -> ErrorResult<()> {
        let mut game = game.clone();

        for m in moves {
            self.add(&game, *m);
            game.make_move(*m)?;
        }

        Ok(())
    }

    pub fn get(&self, game: &Game) -> Option<(BoardIndex, BoardIndex)> {
        let hash = game.zobrist().value();
        let masked = hash & self.mask;

        self.best_moves[masked as usize]
    }

    pub fn sort(&self, game: &Game, moves: &mut Vec<Move>) -> ErrorResult<()> {
        let hash = game.zobrist().value();
        let masked = hash & self.mask;

        if let Some((start, end)) = self.best_moves[masked as usize] {
            let i = moves
                .iter()
                .position(|m| m.start_index == start && m.end_index == end);
            if let Some(i) = i {
                moves.swap(0, i);
            }
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct IterativeSearchOptions {
    skip_quiescence: bool,
    skip_cache_sort: bool,
}

pub struct IterativeSearch {
    search: SearchStack,
    start_game: Game,

    best_variations_per_depth: Vec<Vec<Move>>,
    best_moves_cache: BestMovesCache,

    options: IterativeSearchOptions,

    no_moves_found: bool,
}

impl IterativeSearch {
    pub fn new(game: Game, options: IterativeSearchOptions) -> ErrorResult<Self> {
        let best_moves_cache = BestMovesCache::new();
        let search = SearchStack::with(game, 1, options.skip_quiescence)?;
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

    pub fn iterate(&mut self, log: &mut Vec<String>) -> ErrorResult<()> {
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
                        log.push(format!(
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

                        self.search = SearchStack::with(
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
fn test_iterative_deepening_for_count() {
    // let fen = "startpos";

    // mid-game fen
    let fen = "r3k2r/1bq1bppp/pp2p3/2p1n3/P3PP2/2PBN3/1P1BQ1PP/R4RK1 b kq - 0 16";

    // late-game fen
    // let fen = "6k1/8/4p3/3r4/5n2/1Q6/1K1R4/8 w";

    IterativeSearch::new(
        Game::from_fen(fen).unwrap(),
        IterativeSearchOptions::default(),
    )
    .unwrap()
    .iterate(&mut vec![])
    .unwrap();

    for &skip_quiescence in [false, true].iter() {
        for &skip_cache_sort in [false, true].iter() {
            let options = IterativeSearchOptions {
                skip_cache_sort,
                skip_quiescence,
            };
            let mut search = IterativeSearch::new(Game::from_fen(fen).unwrap(), options).unwrap();

            let mut log = vec![];

            for _ in 0..1_000_000 {
                search.iterate(&mut log).unwrap();
            }

            log.push(format!(
                "at depth {}: beta-cutoffs {}, evaluations {}, start moves searched {}",
                search.search.max_depth,
                search.search.num_beta_cutoffs,
                search.search.num_evaluations,
                search.search.num_starting_moves_searched,
            ));

            // Calling `iterate()` should be idempotent
            search.iterate(&mut log).unwrap();

            println!(
                "\nskip_cache_sort {}, skip_quiescence {}",
                skip_cache_sort, skip_quiescence
            );
            println!("{:#?}", log);
        }
    }
}

#[test]
fn test_iterative_deepening_for_depth() {
    // let fen = "startpos";

    // mid-game fen
    let fen = "r3k2r/1bq1bppp/pp2p3/2p1n3/P3PP2/2PBN3/1P1BQ1PP/R4RK1 b kq - 0 16";

    // late-game fen
    // let fen = "6k1/8/4p3/3r4/5n2/1Q6/1K1R4/8 w";

    // make sure any lazy-statics are generated
    IterativeSearch::new(Game::from_fen(fen).unwrap(), IterativeSearchOptions::default())
        .unwrap()
        .iterate(&mut vec![])
        .unwrap();

    for &skip_quiescence in [false, true].iter() {
        for &skip_cache_sort in [false, true].iter() {
            let options = IterativeSearchOptions {
                skip_cache_sort,
                skip_quiescence,
            };
            let mut search = IterativeSearch::new(Game::from_fen(fen).unwrap(), options).unwrap();

            let mut log = vec![];

            let start_time = std::time::Instant::now();

            let mut start_time_for_depth = std::time::Instant::now();
            let mut last_depth = 0;

            loop {
                search.iterate(&mut log).unwrap();

                let depth = search.search.max_depth;
                if depth > last_depth {
                    println!(
                        "{} ms at depth {}",
                        start_time_for_depth.elapsed().as_millis(),
                        depth - 1
                    );
                    last_depth = depth;
                    start_time_for_depth = std::time::Instant::now();
                }
                if depth >= 6 {
                    break;
                }
            }

            log.push(format!(
                "{} ms, at depth {}: beta-cutoffs {}, evaluations {}, start moves searched {}",
                start_time.elapsed().as_millis(),
                search.search.max_depth,
                search.search.num_beta_cutoffs,
                search.search.num_evaluations,
                search.search.num_starting_moves_searched,
            ));

            // Calling `iterate()` should be idempotent
            search.iterate(&mut log).unwrap();

            println!(
                "skip_cache_sort {}, skip_quiescence {}",
                skip_cache_sort, skip_quiescence
            );
            println!("{:#?}\n", log);
        }
    }
}
