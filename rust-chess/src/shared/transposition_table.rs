use core::fmt;
use std::{cell::RefCell, fmt::Formatter};

use crate::{alphabeta::Score, game::Game, helpers::ErrorResult, moves::Move, zobrist::SimpleMove};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheValue {
    Static(Score),
    Exact(Score, SimpleMove),
    BetaCutoff(Score, SimpleMove),
    AlphaMiss(Score),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheEntry {
    hash: u64,
    pub depth_remaining: usize,
    pub value: CacheValue,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TranspositionStats {
    pub hits: usize,
    pub misses: usize,
    pub collisions: usize,
    pub updates: usize,
}

#[derive(Clone)]
pub struct TranspositionTable {
    table: Vec<Option<CacheEntry>>,
    bits: usize,
    mask: u64,
    pub stats: RefCell<TranspositionStats>,
}

impl fmt::Debug for TranspositionTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TranspositionTable")
            .field("bits", &self.bits)
            .field("mask", &self.mask)
            .finish()
    }
}

const DEFAULT_BITS: u32 = 24;

impl TranspositionTable {
    pub fn new() -> Self {
        Self {
            table: vec![None; (2 as usize).pow(DEFAULT_BITS)],
            bits: DEFAULT_BITS as usize,
            mask: (2 as u64).pow(DEFAULT_BITS) - 1,
            stats: RefCell::new(TranspositionStats::default()),
        }
    }

    pub fn get(&self, game: &Game) -> Option<&CacheEntry> {
        let hash = game.zobrist().value();
        let mask = hash & self.mask;

        let entry = &self.table[mask as usize];
        if let Some(entry) = &entry {
            if entry.hash == hash {
                self.stats.borrow_mut().hits += 1;
                return Some(&entry);
            } else {
                self.stats.borrow_mut().collisions += 1;
            }
        }
        self.stats.borrow_mut().misses += 1;
        None
    }

    pub fn update(&mut self, game: &Game, value: CacheValue, depth: usize) -> ErrorResult<()> {
        let hash = game.zobrist().value();
        let mask = hash & self.mask;

        self.stats.borrow_mut().updates += 1;

        let entry = &mut self.table[mask as usize];

        // Always replace for now
        *entry = Some(CacheEntry { hash, depth_remaining: depth, value });

        Ok(())
    }
}
