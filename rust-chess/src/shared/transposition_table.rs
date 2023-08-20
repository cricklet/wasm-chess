use core::fmt;
use std::fmt::Formatter;

use crate::{alphabeta::Score, game::Game, helpers::ErrorResult, moves::Move, zobrist::SimpleMove};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CachedValue {
    Static(Score),
    Exact(Score, SimpleMove),
    BetaCutoff(Score, SimpleMove), // lower bound
    AlphaMiss(Score, SimpleMove),  // upper bound
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheEntry {
    hash: u64,
    pub depth: usize,
    pub value: CachedValue,
}

#[derive(Clone)]
pub struct TranspositionTable {
    table: Vec<Option<CacheEntry>>,
    bits: usize,
    mask: u64,
}

impl fmt::Debug for TranspositionTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TranspositionTable")
            .field("bits", &self.bits)
            .field("mask", &self.mask)
            .finish()
    }
}

impl TranspositionTable {
    pub fn new() -> Self {
        Self {
            table: vec![None; (2 as usize).pow(22)],
            bits: 22,
            mask: (2 as u64).pow(22) - 1,
        }
    }

    pub fn get(&self, game: &Game) -> Option<CacheEntry> {
        let hash = game.zobrist().value();
        let mask = hash & self.mask;

        let entry = self.table[mask as usize];
        if let Some(entry) = entry {
            if entry.hash == hash {
                return Some(entry);
            }
        }
        None
    }

    pub fn update(
        &mut self,
        game: &Game,
        value: CachedValue,
        depth: usize,
    ) -> ErrorResult<()> {
        let hash = game.zobrist().value();
        let mask = hash & self.mask;

        let entry = &mut self.table[mask as usize];

        // Always replace for now
        *entry = Some(CacheEntry { hash, depth, value });

        Ok(())
    }
}
