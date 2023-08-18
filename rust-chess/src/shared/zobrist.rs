use std::{sync::Mutex, fmt::{Display, Formatter}};

use lazy_static::lazy_static;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use strum::IntoEnumIterator;

use crate::{
    bitboard::{BoardIndex, self, Bitboards, ForPlayer},
    game::{Game, CanCastleOnSide},
    types::{CastlingSide, Player, PlayerPiece, Piece}, moves::Move, helpers::ErrorResult,
};

lazy_static! {
    static ref RANDOMIZER: Mutex<ChaCha8Rng> = {
        let r = ChaCha8Rng::seed_from_u64(32879419);
        Mutex::new(r)
    };
    static ref ZOBRIST_PIECE_AT_SQUARE: [[u64; 64]; 12] = {
        let mut arr = [[0; 64]; 12];
        let mut r = RANDOMIZER.lock().unwrap();
        for player in Player::iter() {
            for piece in Piece::iter() {
                let player_piece = PlayerPiece::new(player, piece);
                for board_index in 0..64 {
                    arr[player_piece.to_usize()][board_index] = r.gen();
                }
            }
        }
        for row in arr {
            for v in row {
                if v == 0 {
                    panic!("failed to initialize zobrist array");
                }
            }
        }
        arr
    };
    static ref ZOBRIST_SIDE_TO_MOVE: u64 = {
        let mut r = RANDOMIZER.lock().unwrap();
        r.gen()
    };
    static ref ZOBRIST_CASTLING_RIGHTS: [u64; 4] = {
        let mut arr = [0; 4];
        let mut r = RANDOMIZER.lock().unwrap();
        for i in 0..4 {
            arr[i] = r.gen();
        }
        arr
    };
    static ref ZOBRIST_EN_PASSANT: [u64; 8] = {
        let mut arr = [0; 8];
        let mut r = RANDOMIZER.lock().unwrap();
        for i in 0..8 {
            arr[i] = r.gen();
        }
        arr
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ZobristHash {
    value: u64,
}

impl Display for ZobristHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", self.value)
    }
}


impl ZobristHash {
    pub fn value(self) -> u64 {
        self.value
    }

    pub fn from(
        bitboards: &Bitboards,
        player: Player,
        can_castle: ForPlayer<CanCastleOnSide>,
        en_passant: Option<BoardIndex>
    ) -> Self {
        let mut hash = 0;
        for board_index in 0..64 {
            let piece = bitboards.piece_at_index(BoardIndex::from(board_index));
            match piece {
                None => {}
                Some(player_piece) => {
                    hash ^= ZOBRIST_PIECE_AT_SQUARE[player_piece.to_usize()][board_index];
                }
            }
        }
        if player == Player::White {
            hash ^= *ZOBRIST_SIDE_TO_MOVE;
        }
        for (i, &player) in [Player::White, Player::Black].iter().enumerate() {
            for (j, &side) in [CastlingSide::Kingside, CastlingSide::Queenside]
                .iter()
                .enumerate()
            {
                if can_castle[player][side] {
                    hash ^= ZOBRIST_CASTLING_RIGHTS[2 * i + j];
                }
            }
        }
        if let Some(en_passant) = en_passant {
            hash ^= ZOBRIST_EN_PASSANT[en_passant.file()];
        }
        Self { value: hash }
    }

    pub fn on_castling_change(&mut self, player: Player, side: CastlingSide) {
        self.value ^= ZOBRIST_CASTLING_RIGHTS[2 * player.to_usize() + side.to_usize()];
    }
    pub fn on_player_change(&mut self) {
        self.value ^= *ZOBRIST_SIDE_TO_MOVE;
    }
    pub fn on_en_passant_change(&mut self, target: BoardIndex) {
        self.value ^= ZOBRIST_EN_PASSANT[target.file()];
    }
    pub fn on_update_square(&mut self, index: BoardIndex, piece: PlayerPiece) {
        self.value ^= ZOBRIST_PIECE_AT_SQUARE[piece.to_usize()][index.i];
    }
}

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
