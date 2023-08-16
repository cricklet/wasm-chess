/*
To implement iterative deepening, we need a few things:
* SearchStack::search needs to return the PV
* To do that, as we do a regular search, we need to keep track of
the best line at each frame.
* Then, we need some way to sort the moves to prioritize PV moves
*/

use std::iter;

use crate::{
    bitboard::BoardIndex,
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

pub struct IterativeSearch {
    search: SearchStack,
    start_game: Game,

    best_variations_per_depth: Vec<Vec<Move>>,
    best_moves_cache: BestMovesCache,

    use_cache_sort: bool,

    no_moves_found: bool,
}

impl IterativeSearch {
    pub fn new(game: Game) -> ErrorResult<Self> {
        let best_moves_cache = BestMovesCache::new();
        let search = SearchStack::with(game, 1)?;
        Ok(Self {
            search,
            start_game: game,
            best_variations_per_depth: vec![],
            best_moves_cache,
            use_cache_sort: true,
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

        let use_cache_sort = self.use_cache_sort;
        let best_moves_cache = &self.best_moves_cache;

        let sorter = move |game: &Game, moves: &mut Vec<Move>| -> ErrorResult<()> {
            if use_cache_sort {
                best_moves_cache.sort(game, moves)
            } else {
                null_move_sort(game, moves)
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
                            "at depth {}: bestmove {} ponder {} ({}), beta-cutoffs {}, evaluations {}",
                            depth,
                            bestmove.to_uci(),
                            response.join_vec(" "),
                            score,
                            self.search.num_beta_cutoffs,
                            self.search.num_evaluations,
                        ));

                        let best_variation = iter::once(bestmove).chain(response).collect();
                        self.best_moves_cache
                            .update(&self.start_game, &best_variation)?;
                        self.best_variations_per_depth.push(best_variation);

                        self.search = SearchStack::with(self.start_game.clone(), depth + 1)?;
                    }
                }
            }
            LoopResult::Continue => {}
        }

        Ok(())
    }
}

#[test]
fn test_start_iterative_deepening() {
    for &use_cache_sort in [false, true].iter() {
        let mut search = IterativeSearch::new(Game::from_fen("startpos").unwrap()).unwrap();
        search.use_cache_sort = use_cache_sort;

        let mut log = vec![];

        for _ in 0..1_000_000 {
            search.iterate(&mut log).unwrap();
        }

        // Calling `iterate()` should be idempotent
        search.iterate(&mut log).unwrap();

        println!("\nuse_cache_sort {}", use_cache_sort);
        println!("{:#?}", log);
        println!("{:#?}", search.bestmove());
    }
}
