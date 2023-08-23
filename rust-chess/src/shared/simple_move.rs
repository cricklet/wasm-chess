// pub fn walk_move_is_legal(
//     start: BoardIndex,
//     end: BoardIndex,
//     PlayerPiece { player, piece }: PlayerPiece,
//     game: &Game,
// ) -> ErrorResult<bool> {
//     todo!()
// }

// pub fn walk_or_jump_move_is_legal(
//     start: BoardIndex,
//     end: BoardIndex,
//     piece: PlayerPiece,
//     game: &Game,
// ) -> ErrorResult<bool> {
//     match piece.piece {
//         Piece::Bishop => walk_move_is_legal(start, end, piece, game),
//         Piece::Rook => walk_move_is_legal(start, end, piece, game),
//         Piece::Queen => walk_move_is_legal(start, end, piece, game),
//         Piece::Pawn => pawn_move_is_legal(start, end, piece, game),
//         Piece::Knight => jump_move_is_legal(start, end, piece, game),
//         Piece::King => jump_move_is_legal(start, end, piece, game),
//     }
// }

use std::fmt::{Display, Formatter};

use crate::{bitboard::{BoardIndex, matches_castling, pawn_push_direction_for_player, starting_pawns_mask, single_bitboard}, types::Piece, helpers::{ErrorResult, OptionResult, err_result}, moves::{Move, all_moves, MoveOptions, castling_side_is_safe, MoveType, Quiet, Capture}, game::Game};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SimpleMove {
    pub start: BoardIndex,
    pub end: BoardIndex,
    pub promotion: Option<Piece>,
}

impl Display for SimpleMove {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let promo = self.promotion.map(|p| p.to_uci());
        let promo = promo.unwrap_or(&"");
        write!(f, "{}{}{}", self.start, self.end, promo)
    }
}

impl SimpleMove {
    pub fn from_str(s: &str) -> ErrorResult<Self> {
        let start = BoardIndex::from_str(&s[0..2])?;
        let end = BoardIndex::from_str(&s[2..4])?;
        let promotion = if s.len() > 4 {
            Piece::from(s.chars().nth(4).as_result()?)
        } else {
            None
        };

        Ok(Self {
            start,
            end,
            promotion,
        })
    }

    pub fn from(m: &Move) -> Self {
        Self {
            start: m.start_index,
            end: m.end_index,
            promotion: m.promotion,
        }
    }

    pub fn to_move(&self, game: &Game) -> ErrorResult<Option<Move>> {
        let start = self.start;
        let end = self.end;
        let promo = self.promotion;

        let start_piece = game.bitboards().piece_at_index(start);
        let start_piece = match start_piece {
            Some(start_piece) => start_piece,
            None => return Ok(None),
        };

        let end_piece = game.bitboards().piece_at_index(end);

        if start_piece.player != game.player() {
            return Ok(None);
        }

        if promo.is_some() && start_piece.piece != Piece::Pawn {
            return Ok(None);
        }

        // Quiet
        match end_piece {
            None => {
                // => Castle
                if start_piece.piece == Piece::King {
                    if let Some((side, req)) = matches_castling(game.player(), start, end) {
                        if castling_side_is_safe(side, start_piece.player, game, req)? {
                            return Ok(Some(Move {
                                piece: start_piece,
                                start_index: start,
                                end_index: end,
                                move_type: MoveType::Quiet(Quiet::Castle {
                                    rook_start: req.rook_start,
                                    rook_end: req.rook_end,
                                }),
                                promotion: promo.expect_none(|| {
                                    "promotions not allowed on castling moves".to_string()
                                })?,
                            }));
                        } else {
                            return Ok(None);
                        }
                    }
                }
                if start_piece.piece == Piece::Pawn {
                    // => PawnSkip
                    let pawn_dir = pawn_push_direction_for_player(start_piece.player).offset();
                    let pawn_start_mask = starting_pawns_mask(start_piece.player);
                    let starting_bb = single_bitboard(start);
                    if (starting_bb & pawn_start_mask) != 0 {
                        let skipped = BoardIndex::from((start.i as isize + pawn_dir) as usize);
                        let expected_end =
                            BoardIndex::from((start.i as isize + pawn_dir + pawn_dir) as usize);

                        if end == expected_end {
                            if game.bitboards().is_occupied(skipped) {
                                return Ok(None);
                            }
                            if game.bitboards().is_occupied(expected_end) {
                                return err_result("pawn skip end index is occupied, should have been checked above");
                            }

                            return Ok(Some(Move {
                                piece: start_piece,
                                start_index: start,
                                end_index: end,
                                move_type: MoveType::Quiet(Quiet::PawnSkip {
                                    skipped_index: skipped,
                                }),
                                promotion: promo.expect_none(|| {
                                    "promotions not allowed on pawn skip moves".to_string()
                                })?,
                            }));
                        }
                    }

                    // Capture => EnPassant
                    if let Some(en_passant) = game.en_passant() {
                        if end == en_passant {
                            let taken_index =
                                BoardIndex::from((end.i as isize - pawn_dir) as usize);
                            let taken_piece =
                                game.bitboards().piece_at_index(taken_index).as_result()?;
                            if taken_piece.piece != Piece::Pawn
                                && taken_piece.player != game.player().other()
                            {
                                return err_result(&format!(
                                    "taken piece {} for en-passant isn't enemy pawn",
                                    taken_piece
                                ));
                            }
                            return Ok(Some(Move {
                                piece: start_piece,
                                start_index: start,
                                end_index: end,
                                move_type: MoveType::Capture(Capture::EnPassant { taken_index }),
                                promotion: promo.expect_none(|| {
                                    "promotions not allowed on pawn skip moves".to_string()
                                })?,
                            }));
                        }
                    }
                }
                // => Move
                // TODO: make sure the walk squares are empty
                Ok(Some(Move {
                    piece: start_piece,
                    start_index: start,
                    end_index: end,
                    move_type: MoveType::Quiet(Quiet::Move),
                    promotion: promo,
                }))
            }
            Some(end_piece) => {
                // Capture => Take
                if end_piece.player != game.player().other() {
                    return Ok(None);
                }
                if end_piece.piece == Piece::King {
                    return Ok(None);
                }
                // TODO: make sure the walk squares are empty
                Ok(Some(Move {
                    piece: start_piece,
                    start_index: start,
                    end_index: end,
                    move_type: MoveType::Capture(Capture::Take {
                        taken_piece: end_piece,
                    }),
                    promotion: promo,
                }))
            }
        }
    }


    pub fn make_move(&self, game: Game) -> ErrorResult<Game> {
        let mut next = game.clone();
        let mut moves = vec![];
        all_moves(&mut moves, next.player(), &next, MoveOptions::default())?;
        for m in &moves {
            if m.start_index == self.start
                && m.end_index == self.end
                && m.promotion == self.promotion
            {
                next.make_move(*m)?;
                return Ok(next);
            }
        }
        return err_result(&format!("couldn't find move {} for game {}", self, game));
    }
}
