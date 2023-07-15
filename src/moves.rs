use crate::{bitboards::*, game::Game, helpers::*, types::*};

#[derive(Debug, Copy, Clone)]
pub enum OnlyCaptures {
    NO,
    YES,
}

#[derive(Debug, Copy, Clone)]
pub enum Quiet {
    Castle { rook_start: usize, rook_end: usize },
    PawnSkip { skipped_index: usize },
    Move,
}

#[derive(Debug, Copy, Clone)]
pub enum Capture {
    EnPassant { taken_index: usize },
    Take { taken_piece: PlayerPiece },
}

#[derive(Debug, Copy, Clone)]
pub enum MoveType {
    Quiet(Quiet),
    Capture(Capture),
}

#[derive(Debug, Copy, Clone)]
pub struct Move {
    player: Player,
    start_index: usize,
    end_index: usize,
    move_type: MoveType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JumpingPiece {
    Knight,
    King,
}

pub fn moves_from_bitboards(
    player: Player,
    piece_index: usize,
    potential: Bitboard,
    bitboards: Bitboards,
    only_captures: OnlyCaptures,
) -> Box<dyn Iterator<Item = ErrorResult<Move>>> {
    let self_occupied = bitboards.occupied[player];
    let enemy_occupied = bitboards.occupied[other_player(player)];

    let moves = potential & !self_occupied;

    let capture = moves & enemy_occupied;
    let quiet = moves & !capture;

    let start_index = piece_index;

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

    let quiet_moves = each_index_of_one(quiet).map(move |end_index| {
        Ok(Move {
            player,
            start_index,
            end_index,
            move_type: MoveType::Quiet(Quiet::Move),
        })
    });

    match only_captures {
        OnlyCaptures::NO => Box::new(capture_moves.chain(quiet_moves)),
        OnlyCaptures::YES => Box::new(capture_moves),
    }
}

pub fn walk_moves(
    player: Player,
    bitboards: Bitboards,
    piece: Piece,
    only_captures: OnlyCaptures,
) -> Box<dyn Iterator<Item = ErrorResult<Move>>> {
    let walk_types = walk_type_for_piece(piece);
    let walk_types = flatten_iter_result(walk_types.map(|walk_type| walk_type.iter()));

    let moves = map_successes(walk_types, move |walk_type| {
        each_index_of_one(bitboards.pieces[player][piece])
            .map(move |piece_index| {
                let potential = moves_bb_for_piece_and_blockers(
                    piece_index,
                    *walk_type,
                    bitboards.all_occupied(),
                );
                moves_from_bitboards(player, piece_index, potential, bitboards, only_captures)
            })
            .flatten()
    })
    .flatten()
    .flatten();

    Box::new(moves)
}

pub fn jump_moves(
    player: Player,
    bitboards: Bitboards,
    jumping_piece: JumpingPiece,
    only_captures: OnlyCaptures,
) -> impl Iterator<Item = ErrorResult<Move>> {
    let lookup: &[Bitboard; 64] = match jumping_piece {
        JumpingPiece::Knight => &KNIGHT_MOVE_BITBOARD,
        JumpingPiece::King => &KING_MOVE_BITBOARD,
    };
    let piece = match jumping_piece {
        JumpingPiece::Knight => Piece::Knight,
        JumpingPiece::King => Piece::King,
    };
    each_index_of_one(bitboards.pieces[player][piece])
        .map(move |piece_index| {
            let potential = lookup[piece_index];
            moves_from_bitboards(player, piece_index, potential, bitboards, only_captures)
        })
        .flatten()
}

pub fn pawn_moves(
    player: Player,
    bitboards: Bitboards,
    only_captures: OnlyCaptures,
) -> Box<dyn Iterator<Item = Move>> {
    let pawns = bitboards.pieces[player][Piece::Pawn];
    let pawn_offset = pawn_push_offset_for_player(player);

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

    if let OnlyCaptures::YES = only_captures {
        return Box::new(capture_moves);
    }

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

    Box::new(push_moves.chain(skip_moves).chain(capture_moves))
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

// pub fn castling_moves(player: Player) -> impl Iterator<Item = Move> {

// }

pub fn all_moves(
    player: Player,
    state: &Game,
    only_captures: OnlyCaptures,
) -> impl Iterator<Item = ErrorResult<Move>> {
    let pawn = pawn_moves(player, state.board, only_captures).map(Ok);
    let en_passant = en_passant_move(player, state.board, state.en_passant).map(Ok);
    let knight_moves = jump_moves(player, state.board, JumpingPiece::Knight, only_captures);
    let king_moves = jump_moves(player, state.board, JumpingPiece::King, only_captures);
    let bishop_moves = walk_moves(player, state.board, Piece::Bishop, only_captures);
    let rook_moves = walk_moves(player, state.board, Piece::Rook, only_captures);
    let queen_moves = walk_moves(player, state.board, Piece::Queen, only_captures);

    pawn.chain(en_passant)
        .chain(knight_moves)
        .chain(king_moves)
        .chain(bishop_moves)
        .chain(rook_moves)
        .chain(queen_moves)
}
