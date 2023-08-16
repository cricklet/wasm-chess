use std::sync::Mutex;

use lazy_static::lazy_static;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::{
    bitboard::BoardIndex,
    game::Game,
    types::{CastlingSide, Player},
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

pub fn zobrist_for_game(game: &Game) -> u64 {
    let mut hash = 0;
    for board_index in 0..64 {
        let piece = game.board.piece_at_index(BoardIndex::from(board_index));
        match piece {
            None => {}
            Some(player_piece) => {
                hash ^= ZOBRIST_PIECE_AT_SQUARE[player_piece.to_usize()][board_index];
            }
        }
    }
    if game.player == Player::White {
        hash ^= *ZOBRIST_SIDE_TO_MOVE;
    }
    for (i, &player) in [Player::White, Player::Black].iter().enumerate() {
        for (j, &side) in [CastlingSide::Kingside, CastlingSide::Queenside]
            .iter()
            .enumerate()
        {
            if game.can_castle[player][side] {
                hash ^= ZOBRIST_CASTLING_RIGHTS[2 * i + j];
            }
        }
    }
    if let Some(en_passant) = game.en_passant {
        hash ^= ZOBRIST_EN_PASSANT[en_passant.file()];
    }
    hash
}

