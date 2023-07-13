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

    each_index_of_one(quiet)
        .map(move |end_index| {
            Ok(Move {
                player,
                start_index,
                end_index,
                move_type: MoveType::Quiet(Quiet::Move),
            })
        })
        .chain(each_index_of_one(capture).map(move |end_index| {
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
        }))
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
        JumpingPiece::Knight => knight_move_bitboard(piece_index),
        JumpingPiece::King => king_move_bitobard(piece_index),
    };
    moves_from_bitboards(player, piece_index, potential, bitboards)
}

pub enum Quiet {
    Castle { rook_start: usize, rook_end: usize },
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

pub fn pawn_pushes(player: Player, bitboards: Bitboards) -> impl Iterator<Item = Move> {
    let pawns = bitboards.pieces[player][Piece::Pawn];
    let pawn_offset = pawn_push_offset_for_player(player);

    let unblocked_pawns = pawns & pre_move_mask(pawn_offset).unwrap();
    let potential = shift_toward_index_63(unblocked_pawns, pawn_offset);
    let moves = unblocked_pawns & !bitboards.all_occupied();

    each_index_of_one(moves)
        .map(move |end_index| {
            let start_index = end_index - pawn_offset;
            Ok(Move {
                player,
                start_index,
                end_index,
                move_type: MoveType::Quiet(Quiet::Move),
            })
}
