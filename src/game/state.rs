use strum::IntoEnumIterator;

use crate::bitboards::{self, castling_allowed_after_move, Bitboards, BoardIndex};
use crate::bitboards::{index_from_file_rank_str, ForPlayer};
use crate::helpers::{err, err_result, ErrorResult};
use crate::moves::{
    all_moves, index_in_danger, Move, MoveType, OnlyCaptures, OnlyQueenPromotion, Quiet,
};
use crate::types::{self, CastlingSide, Piece, PlayerPiece};

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
    pub board: bitboards::Bitboards,
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
        write!(f, "{}", self,)
    }
}

impl Game {
    pub fn new() -> Game {
        Game {
            board: bitboards::state::Bitboards::new(),
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
                let fen_part = &uci_line["fen".len()..].trim();
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

        let split: Vec<&str> = fen.split(' ').collect();

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
        let moves = all_moves(self.player, self, OnlyCaptures::NO, OnlyQueenPromotion::NO);
        for m in moves {
            let m = m.unwrap();
            let mut next_game = *self;
            next_game.make_move(m).unwrap();

            let king_index = next_game.board.index_of_piece(self.player, Piece::King);
            let illegal_move = index_in_danger(self.player, king_index, &next_game).unwrap();

            if illegal_move {
                continue;
            }

            if m.to_uci() == move_str {
                return Some(m);
            }
        }

        None
    }

    pub fn make_move(&mut self, m: Move) -> ErrorResult<()> {
        let player = m.piece.player;
        let enemy = player.other();

        for castling_side in CastlingSide::iter() {
            self.can_castle[player][castling_side] &=
                castling_allowed_after_move(player, castling_side, m.start_index);
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
                    Quiet::PawnPromotion { promotion_piece } => {
                        let promotion_piece = PlayerPiece::new(player, promotion_piece);
                        if !types::PROMOTION_PIECES.contains(&promotion_piece.piece) {
                            return self.err(format!(
                                "invalid pawn promotion: promotion piece {} isn't a promotion piece",
                                promotion_piece,
                            ).as_str());
                        }
                        self.board.clear_square(m.start_index, m.piece);
                        self.board.set_square(m.end_index, promotion_piece);
                    }
                }
            }
            MoveType::Capture(c) => match c {
                crate::moves::Capture::EnPassant { taken_index } => {
                    let taken_piece = PlayerPiece::new(enemy, Piece::Pawn);
                    if self.board.piece_at_index(taken_index) != Some(taken_piece) {
                        return self.err("invalid en-passant: taken piece isn't enemy pawn");
                    }
                    self.board.clear_square(taken_index, taken_piece);

                    self.board.clear_square(m.start_index, m.piece);
                    self.board.set_square(m.end_index, m.piece);
                }
                crate::moves::Capture::Take { taken_piece } => {
                    if taken_piece.player != enemy {
                        return self.err("invalid capture: taken piece isn't enemy piece");
                    }
                    self.board.clear_square(m.end_index, taken_piece);

                    self.board.clear_square(m.start_index, m.piece);
                    self.board.set_square(m.end_index, m.piece);
                }
            },
        }

        self.player = enemy;
        self.half_moves_since_pawn_or_capture += 1;
        if self.player == types::Player::White {
            self.full_moves_total += 1;
        }

        Ok(())
    }
}
