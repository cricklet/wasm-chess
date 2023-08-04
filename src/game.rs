use std::iter;

use strum::IntoEnumIterator;

use crate::bitboard::{self, castling_allowed_after_move, Bitboards, BoardIndex};
use crate::bitboard::{index_from_file_rank_str, ForPlayer};
use crate::danger::Danger;
use crate::helpers::*;
use crate::moves::{
    all_moves, index_in_danger, Capture, Move, MoveOptions, MoveType, OnlyCaptures,
    OnlyQueenPromotion, Quiet,
};
use crate::types::{self, CastlingSide, Piece, Player, PlayerPiece, CASTLING_SIDES};

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

#[derive(Copy, Clone)]
pub struct Game {
    pub board: bitboard::Bitboards,
    pub player: types::Player,
    pub can_castle: ForPlayer<CanCastleOnSide>,
    pub en_passant: Option<BoardIndex>,
    pub half_moves_since_pawn_or_capture: usize,
    pub full_moves_total: usize,
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}", self.to_fen(), self.board)
    }
}

impl std::fmt::Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}", self.to_fen(), self.board)
    }
}

impl Game {
    pub fn new() -> Game {
        Game {
            board: bitboard::state::Bitboards::new(),
            player: types::Player::White,
            can_castle: ForPlayer {
                white: Default::default(),
                black: Default::default(),
            },
            en_passant: None,
            half_moves_since_pawn_or_capture: 0,
            full_moves_total: 1,
        }
    }

    pub fn err<T>(&self, msg: &str) -> ErrorResult<T> {
        err_result::<T>(&format!("{}\n\n{}", msg, self))
    }

    pub fn to_fen(&self) -> String {
        format!(
            "{} {} {} {} {} {}",
            self.board.to_fen(),
            self.player.to_fen(),
            self.can_castle.to_fen(),
            self.en_passant
                .map(|i| i.to_string())
                .unwrap_or("-".to_string()),
            self.half_moves_since_pawn_or_capture,
            self.full_moves_total
        )
    }

    pub fn from_position_uci(uci_line: &str) -> ErrorResult<Game> {
        let uci_line = uci_line.trim();

        let position_prefix = "position";
        let moves_separator = "moves";

        if !uci_line.starts_with(position_prefix) {
            return err_result(&format!("invalid uci line {}", uci_line));
        }

        let position_str = uci_line[position_prefix.len()..].trim().to_string();
        let (position_str, moves_str) = if position_str.contains(moves_separator) {
            let split: Vec<&str> = position_str.split(moves_separator).collect();
            if split.len() != 2 {
                return err_result(&format!("invalid uci line {}", uci_line));
            }
            (split[0].trim(), split[1].trim())
        } else {
            (position_str.trim(), "")
        };

        let game: ErrorResult<Game> = {
            if position_str == "startpos" {
                Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            } else if position_str.starts_with("fen") {
                let fen_part = &position_str["fen".len()..].trim();
                Game::from_fen(fen_part)
            } else {
                err_result(&format!("invalid uci line {}", uci_line))
            }
        };
        let mut game = game?;

        let moves: Vec<&str> = moves_str.split(" ").filter(|m| !m.is_empty()).collect();
        for m in moves {
            let m = game
                .move_from_str(m)
                .ok_or(err(&format!("invalid move '{}'", m)))?;
            game.make_move(m)?;
        }

        Ok(game)
    }

    pub fn from_fen(fen: &str) -> ErrorResult<Game> {
        let mut game = Game::new();

        let split: Vec<&str> = fen.split(' ').filter(|v| !v.is_empty()).collect();

        // parse a string like "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"

        if split.len() <= 0 {
            return err_result(&format!("empty fen {}", fen));
        }

        let board = Bitboards::from_fen(split[0]);
        game.board = board?;

        if split.len() <= 1 {
            return Ok(game);
        }

        game.player = match split[1] {
            "w" => types::Player::White,
            "b" => types::Player::Black,
            _ => return err_result(&format!("invalid player {}", split[1])),
        };

        if split.len() <= 2 {
            return Ok(game);
        }

        let can_castle_on_side_for_player = ForPlayer::<CanCastleOnSide>::from_str(split[2]);
        game.can_castle = can_castle_on_side_for_player?;

        if split.len() <= 3 {
            return Ok(game);
        }

        let en_passant_str = split[3];
        game.en_passant = match en_passant_str {
            "-" => None,
            _ => Some(index_from_file_rank_str(en_passant_str)?),
        };

        if split.len() <= 4 {
            return Ok(game);
        }

        game.half_moves_since_pawn_or_capture = match split[4].parse::<usize>() {
            Ok(half_moves_since_pawn_or_capture) => half_moves_since_pawn_or_capture,
            Err(e) => {
                return err_result(&format!(
                    "error parsing half moves since pawn or capture: {}",
                    e
                ))
            }
        };

        if split.len() <= 5 {
            return Ok(game);
        }

        game.full_moves_total = match split[5].parse::<usize>() {
            Ok(full_moves_total) => full_moves_total,
            Err(e) => return err_result(&format!("error parsing full moves total: {}", e)),
        };

        if split.len() > 6 {
            return err_result(&format!("invalid fen {}", fen));
        }

        Ok(game)
    }

    pub fn move_from_str(&self, move_str: &str) -> Option<Move> {
        let moves = all_moves(self.player, self, MoveOptions::default());
        for m in moves {
            let m = m.unwrap();
            let mut next_game = *self;
            next_game.make_move(m).unwrap();

            let king_index = next_game.board.index_of_piece(self.player, Piece::King);
            let illegal_move = index_in_danger(self.player, king_index, &next_game.board).unwrap();

            if illegal_move {
                continue;
            }

            if m.to_uci() == move_str {
                return Some(m);
            }
        }

        None
    }

    pub fn for_each_pseudo_move(
        &self,
        options: MoveOptions,
    ) -> Box<dyn Iterator<Item = ErrorResult<(Game, Move)>> + '_> {
        let player: Player = self.player;
        let moves = all_moves(player, self, options);

        let games = moves.map(|m| -> ErrorResult<(Game, Move)> {
            let m = m?;

            let mut next_game = *self;
            next_game.make_move(m)?;

            Ok((next_game, m))
        });

        Box::new(games)
    }

    pub fn move_is_illegal(&self, m: &Move, previous_danger: &Danger) -> bool {
        let previous_player = self.player.other();

        let be_extra_careful = previous_danger.check
            || m.piece.piece == Piece::King
            || matches!(m.move_type, MoveType::Capture(Capture::EnPassant { .. }))
            || previous_danger.piece_is_pinned(m.start_index);

        if be_extra_careful {
            let king_index = self.board.index_of_piece(previous_player, Piece::King);
            let illegal_move = index_in_danger(previous_player, king_index, &self.board).unwrap();
            if illegal_move {
                return false;
            }
        }
        true
    }

    pub fn for_each_legal_move(
        &self,
        options: MoveOptions,
    ) -> Box<dyn Iterator<Item = ErrorResult<(Game, Move)>> + '_> {
        let player = self.player;
        let danger = match Danger::from(player, &self.board) {
            Ok(danger) => danger,
            Err(e) => return Box::new(iter::once(Err(e))),
        };

        self.for_each_legal_move_with_danger(danger, options)
    }

    pub fn for_each_legal_move_with_danger<'t>(
        &'t self,
        danger: Danger,
        options: MoveOptions,
    ) -> Box<dyn Iterator<Item = ErrorResult<(Game, Move)>> + 't> {
        let games = self.for_each_pseudo_move(options);

        let legal_games = games.filter_results(move |(next_game, m)| -> bool {
            next_game.move_is_illegal(m, &danger)
        });

        Box::new(legal_games)
    }

    pub fn make_move(&mut self, m: Move) -> ErrorResult<()> {
        let player = m.piece.player;
        let enemy = player.other();

        for &castling_side in CASTLING_SIDES.iter() {
            if m.piece.piece != Piece::King && m.piece.piece != Piece::Rook {
                continue;
            }
            let ref mut player_can_castle = self.can_castle[player][castling_side];
            if !*player_can_castle {
                continue;
            }
            *player_can_castle = castling_allowed_after_move(player, castling_side, m.start_index);
        }

        for &castling_side in CASTLING_SIDES.iter() {
            if let MoveType::Capture(Capture::Take { .. }) = m.move_type {
                let ref mut enemy_can_castle = self.can_castle[enemy][castling_side];
                if !*enemy_can_castle {
                    continue;
                }
                *enemy_can_castle &= castling_allowed_after_move(enemy, castling_side, m.end_index);
            }
        }

        self.en_passant = None;

        match m.move_type {
            MoveType::Quiet(q) => {
                if self.board.is_occupied(m.end_index) {
                    return self.err(&format!(
                        "invalid quiet move ({:?}): end index {} is occupied",
                        m, m.end_index
                    ));
                }
                if self.board.piece_at_index(m.start_index) != Some(m.piece) {
                    return self.err(&format!(
                        "invalid quiet move ({:?}): piece isn't at start index {}",
                        m, m.start_index
                    ));
                }

                match q {
                    Quiet::Move => {
                        self.board.clear_square(m.start_index, m.piece);
                        self.board.set_square(m.end_index, m.piece);
                    }
                    Quiet::Castle {
                        rook_start,
                        rook_end,
                    } => {
                        if m.piece.piece != Piece::King {
                            return self
                                .err(&format!("invalid castle move ({:?}), piece isn't king", m));
                        }
                        if self.board.piece_at_index(rook_start)
                            != Some(PlayerPiece::new(player, Piece::Rook))
                        {
                            return self.err(&format!(
                                "invalid castle move ({:?}), rook isn't at rook start",
                                m
                            ));
                        }

                        self.board.clear_square(m.start_index, m.piece);
                        self.board.set_square(m.end_index, m.piece);

                        self.board
                            .clear_square(rook_start, PlayerPiece::new(player, Piece::Rook));
                        self.board
                            .set_square(rook_end, PlayerPiece::new(player, Piece::Rook));
                    }
                    Quiet::PawnSkip { skipped_index } => {
                        self.board.clear_square(m.start_index, m.piece);
                        self.board.set_square(m.end_index, m.piece);

                        self.en_passant = Some(skipped_index);
                    }
                }
            }
            MoveType::Capture(c) => match c {
                crate::moves::Capture::EnPassant { taken_index } => {
                    let taken_piece = PlayerPiece::new(enemy, Piece::Pawn);
                    if self.board.piece_at_index(taken_index) != Some(taken_piece) {
                        return self.err(&format!(
                            "invalid en-passant {:?}: taken piece isn't enemy pawn",
                            m
                        ));
                    }
                    self.board.clear_square(taken_index, taken_piece);

                    self.board.clear_square(m.start_index, m.piece);
                    self.board.set_square(m.end_index, m.piece);
                }
                crate::moves::Capture::Take { taken_piece } => {
                    if taken_piece.player != enemy {
                        return self.err(&format!(
                            "invalid en-passant {:?}: taken piece isn't enemy piece",
                            m
                        ));
                    }
                    self.board.clear_square(m.end_index, taken_piece);

                    self.board.clear_square(m.start_index, m.piece);
                    self.board.set_square(m.end_index, m.piece);
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
            self.board.clear_square(m.end_index, m.piece);
            self.board.set_square(m.end_index, promo_piece);
        }

        self.player = enemy;
        self.half_moves_since_pawn_or_capture += 1;
        if self.player == types::Player::White {
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
        game.en_passant,
        Some(index_from_file_rank_str("b3").unwrap())
    );

    for m in all_moves(game.player, &game, MoveOptions::default()) {
        let mut next_game = game;
        next_game.make_move(m.unwrap()).unwrap();
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

    assert_eq!(false, game.can_castle[game.player][CastlingSide::Kingside]);
    assert_eq!(true, game.can_castle[game.player][CastlingSide::Queenside]);

    let castling = all_moves(game.player, &game, MoveOptions::default())
        .map(|m| m.unwrap())
        .filter(|m| match m.move_type {
            MoveType::Quiet(Quiet::Castle { .. }) => true,
            _ => false,
        });

    assert_eq!(castling.count(), 0);
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
