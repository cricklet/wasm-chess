use std::iter;

use strum::IntoEnumIterator;

use crate::{bitboards::*, game::Game, helpers::*, types::*};

#[derive(Debug, Copy, Clone)]
pub enum OnlyCaptures {
    NO,
    YES,
}

#[derive(Debug, Copy, Clone)]
pub enum OnlyQueenPromotion {
    NO,
    YES,
}

impl OnlyQueenPromotion {
    pub fn pieces(&self) -> Box<dyn Iterator<Item = Piece>> {
        match self {
            OnlyQueenPromotion::NO => Box::new(PROMOTION_PIECES.into_iter()),
            OnlyQueenPromotion::YES => Box::new(iter::once(Piece::Queen)),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Quiet {
    Castle {
        rook_start: BoardIndex,
        rook_end: BoardIndex,
    },
    PawnSkip {
        skipped_index: BoardIndex,
    },
    PawnPromotion {
        promotion_piece: Piece,
    },
    Move,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Capture {
    EnPassant { taken_index: BoardIndex },
    Take { taken_piece: PlayerPiece },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MoveType {
    Quiet(Quiet),
    Capture(Capture),
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Move {
    pub piece: PlayerPiece,
    pub start_index: BoardIndex,
    pub end_index: BoardIndex,
    pub move_type: MoveType,
}

impl Move {
    pub fn to_uci(&self) -> String {
        format!("{}{}", self.start_index, self.end_index)
    }
}

#[test]
pub fn test_move_to_uci() {
    let m = Move {
        piece: PlayerPiece::new(Player::White, Piece::Pawn),
        start_index: BoardIndex::from(8),
        end_index: BoardIndex::from(16),
        move_type: MoveType::Quiet(Quiet::Move),
    };
    assert_eq!(m.to_uci(), "a2a3");
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}{} {:?}",
            self.piece, self.start_index, self.end_index, self.move_type,
        )
    }
}

impl std::fmt::Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}{} {:?}",
            self.piece, self.start_index, self.end_index, self.move_type,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JumpingPiece {
    Knight,
    King,
}

pub fn potential_bb_to_moves(
    PlayerPiece { player, piece }: PlayerPiece,
    piece_index: BoardIndex,
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
        let taken_piece = bitboards.piece_at_index(end_index);

        match taken_piece {
            Some(taken_piece) => Ok(Move {
                piece: PlayerPiece { player, piece },
                start_index,
                end_index,
                move_type: MoveType::Capture(Capture::Take { taken_piece }),
            }),
            None => err_result(&format!(
                "no piece at index {:} but marked as capture",
                end_index
            )),
        }
    });

    let quiet_moves = each_index_of_one(quiet).map(move |end_index| Move {
        piece: PlayerPiece::new(player, piece),
        start_index,
        end_index,
        move_type: MoveType::Quiet(Quiet::Move),
    });

    match only_captures {
        OnlyCaptures::NO => Box::new(capture_moves.chain(quiet_moves.map(Ok))),
        OnlyCaptures::YES => Box::new(capture_moves),
    }
}

pub fn walk_potential_bb(
    index: BoardIndex,
    bitboards: Bitboards,
    piece: Piece,
) -> ErrorResult<Bitboard> {
    let walk_types = walk_type_for_piece(piece);

    let walk_types = walk_types?;

    let mut danger_bb = 0;

    for walk_type in walk_types.iter() {
        danger_bb |= moves_bb_for_piece_and_blockers(index, *walk_type, bitboards.all_occupied());
    }

    Ok(danger_bb)
}

pub fn walk_moves(
    PlayerPiece { player, piece }: PlayerPiece,
    bitboards: Bitboards,
    only_captures: OnlyCaptures,
) -> Box<dyn Iterator<Item = ErrorResult<Move>>> {
    let moves = each_index_of_one(bitboards.pieces[player][piece]).flat_map(move |piece_index| {
        let potential_bb = walk_potential_bb(piece_index, bitboards, piece);

        match potential_bb {
            Err(err) => Box::new(iter::once(Err(err))),
            Ok(potential) => potential_bb_to_moves(
                PlayerPiece::new(player, piece),
                piece_index,
                potential,
                bitboards,
                only_captures,
            ),
        }
    });

    Box::new(moves)
}

pub fn jumping_bitboard(index: BoardIndex, jumping_piece: JumpingPiece) -> Bitboard {
    let lookup: &[Bitboard; 64] = match jumping_piece {
        JumpingPiece::Knight => &KNIGHT_MOVE_BITBOARD,
        JumpingPiece::King => &KING_MOVE_BITBOARD,
    };
    lookup[index.i]
}

pub fn jump_moves(
    player: Player,
    bitboards: Bitboards,
    jumping_piece: JumpingPiece,
    only_captures: OnlyCaptures,
) -> impl Iterator<Item = ErrorResult<Move>> {
    let piece = match jumping_piece {
        JumpingPiece::Knight => Piece::Knight,
        JumpingPiece::King => Piece::King,
    };
    each_index_of_one(bitboards.pieces[player][piece]).flat_map(move |piece_index| {
        let potential = jumping_bitboard(piece_index, jumping_piece);
        potential_bb_to_moves(
            PlayerPiece::new(player, piece),
            piece_index,
            potential,
            bitboards,
            only_captures,
        )
    })
}

pub fn pawn_attacking_bb(start_bb: Bitboard, capture_dir: Direction) -> Bitboard {
    let start_bb = start_bb & pre_move_mask(capture_dir);
    let attacking_bb = rotate_toward_index_63(start_bb, capture_dir.offset());
    attacking_bb
}

pub fn pawn_moves(
    player: Player,
    bitboards: Bitboards,
    only_captures: OnlyCaptures,
    only_queen_promotion: OnlyQueenPromotion,
) -> Box<dyn Iterator<Item = ErrorResult<Move>>> {
    let pawns = bitboards.pieces[player][Piece::Pawn];

    let capture_moves = {
        let offsets = pawn_capture_directions_for_player(player);
        offsets.iter().flat_map(move |capture_dir| {
            let attacking_bb = pawn_attacking_bb(pawns, *capture_dir);
            let capture_bb = attacking_bb & bitboards.occupied[other_player(player)];

            each_index_of_one(capture_bb).map(move |end_index| {
                let start_index =
                    BoardIndex::from((end_index.i as isize - capture_dir.offset()) as usize);
                let taken_piece = bitboards.piece_at_index(end_index);
                match taken_piece {
                    Some(taken_piece) => Ok(Move {
                        piece: PlayerPiece::new(player, Piece::Pawn),
                        start_index,
                        end_index,
                        move_type: MoveType::Capture(Capture::Take { taken_piece }),
                    }),
                    None => {
                        err_result(&format!("no piece at {:} but marked as capture", end_index))
                    }
                }
            })
        })
    };

    if let OnlyCaptures::YES = only_captures {
        return Box::new(capture_moves);
    }

    let quiet_push_dir = pawn_push_direction_for_player(player);

    let push_moves = {
        let masked_pawns = pawns & pre_move_mask(quiet_push_dir);

        let moved_pawns = rotate_toward_index_63(masked_pawns, quiet_push_dir.offset())
            & !bitboards.all_occupied();

        let pushed_pawns = moved_pawns & !*PAWN_PROMOTION_BITBOARD;
        let promotion_pawns = moved_pawns & *PAWN_PROMOTION_BITBOARD;

        let pushed_pawns = each_index_of_one(pushed_pawns).map(move |end_index| {
            let start_index =
                BoardIndex::from((end_index.i as isize - quiet_push_dir.offset()) as usize);
            Ok(Move {
                piece: PlayerPiece::new(player, Piece::Pawn),
                start_index,
                end_index,
                move_type: MoveType::Quiet(Quiet::Move),
            })
        });

        let promotion_pawns = each_index_of_one(promotion_pawns).flat_map(move |end_index| {
            let start_index =
                BoardIndex::from((end_index.i as isize - quiet_push_dir.offset()) as usize);
            only_queen_promotion.pieces().map(move |promo_piece| {
                Ok(Move {
                    piece: PlayerPiece::new(player, Piece::Pawn),
                    start_index,
                    end_index,
                    move_type: MoveType::Quiet(Quiet::PawnPromotion {
                        promotion_piece: promo_piece,
                    }),
                })
            })
        });

        pushed_pawns.chain(promotion_pawns)
    };
    let skip_moves = {
        let masked_pawns = pawns & starting_pawns_mask(player);
        let push1 = rotate_toward_index_63(masked_pawns, quiet_push_dir.offset())
            & !bitboards.all_occupied();
        let push2 =
            rotate_toward_index_63(push1, quiet_push_dir.offset()) & !bitboards.all_occupied();

        each_index_of_one(push2).map(move |end_index| {
            let start_index =
                BoardIndex::from((end_index.i as isize - 2 * quiet_push_dir.offset()) as usize);
            let skipped_index =
                BoardIndex::from((end_index.i as isize - quiet_push_dir.offset()) as usize);
            Ok(Move {
                piece: PlayerPiece::new(player, Piece::Pawn),
                start_index,
                end_index,
                move_type: MoveType::Quiet(Quiet::PawnSkip { skipped_index }),
            })
        })
    };

    Box::new(push_moves.chain(skip_moves).chain(capture_moves))
}

pub fn en_passant_move(
    player: Player,
    bitboards: Bitboards,
    en_passant_index: Option<BoardIndex>,
) -> Option<Move> {
    let pawns = bitboards.pieces[player][Piece::Pawn];

    if let Some(en_passant_index) = en_passant_index {
        let en_passant_bb = single_bitboard(en_passant_index);

        for (dir, target_dir) in en_passant_move_and_target_offsets(player) {
            let pawns = pawns & pre_move_mask(*dir);
            let moved_pawns = rotate_toward_index_63(pawns, dir.offset());

            if moved_pawns & en_passant_bb != 0 {
                let start_index =
                    BoardIndex::from((en_passant_index.i as isize - dir.offset()) as usize);
                let taken_index =
                    BoardIndex::from((start_index.i as isize + target_dir.offset()) as usize);

                return Some(Move {
                    piece: PlayerPiece::new(player, Piece::Pawn),
                    start_index,
                    end_index: en_passant_index,
                    move_type: MoveType::Capture(Capture::EnPassant { taken_index }),
                });
            }
        }
    }

    None
}

pub fn castling_moves<'game>(
    player: Player,
    state: &'game Game,
) -> impl Iterator<Item = ErrorResult<Move>> + 'game {
    let castling_sides: CastlingSideIter = CastlingSide::iter();
    let allowed_castling_sides =
        castling_sides.filter(move |castling_side| state.can_castle[player][*castling_side]);

    let castling_requirements = allowed_castling_sides
        .map(move |castling_side| castling_requirements(player, castling_side));

    let empty_castling_sides = castling_requirements.filter(move |&req| {
        for &empty_index in &req.require_empty {
            if state.board.piece_at_index(empty_index).is_some() {
                return false;
            }
        }
        return true;
    });

    let safe_castling_sides = empty_castling_sides.flat_map(move |req| {
        let require_safe = req.require_safe.iter();
        let potential_castles = require_safe.filter_map(move |&safe_index| {
            match index_in_danger(player, safe_index, state) {
                Err(err) => {
                    return Some(Err(err));
                }
                Ok(true) => return None,
                Ok(false) => return Some(Ok(req)),
            }
        });

        potential_castles
    });

    let moves = safe_castling_sides.map(move |req_result| {
        req_result.map(|req| Move {
            piece: PlayerPiece::new(player, Piece::King),
            start_index: req.king_start,
            end_index: req.king_end,
            move_type: MoveType::Quiet(Quiet::Castle {
                rook_start: req.rook_start,
                rook_end: req.rook_end,
            }),
        })
    });

    moves
}

pub fn all_moves<'game>(
    player: Player,
    state: &'game Game,
    only_captures: OnlyCaptures,
    only_queen_promotion: OnlyQueenPromotion,
) -> impl Iterator<Item = ErrorResult<Move>> + 'game {
    let pawn_moves = pawn_moves(player, state.board, only_captures, only_queen_promotion);
    let knight_moves = jump_moves(player, state.board, JumpingPiece::Knight, only_captures);
    let king_moves = jump_moves(player, state.board, JumpingPiece::King, only_captures);
    let bishop_moves = walk_moves(
        PlayerPiece::new(player, Piece::Bishop),
        state.board,
        only_captures,
    );
    let rook_moves = walk_moves(
        PlayerPiece::new(player, Piece::Rook),
        state.board,
        only_captures,
    );
    let queen_moves = walk_moves(
        PlayerPiece::new(player, Piece::Queen),
        state.board,
        only_captures,
    );
    let castling_moves = castling_moves(player, state);

    let en_passant = en_passant_move(player, state.board, state.en_passant);
    let en_passant = en_passant.map(Ok);

    pawn_moves
        .chain(en_passant)
        .chain(knight_moves)
        .chain(king_moves)
        .chain(bishop_moves)
        .chain(rook_moves)
        .chain(queen_moves)
        .chain(castling_moves)
}

pub fn index_in_danger(player: Player, target: BoardIndex, state: &Game) -> ErrorResult<bool> {
    let enemy = other_player(player);

    let target_bb = single_bitboard(target);

    let pawn_dangers = pawn_capture_directions_for_player(player).iter().fold(
        0,
        move |danger_bb, capture_dir| -> Bitboard {
            danger_bb | pawn_attacking_bb(target_bb, *capture_dir)
        },
    );

    let knight_dangers = jumping_bitboard(target, JumpingPiece::Knight);
    let king_dangers = jumping_bitboard(target, JumpingPiece::King);

    let bishop_dangers = walk_potential_bb(target, state.board, Piece::Bishop);
    let rook_dangers = walk_potential_bb(target, state.board, Piece::Rook);

    let bishop_dangers = bishop_dangers?;
    let rook_dangers = rook_dangers?;

    let queen_dangers = bishop_dangers | rook_dangers;

    let enemy_pawns = state.board.pieces[enemy][Piece::Pawn];
    let enemy_knights = state.board.pieces[enemy][Piece::Knight];
    let enemy_kings = state.board.pieces[enemy][Piece::King];
    let enemy_bishops = state.board.pieces[enemy][Piece::Bishop];
    let enemy_rooks = state.board.pieces[enemy][Piece::Rook];
    let enemy_queens = state.board.pieces[enemy][Piece::Queen];

    if enemy_pawns & pawn_dangers != 0
        || enemy_knights & knight_dangers != 0
        || enemy_kings & king_dangers != 0
        || enemy_bishops & bishop_dangers != 0
        || enemy_rooks & rook_dangers != 0
        || enemy_queens & queen_dangers != 0
    {
        Ok(true)
    } else {
        Ok(false)
    }
}
