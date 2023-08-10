use lazy_static::*;
use memoize::memoize;

use super::super::types::*;
use super::*;

#[derive(Debug, Clone)]
pub struct CastlingRequirements {
    pub require_safe: Vec<BoardIndex>,
    pub require_empty: Vec<BoardIndex>,
    pub king_start: BoardIndex,
    pub king_end: BoardIndex,
    pub rook_start: BoardIndex,
    pub rook_end: BoardIndex,
    pub castling_pieces: Bitboard,
}

lazy_static! {
    static ref WHITE_KINGSIDE_CASTLING: CastlingRequirements = CastlingRequirements {
        require_safe: map_index_from_file_rank_strs(["e1", "f1", "g1"]),
        require_empty: map_index_from_file_rank_strs(["f1", "g1"]),
        king_start: index_from_file_rank_str("e1").unwrap(),
        king_end: index_from_file_rank_str("g1").unwrap(),
        rook_start: index_from_file_rank_str("h1").unwrap(),
        rook_end: index_from_file_rank_str("f1").unwrap(),
        castling_pieces: bitboard_with_file_rank_strs_set(&["e1", "h1"]),
    };
    static ref WHITE_QUEENSIDE_CASTLING: CastlingRequirements = CastlingRequirements {
        require_safe: map_index_from_file_rank_strs(["e1", "d1", "c1"]),
        require_empty: map_index_from_file_rank_strs(["b1", "c1", "d1"]),
        king_start: index_from_file_rank_str("e1").unwrap(),
        king_end: index_from_file_rank_str("c1").unwrap(),
        rook_start: index_from_file_rank_str("a1").unwrap(),
        rook_end: index_from_file_rank_str("d1").unwrap(),
        castling_pieces: bitboard_with_file_rank_strs_set(&["e1", "a1"]),
    };
    static ref BLACK_KINGSIDE_CASTLING: CastlingRequirements = CastlingRequirements {
        require_safe: map_index_from_file_rank_strs(["e8", "f8", "g8"]),
        require_empty: map_index_from_file_rank_strs(["f8", "g8"]),
        king_start: index_from_file_rank_str("e8").unwrap(),
        king_end: index_from_file_rank_str("g8").unwrap(),
        rook_start: index_from_file_rank_str("h8").unwrap(),
        rook_end: index_from_file_rank_str("f8").unwrap(),
        castling_pieces: bitboard_with_file_rank_strs_set(&["e8", "h8"]),
    };
    static ref BLACK_QUEENSIDE_CASTLING: CastlingRequirements = CastlingRequirements {
        require_safe: map_index_from_file_rank_strs(["e8", "d8", "c8"]),
        require_empty: map_index_from_file_rank_strs(["b8", "c8", "d8"]),
        king_start: index_from_file_rank_str("e8").unwrap(),
        king_end: index_from_file_rank_str("c8").unwrap(),
        rook_start: index_from_file_rank_str("a8").unwrap(),
        rook_end: index_from_file_rank_str("d8").unwrap(),
        castling_pieces: bitboard_with_file_rank_strs_set(&["e8", "a8"]),
    };
}

pub fn castling_requirements(
    player: Player,
    castling_side: CastlingSide,
) -> &'static CastlingRequirements {
    match player {
        Player::White => match castling_side {
            CastlingSide::Kingside => &WHITE_KINGSIDE_CASTLING,
            CastlingSide::Queenside => &WHITE_QUEENSIDE_CASTLING,
        },
        Player::Black => match castling_side {
            CastlingSide::Kingside => &BLACK_KINGSIDE_CASTLING,
            CastlingSide::Queenside => &BLACK_QUEENSIDE_CASTLING,
        },
    }
}

pub fn castling_allowed_after_move(
    player: Player,
    castling_side: CastlingSide,
    start_index: BoardIndex,
) -> bool {
    let castling_requirements = castling_requirements(player, castling_side);
    let castling_piece_moved = bb_contains(castling_requirements.castling_pieces, start_index);
    !castling_piece_moved
}
