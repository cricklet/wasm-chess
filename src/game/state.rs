use strum::IntoEnumIterator;

use crate::bitboards::{self, castling_allowed_after_move, index_to_file_rank_str, Bitboards};
use crate::bitboards::{index_from_file_rank_str, ForPlayer};
use crate::helpers::{err, ErrorResult};
use crate::moves::{Move, MoveType, Quiet};
use crate::types::{self, player_and_piece_to_fen_char, CastlingSide, Piece};

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
                        _ => return err(&format!("invalid castling side {}", c)),
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

#[derive(Debug, Copy, Clone)]
pub struct Game {
    pub board: bitboards::Bitboards,
    pub player: types::Player,
    pub can_castle: ForPlayer<CanCastleOnSide>,
    pub en_passant: Option<usize>,
    pub half_moves_since_pawn_or_capture: usize,
    pub full_moves_total: usize,
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
        err::<T>(&format!("{}\n\n{}", msg, self.pretty()))
    }

    pub fn pretty(&self) -> String {
        format!("{}\n{}", self.to_fen(), self.board.pretty())
    }

    pub fn to_fen(&self) -> String {
        format!(
            "{} {} {} {} {} {}",
            self.board.to_fen(),
            self.player.to_fen(),
            self.can_castle.to_fen(),
            self.en_passant
                .map(index_to_file_rank_str)
                .unwrap_or("-".to_string()),
            self.half_moves_since_pawn_or_capture,
            self.full_moves_total
        )
    }

    pub fn from_fen(fen: &str) -> ErrorResult<Game> {
        let mut game = Game::new();

        let split: Vec<&str> = fen.split(' ').collect();

        // parse a string like "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"

        if split.len() <= 0 {
            return err(&format!("empty fen {}", fen));
        }

        let board = Bitboards::from_fen(split[0]);
        game.board = board?;

        if split.len() <= 1 {
            return Ok(game);
        }

        game.player = match split[1] {
            "w" => types::Player::White,
            "b" => types::Player::Black,
            _ => return err(&format!("invalid player {}", split[1])),
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
                return err(&format!(
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
            Err(e) => return err(&format!("error parsing full moves total: {}", e)),
        };

        if split.len() > 6 {
            return err(&format!("invalid fen {}", fen));
        }

        Ok(game)
    }

    pub fn make_move(&mut self, m: Move) -> ErrorResult<()> {
        let player = m.player;
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
                        "invalid quiet move: end index {} is occupied",
                        index_to_file_rank_str(m.end_index)
                    ));
                }
                if self.board.piece_at_index(m.start_index) != Some((player, m.piece)) {
                    return self.err(&format!(
                        "invalid quiet move: piece {} isn't at start index {}",
                        player_and_piece_to_fen_char((player, m.piece)),
                        index_to_file_rank_str(m.start_index)
                    ));
                }

                match q {
                    Quiet::Move => {
                        self.board.clear_square(m.start_index, player, m.piece);
                        self.board.set_square(m.end_index, player, m.piece);
                    }
                    Quiet::Castle {
                        rook_start,
                        rook_end,
                    } => {
                        if m.piece != Piece::King {
                            return self.err("invalid castle move, piece isn't king");
                        }
                        if self.board.piece_at_index(rook_start) != Some((player, Piece::Rook)) {
                            return self.err("invalid castle move, rook isn't at rook start");
                        }

                        self.board.clear_square(m.start_index, player, m.piece);
                        self.board.set_square(m.end_index, player, m.piece);

                        self.board.clear_square(rook_start, player, Piece::Rook);
                        self.board.set_square(rook_end, player, Piece::Rook);
                    }
                    Quiet::PawnSkip { skipped_index } => {
                        self.board.clear_square(m.start_index, player, m.piece);
                        self.board.set_square(m.end_index, player, m.piece);

                        self.en_passant = Some(skipped_index);
                    }
                    Quiet::PawnPromotion { promotion_piece } => {
                        if !types::PROMOTION_PIECES.contains(&promotion_piece) {
                            return self.err(format!(
                                "invalid pawn promotion: promotion piece {} isn't a promotion piece",
                                player_and_piece_to_fen_char((player, promotion_piece))
                            ).as_str());
                        }
                        self.board.clear_square(m.start_index, player, m.piece);
                        self.board.set_square(m.end_index, player, promotion_piece);
                    }
                }
            }
            MoveType::Capture(c) => match c {
                crate::moves::Capture::EnPassant { taken_index } => {
                    if self.board.piece_at_index(taken_index) != Some((enemy, Piece::Pawn)) {
                        return self.err("invalid en-passant: taken piece isn't enemy pawn");
                    }
                    self.board.clear_square(taken_index, enemy, Piece::Pawn);

                    self.board.clear_square(m.start_index, player, m.piece);
                    self.board.set_square(m.end_index, player, m.piece);
                }
                crate::moves::Capture::Take { taken_piece } => {
                    let (taken_player, taken_piece) = taken_piece;
                    if taken_player != enemy {
                        return self.err("invalid capture: taken piece isn't enemy piece");
                    }
                    self.board
                        .clear_square(m.end_index, taken_player, taken_piece);

                    self.board.clear_square(m.start_index, player, m.piece);
                    self.board.set_square(m.end_index, player, m.piece);
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
