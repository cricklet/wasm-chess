use crate::{
    bitboard::{Bitboard, BoardIndex, ForPlayer, Bitboards, index_from_file_rank_str},
    game::CanCastleOnSide,
    helpers::{err_result, ErrorResult},
    types::Player,
};

pub struct FenDefinition {
    pub bitboards: Bitboards,
    pub player: Player,
    pub can_castle: ForPlayer<CanCastleOnSide>,
    pub en_passant: Option<BoardIndex>,
    pub half_moves_since_pawn_or_capture: usize,
    pub full_moves_total: usize,
}

impl FenDefinition {
    pub fn from(fen: &str) -> ErrorResult<FenDefinition> {
        if fen == "startpos" {
            return FenDefinition::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        }

        let split = fen.split(' ');
        let split = split.filter(|v| !v.is_empty()).collect::<Vec<_>>();

        // parse a string like "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"

        if split.len() == 0 {
            return err_result(&format!("empty fen {}", fen));
        }

        let mut definition = FenDefinition {
            bitboards: Bitboards::new(),
            player: Player::White,
            can_castle: ForPlayer::new(CanCastleOnSide::default(), CanCastleOnSide::default()),
            en_passant: None,
            half_moves_since_pawn_or_capture: 0,
            full_moves_total: 0,
        };

        definition.bitboards = Bitboards::from_fen(split[0])?;
        if split.len() <= 1 {
            return Ok(definition);
        }

        definition.player = match split[1] {
            "w" => Player::White,
            "b" => Player::Black,
            _ => return err_result(&format!("invalid player {}", split[1])),
        };

        if split.len() <= 2 {
            return Ok(definition);
        }

        let can_castle_on_side_for_player = ForPlayer::<CanCastleOnSide>::from_str(split[2]);
        definition.can_castle = can_castle_on_side_for_player?;

        if split.len() <= 3 {
            return Ok(definition);
        }

        let en_passant_str = split[3];
        definition.en_passant = match en_passant_str {
            "-" => None,
            _ => Some(index_from_file_rank_str(en_passant_str)?),
        };

        if split.len() <= 4 {
            return Ok(definition);
        }

        definition.half_moves_since_pawn_or_capture = match split[4].parse::<usize>() {
            Ok(half_moves_since_pawn_or_capture) => half_moves_since_pawn_or_capture,
            Err(e) => {
                return err_result(&format!(
                    "error parsing half moves since pawn or capture: {}",
                    e
                ))
            }
        };

        if split.len() <= 5 {
            return Ok(definition);
        }

        definition.full_moves_total = match split[5].parse::<usize>() {
            Ok(full_moves_total) => full_moves_total,
            Err(e) => return err_result(&format!("error parsing full moves total: {}", e)),
        };

        if split.len() > 6 {
            return err_result(&format!("invalid fen {}", fen));
        }

        Ok(definition)
    }
}
