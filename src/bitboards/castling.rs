use memoize::memoize;

use crate::types::*;

use super::arithmetic::*;

#[derive(Debug, Clone, Copy)]
pub struct CastlingRequirements {
    require_safe: Bitboard,
    require_empty: Bitboard,
    king_start: isize,
    king_end: isize,
    rook_start: isize,
    rook_end: isize,
    castling_pieces: Bitboard,
}

#[memoize]
pub fn castling_requirements(player: Player, castling_side: CastlingSide) -> CastlingRequirements {
    match player {
        Player::White => match castling_side {
            CastlingSide::KingSide => CastlingRequirements {
                require_safe: bitboard_with_file_rank_strs_set(&["e1", "f1", "g1"]),
                require_empty: bitboard_with_file_rank_strs_set(&["f1", "g1"]),
                king_start: index_from_file_rank_str("e1").unwrap(),
                king_end: index_from_file_rank_str("g1").unwrap(),
                rook_start: index_from_file_rank_str("h1").unwrap(),
                rook_end: index_from_file_rank_str("f1").unwrap(),
                castling_pieces: bitboard_with_file_rank_strs_set(&["e1", "h1"]),
            },
            CastlingSide::QueenSide => CastlingRequirements {
                require_safe: bitboard_with_file_rank_strs_set(&["e1", "d1", "c1"]),
                require_empty: bitboard_with_file_rank_strs_set(&["b1", "c1", "d1"]),
                king_start: index_from_file_rank_str("e1").unwrap(),
                king_end: index_from_file_rank_str("c1").unwrap(),
                rook_start: index_from_file_rank_str("a1").unwrap(),
                rook_end: index_from_file_rank_str("d1").unwrap(),
                castling_pieces: bitboard_with_file_rank_strs_set(&["e1", "a1"]),
            },
        },
        Player::Black => match castling_side {
            CastlingSide::KingSide => CastlingRequirements {
                require_safe: bitboard_with_file_rank_strs_set(&["e8", "f8", "g8"]),
                require_empty: bitboard_with_file_rank_strs_set(&["f8", "g8"]),
                king_start: index_from_file_rank_str("e8").unwrap(),
                king_end: index_from_file_rank_str("g8").unwrap(),
                rook_start: index_from_file_rank_str("h8").unwrap(),
                rook_end: index_from_file_rank_str("f8").unwrap(),
                castling_pieces: bitboard_with_file_rank_strs_set(&["e8", "h8"]),
            },
            CastlingSide::QueenSide => CastlingRequirements {
                require_safe: bitboard_with_file_rank_strs_set(&["e8", "d8", "c8"]),
                require_empty: bitboard_with_file_rank_strs_set(&["b8", "c8", "d8"]),
                king_start: index_from_file_rank_str("e8").unwrap(),
                king_end: index_from_file_rank_str("c8").unwrap(),
                rook_start: index_from_file_rank_str("a8").unwrap(),
                rook_end: index_from_file_rank_str("d8").unwrap(),
                castling_pieces: bitboard_with_file_rank_strs_set(&["e8", "a8"]),
            },
        },
    }
}
