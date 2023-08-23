use std::{
    collections::HashSet,
    fmt::{Display, Formatter},
};

use num_format::{ToFormattedString, Locale};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::{
    bitboard::{
        matches_castling, pawn_capture_directions_for_player, pawn_push_direction_for_player,
        pawn_push_rank_direction_for_player, single_bitboard, starting_pawns_mask, BoardIndex,
    },
    game::Game,
    helpers::{err_result, ErrorResult, OptionResult},
    moves::{
        all_moves, can_castle_on_side, jumping_bitboard, walk_potential_bb, Capture, JumpingPiece,
        Move, MoveOptions, MoveType, Quiet,
    },
    types::{Piece, PlayerPiece},
};

pub fn walk_is_unobstructed(
    start: BoardIndex,
    end: BoardIndex,
    PlayerPiece { piece, .. }: PlayerPiece,
    game: &Game,
) -> ErrorResult<bool> {
    let end_bb = single_bitboard(end);
    let walk_bb = walk_potential_bb(start, game.bitboards().all_occupied(), piece)?;
    return Ok(walk_bb & end_bb != 0);
}

pub fn jump_makes_sense(
    start: BoardIndex,
    end: BoardIndex,
    PlayerPiece { piece, .. }: PlayerPiece,
) -> ErrorResult<bool> {
    let jumping_piece = match piece {
        Piece::Knight => JumpingPiece::Knight,
        Piece::King => JumpingPiece::King,
        _ => {
            return err_result(&format!(
                "jump_is_legal called with non-jumping piece {}",
                piece.to_uci()
            ))
        }
    };
    let end_bb = single_bitboard(end);
    let jump_bb = jumping_bitboard(start, jumping_piece);

    return Ok(jump_bb & end_bb != 0);
}

pub fn matches_pawn_push(
    start: BoardIndex,
    end: BoardIndex,
    PlayerPiece { player, .. }: PlayerPiece,
) -> ErrorResult<bool> {
    let dir = pawn_push_direction_for_player(player);
    if start.i as isize + dir.offset() != end.i as isize {
        return Ok(false);
    }
    return Ok(true);
}

pub fn matches_pawn_capture(
    start: BoardIndex,
    end: BoardIndex,
    PlayerPiece { player, .. }: PlayerPiece,
) -> ErrorResult<bool> {
    let dirs = pawn_capture_directions_for_player(player);
    for dir in dirs {
        if start.i as isize + dir.offset() == end.i as isize {
            return Ok(true);
        }
    }
    return Ok(false);
}

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

        let player = game.player();

        let end_piece = game.bitboards().piece_at_index(end);

        if start_piece.player != player {
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
                    if let Some((side, req)) = matches_castling(player, start, end) {
                        if can_castle_on_side(side, player, game, req)? {
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
                    let pawn_dir = pawn_push_direction_for_player(player).offset();
                    let pawn_start_mask = starting_pawns_mask(player);
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
                                && taken_piece.player != player.other()
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
                let matches = match start_piece.piece {
                    Piece::Bishop => walk_is_unobstructed(start, end, start_piece, game)?,
                    Piece::Rook => walk_is_unobstructed(start, end, start_piece, game)?,
                    Piece::Queen => walk_is_unobstructed(start, end, start_piece, game)?,
                    Piece::Pawn => matches_pawn_push(start, end, start_piece)?,
                    Piece::Knight => jump_makes_sense(start, end, start_piece)?,
                    Piece::King => jump_makes_sense(start, end, start_piece)?,
                };

                if !matches {
                    return Ok(None);
                }

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
                if end_piece.player != player.other() {
                    return Ok(None);
                }
                if end_piece.piece == Piece::King {
                    return Ok(None);
                }

                let matches = match start_piece.piece {
                    Piece::Bishop => walk_is_unobstructed(start, end, start_piece, game)?,
                    Piece::Rook => walk_is_unobstructed(start, end, start_piece, game)?,
                    Piece::Queen => walk_is_unobstructed(start, end, start_piece, game)?,
                    Piece::Pawn => matches_pawn_capture(start, end, start_piece)?,
                    Piece::Knight => jump_makes_sense(start, end, start_piece)?,
                    Piece::King => jump_makes_sense(start, end, start_piece)?,
                };

                if !matches {
                    return Ok(None);
                }

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

#[test]
fn test_simple_move_conversion() {
    let fen = "r3k2r/1bq1bppp/pp2p3/2p1n3/P3PP2/2PBN3/1P1BQ1PP/R4RK1 b kq - 0 16";
    let game = Game::from_fen(fen).unwrap();

    let mut r = ChaCha8Rng::seed_from_u64(32879417);

    let mut moves = vec![];
    all_moves(&mut moves, game.player(), &game, MoveOptions::default()).unwrap();

    for m in &moves {
        let simple_move = SimpleMove::from(m);
        assert_eq!(simple_move.to_move(&game).unwrap(), Some(*m));
    }

    let moves: HashSet<Move> = HashSet::from_iter(moves.into_iter());

    for i in 1..=10_000_000 {
        if i % 1_000_000 == 0 {
            println!("{}", i.to_formatted_string(&Locale::en));
        }

        let start = BoardIndex::from(r.gen_range(0..64));
        let end = BoardIndex::from(r.gen_range(0..64));

        let simple_move = SimpleMove {
            start,
            end,
            promotion: None,
        };

        let m = simple_move.to_move(&game).unwrap();
        if let Some(m) = m {
            assert!(
                moves.contains(&m),
                "randomly generated move {} is not valid",
                m
            );
        } else {
            for m in &moves {
                if m.start_index == simple_move.start && m.end_index == simple_move.end {
                    panic!(
                        "randomly generated move {} is valid, but not found in all_moves",
                        m
                    );
                }
            }
        }
    }
}
