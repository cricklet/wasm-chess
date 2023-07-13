use crate::{bitboards::*, helpers::*, types::*};

pub fn moves_from_bitboards(
    player: Player,
    piece_index: usize,
    potential: Bitboard,
    bitboards: Bitboards,
) -> impl Iterator<Item = ErrorResult<Move>> {
    let self_occupied = bitboards.occupied[player];
    let enemy_occupied = bitboards.occupied[other_player(player)];

    let moves = potential & !self_occupied;

    let capture = moves & enemy_occupied;
    let quiet = moves & !capture;

    let start_index = piece_index;

    let quiet_moves = each_index_of_one(quiet).map(move |end_index| {
        Ok(Move {
            player,
            start_index,
            end_index,
            move_type: MoveType::Quiet(Quiet::Move),
        })
    });

    let capture_moves = each_index_of_one(capture).map(move |end_index| {
        let start_index = piece_index;
        let taken_piece = bitboards.piece_at_index(end_index);

        match taken_piece {
            Some(taken_piece) => Ok(Move {
                player,
                start_index,
                end_index,
                move_type: MoveType::Capture(Capture::Take { taken_piece }),
            }),
            None => Err(format!(
                "no piece at index {:} but marked as capture",
                end_index
            )),
        }
    });

    quiet_moves.chain(capture_moves)
}

pub fn walk_moves(
    player: Player,
    piece_index: usize,
    bitboards: Bitboards,
    piece: WalkingPieces,
) -> impl Iterator<Item = ErrorResult<Move>> {
    let potential = moves_bb_for_piece_and_blockers(piece_index, piece, bitboards.all_occupied());

    moves_from_bitboards(player, piece_index, potential, bitboards)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JumpingPiece {
    Knight,
    King,
}

pub fn jump_moves(
    player: Player,
    piece_index: usize,
    bitboards: Bitboards,
    piece: JumpingPiece,
) -> impl Iterator<Item = ErrorResult<Move>> {
    let potential = match piece {
        JumpingPiece::Knight => KNIGHT_MOVE_BITBOARD[piece_index],
        JumpingPiece::King => KING_MOVE_BITBOARD[piece_index],
    };
    moves_from_bitboards(player, piece_index, potential, bitboards)
}

pub enum Quiet {
    Castle { rook_start: usize, rook_end: usize },
    PawnSkip { skipped_index: usize },
    Move,
}

pub enum Capture {
    EnPassant { taken_index: usize },
    Take { taken_piece: PlayerPiece },
}

pub enum MoveType {
    Quiet(Quiet),
    Capture(Capture),
}

pub struct Move {
    player: Player,
    start_index: usize,
    end_index: usize,
    move_type: MoveType,
}

pub fn pawn_moves(player: Player, bitboards: Bitboards) -> impl Iterator<Item = Move> {
    let pawns = bitboards.pieces[player][Piece::Pawn];
    let pawn_offset = pawn_push_offset_for_player(player);

    let push_moves = {
        let masked_pawns = pawns & pre_move_mask(pawn_offset).unwrap();
        let pushed_pawns =
            shift_toward_index_63(masked_pawns, pawn_offset) & !bitboards.all_occupied();

        each_index_of_one(pushed_pawns).map(move |end_index| {
            let start_index = (end_index as isize - pawn_offset) as usize;
            Move {
                player,
                start_index,
                end_index,
                move_type: MoveType::Quiet(Quiet::Move),
            }
        })
    };
    let skip_moves = {
        let masked_pawns = pawns & starting_pawns_mask(player);
        let push1 = shift_toward_index_63(masked_pawns, pawn_offset) & !bitboards.all_occupied();
        let push2 = shift_toward_index_63(push1, pawn_offset) & !bitboards.all_occupied();

        each_index_of_one(push2).map(move |end_index| {
            let start_index = (end_index as isize - pawn_offset) as usize;
            Move {
                player,
                start_index,
                end_index,
                move_type: MoveType::Quiet(Quiet::Move),
            }
        })
    };
    let capture_moves = {
        let offsets = pawn_capture_offsets_for_player(player);
        offsets.iter().flat_map(move |offset| {
            let masked_pawns = pawns & pre_move_mask(*offset).unwrap();
            let moved_pawns = shift_toward_index_63(masked_pawns, *offset);
            let capture_bb = moved_pawns & bitboards.occupied[other_player(player)];

            each_index_of_one(capture_bb).map(move |end_index| {
                let start_index = (end_index as isize - pawn_offset) as usize;
                Move {
                    player,
                    start_index,
                    end_index,
                    move_type: MoveType::Quiet(Quiet::Move),
                }
            })
        })
    };

    push_moves.chain(skip_moves).chain(capture_moves)
}

pub fn en_passant_move(
    player: Player,
    bitboards: Bitboards,
    en_passant_index: Option<usize>,
) -> Option<Move> {
    let pawns = bitboards.pieces[player][Piece::Pawn];

    if let Some(en_passant_index) = en_passant_index {
        let en_passant_bb = single_bitboard(en_passant_index);

        for (move_offset, target_offset) in en_passant_move_and_target_offsets(player) {
            let moved_pawns = shift_toward_index_63(pawns, *move_offset);

            if moved_pawns & en_passant_bb != 0 {
                return Some(Move {
                    player,
                    start_index: (en_passant_index as isize - move_offset) as usize,
                    end_index: en_passant_index,
                    move_type: MoveType::Capture(Capture::EnPassant {
                        taken_index: (en_passant_index as isize - target_offset) as usize,
                    }),
                });
            }
        }
    }

    None
}
