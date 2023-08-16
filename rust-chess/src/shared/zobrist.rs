use std::sync::Mutex;

use lazy_static::lazy_static;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::{
    bitboard::{BoardIndex, self, Bitboards, ForPlayer},
    game::{Game, CanCastleOnSide},
    types::{CastlingSide, Player, PlayerPiece},
};

lazy_static! {
    static ref RANDOMIZER: Mutex<ChaCha8Rng> = {
        let r = ChaCha8Rng::seed_from_u64(32879419);
        Mutex::new(r)
    };
    static ref ZOBRIST_PIECE_AT_SQUARE: [[u64; 64]; 13] = {
        let mut arr = [[0; 64]; 13];
        let mut r = RANDOMIZER.lock().unwrap();
        for piece in 1..13 {
            for board_index in 0..64 {
                arr[piece][board_index] = r.gen();
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
impl ZobristHash {
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
