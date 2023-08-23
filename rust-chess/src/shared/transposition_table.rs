use crate::{
    game::Game,
    helpers::{err_result, ErrorResult},
    moves::Move,
    score::Score, simple_move::SimpleMove, zobrist::ZobristHash,
};
use core::fmt;
use std::{cell::RefCell, fmt::Formatter, mem::size_of};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheValue {
    Static(Score),
    Exact(Score, SimpleMove),
    BetaCutoff(Score, SimpleMove),
    AlphaMiss(Score),
}

impl CacheValue {
    pub fn best_move(&self) -> Option<SimpleMove> {
        match self {
            CacheValue::Exact(_, m) => Some(*m),
            CacheValue::BetaCutoff(_, m) => Some(*m),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheEntry {
    hash: u64,
    pub depth_remaining: u8,
    pub value: CacheValue,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TranspositionStats {
    pub hits: usize,
    pub misses: usize,
    pub collisions: usize,
    pub updates: usize,
    pub size_in_bytes: usize,
}

#[derive(Clone)]
pub struct TranspositionTable {
    pub table: Vec<Option<CacheEntry>>,
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

// 27 => 8gb
// 26 => 4gb
const DEFAULT_BITS: u32 = 25;

impl TranspositionTable {
    pub fn new() -> Self {
        let result = Self {
            table: vec![None; (2 as usize).pow(DEFAULT_BITS)],
            bits: DEFAULT_BITS as usize,
            mask: (2 as u64).pow(DEFAULT_BITS) - 1,
            stats: RefCell::new(TranspositionStats::default()),
        };
        result.stats.borrow_mut().size_in_bytes = size_of::<CacheEntry>() * result.table.len();

        result
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

    pub fn clear(&mut self, zobrist: ZobristHash) {
        let mask = zobrist.value() & self.mask;
        self.table[mask as usize] = None;
    }

    pub fn update(
        &mut self,
        game: &Game,
        value: CacheValue,
        depth_remaining: usize,
    ) -> ErrorResult<()> {
        if depth_remaining > 255 {
            return err_result("depth_remaining must be less than 255");
        }

        let hash = game.zobrist().value();
        let mask = hash & self.mask;

        self.stats.borrow_mut().updates += 1;

        let entry = &mut self.table[mask as usize];

        if let Some(entry) = &entry {
            if entry.hash == hash && depth_remaining as u8 <= entry.depth_remaining {
                return Ok(());
            }
        }

        *entry = Some(CacheEntry {
            hash,
            depth_remaining: depth_remaining as u8,
            value,
        });
        Ok(())
    }
}
