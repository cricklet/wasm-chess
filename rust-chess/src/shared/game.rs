use strum::IntoEnumIterator;

use crate::bitboard::{
    matches_castling, pawn_push_direction_for_player, single_bitboard, starting_pawns_mask,
};
use crate::board::Board;
use crate::fen::FenDefinition;
use crate::moves::{can_castle_on_side, walk_potential_bb};
use crate::simple_move::SimpleMove;

use super::bitboard::FileRank;
use super::bitboard::{self, castling_allowed_after_move, Bitboards, BoardIndex};
use super::bitboard::{index_from_file_rank_str, ForPlayer};
use super::danger::Danger;
use super::helpers::*;
use super::moves::{all_moves, index_in_danger, Capture, Move, MoveOptions, MoveType, Quiet};
use super::types::{self, CastlingSide, Piece, Player, PlayerPiece, CASTLING_SIDES};
use super::zobrist::ZobristHash;
use derive_getters::Getters;

#[derive(Debug, Default, Copy, Clone)]
pub struct CanCastleOnSide {
    queenside: bool,
    kingside: bool,
}

impl std::ops::Index<CastlingSide> for CanCastleOnSide {
    type Output = bool;
    fn index(&self, index: CastlingSide) -> &Self::Output {
        match index {
            CastlingSide::Queenside => &self.queenside,
            CastlingSide::Kingside => &self.kingside,
        }
    }
}

impl std::ops::IndexMut<CastlingSide> for CanCastleOnSide {
    fn index_mut(&mut self, index: CastlingSide) -> &mut Self::Output {
        match index {
            CastlingSide::Queenside => &mut self.queenside,
            CastlingSide::Kingside => &mut self.kingside,
        }
    }
}

impl ForPlayer<CanCastleOnSide> {
    pub fn from_str(str: &str) -> ErrorResult<ForPlayer<CanCastleOnSide>> {
        Ok(match str {
            "-" => ForPlayer {
                white: Default::default(),
                black: Default::default(),
            },
            _ => {
                let mut can_castle_on_side_for_player: ForPlayer<CanCastleOnSide> = ForPlayer {
                    white: Default::default(),
                    black: Default::default(),
                };
                for c in str.chars() {
                    match c {
                        'K' => can_castle_on_side_for_player.white.kingside = true,
                        'Q' => can_castle_on_side_for_player.white.queenside = true,
                        'k' => can_castle_on_side_for_player.black.kingside = true,
                        'q' => can_castle_on_side_for_player.black.queenside = true,
                        _ => return err_result(&format!("invalid castling side {}", c)),
                    }
                }
                can_castle_on_side_for_player
            }
        })
    }

    pub fn to_fen(&self) -> String {
        let mut fen = String::new();
        if self.white.kingside {
            fen.push('K');
        }
        if self.white.queenside {
            fen.push('Q');
        }
        if self.black.kingside {
            fen.push('k');
        }
        if self.black.queenside {
            fen.push('q');
        }
        if fen.is_empty() {
            "-".to_string()
        } else {
            fen
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Legal {
    No,
    Yes,
}

#[derive(Copy, Clone)]
pub struct Game {
    board: Board,
    pub half_moves_since_pawn_or_capture: usize,
    pub full_moves_total: usize,
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({})\n{}",
            self.to_fen(),
            self.board.zobrist(),
            self.board.bitboards()
        )
    }
}

impl std::fmt::Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Game {{\n{}}}",
            indent(
                &format!(
                    "{} ({})\n{}",
                    self.to_fen(),
                    self.board.zobrist(),
                    self.board.bitboards()
                ),
                2
            ),
        )
    }
}

impl Default for Game {
    fn default() -> Self {
        let bitboards = Bitboards::new();
        let player = Player::White;
        let can_castle = ForPlayer {
            white: Default::default(),
            black: Default::default(),
        };
        let en_passant = None;
        Self {
            board: Board::new(bitboards, player, can_castle, en_passant),
            half_moves_since_pawn_or_capture: 0,
            full_moves_total: 1,
        }
    }
}

impl Game {
    pub fn err<T>(&self, msg: &str) -> ErrorResult<T> {
        err_result::<T>(&format!("{}\n\n{}", msg, self))
    }

    pub fn to_fen(&self) -> String {
        format!(
            "{} {} {}",
            self.board.to_fen(),
            self.half_moves_since_pawn_or_capture,
            self.full_moves_total
        )
    }

    pub fn board(&self) -> &Board {
        &self.board
    }
    pub fn bitboards(&self) -> &Bitboards {
        self.board.bitboards()
    }
    pub fn player(&self) -> Player {
        *self.board.player()
    }
    pub fn can_castle(&self) -> &ForPlayer<CanCastleOnSide> {
        self.board.can_castle()
    }
    pub fn en_passant(&self) -> Option<BoardIndex> {
        *self.board.en_passant()
    }
    pub fn zobrist(&self) -> ZobristHash {
        *self.board.zobrist()
    }

    pub fn from_position_uci(uci: &str) -> ErrorResult<Game> {
        let (position_str, moves) = FenDefinition::split_uci(uci)?;
        Game::from_position_and_moves(&position_str, &moves)
    }

    pub fn from_position_and_moves(position_str: &str, moves: &[String]) -> ErrorResult<Game> {
        let mut game = {
            if position_str == "startpos" {
                Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            } else if position_str.starts_with("fen") {
                let fen_part = &position_str["fen".len()..].trim();
                Game::from_fen(fen_part)
            } else {
                err_result(&format!("invalid position {}", position_str))
            }
        }?;

        for m in moves {
            let m = game
                .move_from_str(m)
                .expect_ok(|| format!("invalid move '{}'", m))?;
            game.make_move(m)?;
        }

        Ok(game)
    }

    pub fn from_fen(fen: &str) -> ErrorResult<Game> {
        if fen == "startpos" {
            return Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        }

        let definition = FenDefinition::from(fen)?;
        Ok(Game {
            board: Board::new(
                definition.bitboards,
                definition.player,
                definition.can_castle,
                definition.en_passant,
            ),
            half_moves_since_pawn_or_capture: definition.half_moves_since_pawn_or_capture,
            full_moves_total: definition.full_moves_total,
        })
    }

    pub fn move_from_str(&self, move_str: &str) -> Option<Move> {
        let mut moves_buffer = vec![];
        all_moves(
            &mut moves_buffer,
            self.player(),
            &self,
            MoveOptions::default(),
        )
        .unwrap();

        for &m in moves_buffer.iter() {
            let mut next_game = *self;
            next_game.make_move(m).unwrap();

            let king_index = next_game
                .bitboards()
                .index_of_piece(self.player(), Piece::King);
            let illegal_move =
                index_in_danger(self.player(), king_index, &next_game.bitboards()).unwrap();

            if illegal_move {
                continue;
            }

            if m.to_uci() == move_str {
                return Some(m);
            }
        }

        None
    }

    pub fn move_legality(&self, m: &Move, previous_danger: &Danger) -> Legal {
        let previous_player = self.player().other();

        let be_extra_careful = previous_danger.check
            || m.piece.piece == Piece::King
            || matches!(m.move_type, MoveType::Capture(Capture::EnPassant { .. }))
            || previous_danger.piece_is_pinned(m.start_index);

        if be_extra_careful {
            let king_index = self
                .bitboards()
                .index_of_piece(previous_player, Piece::King);
            let illegal_move =
                index_in_danger(previous_player, king_index, self.bitboards()).unwrap();
            if illegal_move {
                return Legal::No;
            }
        }
        Legal::Yes
    }

    pub fn make_move(&mut self, m: Move) -> ErrorResult<()> {
        let player = m.piece.player;
        let enemy = player.other();

        #[cfg(test)]
        {
            m.check_simple_move_conversion(self)?;
        }

        for &castling_side in CASTLING_SIDES.iter() {
            if m.piece.piece != Piece::King && m.piece.piece != Piece::Rook {
                continue;
            }
            let player_can_castle = self.can_castle()[player][castling_side];
            if !player_can_castle {
                continue;
            }

            self.board.update_castling(
                player,
                castling_side,
                castling_allowed_after_move(player, castling_side, m.start_index),
            );
        }

        for &castling_side in CASTLING_SIDES.iter() {
            if let MoveType::Capture(Capture::Take { .. }) = m.move_type {
                let enemy_can_castle = self.can_castle()[enemy][castling_side];
                if !enemy_can_castle {
                    continue;
                }
                self.board.update_castling(
                    enemy,
                    castling_side,
                    castling_allowed_after_move(enemy, castling_side, m.end_index),
                );
            }
        }

        self.board.update_en_passant(None);

        match m.move_type {
            MoveType::Invalid => {
                return self.err(&format!("invalid move ({:?})", m));
            }
            MoveType::Quiet(q) => {
                if self.bitboards().is_occupied(m.end_index) {
                    return self.err(&format!(
                        "invalid quiet move ({:?}): end index {} is occupied",
                        m, m.end_index
                    ));
                }
                if self.bitboards().piece_at_index(m.start_index) != Some(m.piece) {
                    return self.err(&format!(
                        "invalid quiet move ({:?}): piece isn't at start index {}",
                        m, m.start_index
                    ));
                }

                match q {
                    Quiet::Move => {
                        self.board.clear_square(m.start_index, m.piece)?;
                        self.board.set_square(m.end_index, m.piece)?;
                    }
                    Quiet::Castle {
                        rook_start,
                        rook_end,
                    } => {
                        if m.piece.piece != Piece::King {
                            return self
                                .err(&format!("invalid castle move ({:?}), piece isn't king", m));
                        }
                        if self.bitboards().piece_at_index(rook_start)
                            != Some(PlayerPiece::new(player, Piece::Rook))
                        {
                            return self.err(&format!(
                                "invalid castle move ({:?}), rook isn't at rook start",
                                m
                            ));
                        }

                        self.board.clear_square(m.start_index, m.piece)?;
                        self.board.set_square(m.end_index, m.piece)?;

                        self.board
                            .clear_square(rook_start, PlayerPiece::new(player, Piece::Rook))?;
                        self.board
                            .set_square(rook_end, PlayerPiece::new(player, Piece::Rook))?;
                    }
                    Quiet::PawnSkip { skipped_index } => {
                        self.board.clear_square(m.start_index, m.piece)?;
                        self.board.set_square(m.end_index, m.piece)?;

                        self.board.update_en_passant(Some(skipped_index));
                    }
                }
            }
            MoveType::Capture(c) => match c {
                Capture::EnPassant { taken_index } => {
                    let taken_piece = PlayerPiece::new(enemy, Piece::Pawn);
                    if self.bitboards().piece_at_index(taken_index) != Some(taken_piece) {
                        return self.err(&format!(
                            "invalid en-passant {:?}: taken piece isn't enemy pawn",
                            m
                        ));
                    }
                    self.board.clear_square(taken_index, taken_piece)?;

                    self.board.clear_square(m.start_index, m.piece)?;
                    self.board.set_square(m.end_index, m.piece)?;
                }
                Capture::Take { taken_piece } => {
                    if taken_piece.player != enemy {
                        return self.err(&format!(
                            "invalid en-passant {:?}: taken piece isn't enemy piece",
                            m
                        ));
                    }
                    self.board.clear_square(m.end_index, taken_piece)?;

                    self.board.clear_square(m.start_index, m.piece)?;
                    self.board.set_square(m.end_index, m.piece)?;
                }
            },
        }

        if let Some(promo_piece) = m.promotion {
            let promo_piece = PlayerPiece::new(player, promo_piece);
            if !types::PROMOTION_PIECES.contains(&promo_piece.piece) {
                return self.err(
                    format!(
                        "invalid pawn promotion: promotion piece {} isn't a promotion piece",
                        promo_piece,
                    )
                    .as_str(),
                );
            }
            self.board.clear_square(m.end_index, m.piece)?;
            self.board.set_square(m.end_index, promo_piece)?;
        }

        self.board.update_player(enemy)?;
        self.half_moves_since_pawn_or_capture += 1;
        if self.player() == Player::White {
            self.full_moves_total += 1;
        }

        Ok(())
    }
}

#[test]
fn test_en_passant_1() {
    let game = Game::from_position_uci("position startpos moves e2e4 a7a5 e4e5 a5a4").unwrap();
    let m = game.move_from_str("b2b4");
    assert_eq!(
        m.unwrap(),
        Move {
            piece: PlayerPiece {
                player: Player::White,
                piece: Piece::Pawn
            },
            start_index: index_from_file_rank_str("b2").unwrap(),
            end_index: index_from_file_rank_str("b4").unwrap(),
            move_type: MoveType::Quiet(Quiet::PawnSkip {
                skipped_index: index_from_file_rank_str("b3").unwrap()
            }),
            promotion: None,
        }
    );
}

#[test]
fn test_en_passsant_2() {
    let game = Game::from_position_uci("position startpos moves e2e4 a7a5 e4e5 a5a4 b2b4").unwrap();
    assert_eq!(
        game.en_passant(),
        Some(index_from_file_rank_str("b3").unwrap())
    );

    let mut moves = vec![];
    all_moves(&mut moves, game.player(), &game, MoveOptions::default()).unwrap();

    for m in moves.iter() {
        let mut next_game = game.clone();
        next_game.make_move(*m).unwrap();
    }
}

#[test]
fn test_en_passant_3() {
    let game =
        Game::from_fen("rnbqkbnr/p2ppppp/2p5/Pp6/8/8/1PPPPPPP/RNBQKBNR w KQkq b6 4 3").unwrap();
    let m = game.move_from_str("a5b6");
    assert_eq!(
        m.unwrap(),
        Move {
            piece: PlayerPiece {
                player: Player::White,
                piece: Piece::Pawn
            },
            start_index: index_from_file_rank_str("a5").unwrap(),
            end_index: index_from_file_rank_str("b6").unwrap(),
            move_type: MoveType::Capture(Capture::EnPassant {
                taken_index: index_from_file_rank_str("b5").unwrap()
            }),
            promotion: None,
        }
    );
}

#[test]
fn test_castling_disallowed() {
    let game = Game::from_position_uci(
        "position fen rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8 moves a2a3 f2h1",
    )
    .unwrap();

    assert_eq!(
        false,
        game.can_castle()[game.player()][CastlingSide::Kingside]
    );
    assert_eq!(
        true,
        game.can_castle()[game.player()][CastlingSide::Queenside]
    );

    let mut moves = vec![];
    all_moves(&mut moves, game.player(), &game, MoveOptions::default()).unwrap();

    let mut castling_count = 0;
    for m in moves.iter() {
        if let MoveType::Quiet(Quiet::Castle { .. }) = m.move_type {
            castling_count += 1;
        }
    }

    assert_eq!(castling_count, 0);
}

#[test]
fn test_map_results() {
    let e = err("err");

    {
        let results = vec![Ok(1), Ok(2), Ok(3), Ok(4), Err(e.clone())];
        let results = results.into_iter();
        let results = results.map(|r| r);
        let results = results.map_results(|i| i + 1);
        let results: Vec<_> = results.collect();
        assert_eq!(
            results,
            vec![Ok(1 + 1), Ok(2 + 1), Ok(3 + 1), Ok(4 + 1), Err(e.clone())]
        )
    }
}

#[test]
fn test_should_discard_invalid_simple_moves() {
    let values = vec![(
        "r2qkb1r/ppp1pppp/2n2n2/3p4/3P4/2N1PP1P/PPP2P2/R1BQKB1R b KQkq - 11 6",
        "f8d6",
    )];
    for (fen, move_str) in values {
        let game = Game::from_fen(fen).unwrap();
        let simple_move = SimpleMove::from_str(move_str).unwrap();

        assert!(
            simple_move.to_move(&game).unwrap().is_none(),
            "should discard move {} for game {:#?}",
            simple_move,
            game
        );
    }
}
