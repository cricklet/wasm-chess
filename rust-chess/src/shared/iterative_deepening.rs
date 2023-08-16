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
    moves::Move,
    search::{LoopResult, SearchStack}, iterative_traversal::null_move_sort,
};

pub struct BestMovesCache {
    best_moves: Vec<(BoardIndex, BoardIndex)>,
    bits: usize,
    mask: u64,
}

impl BestMovesCache {
    pub fn new() -> Self {
        Self {
            best_moves: Vec::with_capacity((2 as usize).pow(20)),
            bits: 20, // 1 mb
            mask: (2 as u64).pow(20) - 1,
        }
    }

    pub fn add(&mut self, game: &Game, m: Move) {
        let hash = game.zobrist().value();
        let masked = hash & self.mask;

        self.best_moves[masked as usize] = (m.start_index, m.end_index);
    }

    pub fn get(&self, game: &Game) -> Option<(BoardIndex, BoardIndex)> {
        let hash = game.zobrist().value();
        let masked = hash & self.mask;

        self.best_moves.get(masked as usize).cloned()
    }

    pub fn sort(&self, game: &Game, moves: &mut Vec<Move>) -> ErrorResult<()> {
        let hash = game.zobrist().value();
        let masked = hash & self.mask;

        if let Some((start, end)) = self.best_moves.get(masked as usize) {
            let i = moves
                .iter()
                .position(|m| m.start_index == *start && m.end_index == *end);
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

        match self.search.iterate(null_move_sort)? {
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
                            "at depth {}: bestmove {} ponder {} ({})",
                            depth,
                            bestmove.to_uci(),
                            response.join_vec(" "),
                            score
                        ));

                        self.best_variations_per_depth
                            .push(iter::once(bestmove).chain(response).collect());

                        self.search = SearchStack::with(
                            self.start_game.clone(),
                            depth + 1,
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
fn test_start_iterative_deepening() {
    let mut search = IterativeSearch::new(Game::from_fen("startpos").unwrap()).unwrap();
    let mut log = vec![];

    for _ in 0..1_000_000 {
        search.iterate(&mut log).unwrap();
    }

    // Calling `iterate()` should be idempotent
    search.iterate(&mut log).unwrap();

    println!("{:#?}", log);
    println!("{:#?}", search.bestmove());
}
