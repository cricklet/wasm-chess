use crate::bitboards;
use crate::bitboards::ForPlayer;
use crate::types::{self, CastlingSide};

#[derive(Debug, Default)]
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

#[derive(Debug)]
pub struct Game {
    pub board: bitboards::Bitboards,
    pub player: types::Player,
    pub can_castle_on_side_for_player: ForPlayer<CanCastleOnSide>,
    pub en_passant: Option<usize>,
    pub half_moves_since_pawn_or_capture: usize,
    pub full_moves_total: usize,
}

impl Game {
    pub fn new() -> Game {
        Game {
            board: bitboards::state::Bitboards::new(),
            player: types::Player::White,
            can_castle_on_side_for_player: ForPlayer {
                white: Default::default(),
                black: Default::default(),
            },
            en_passant: None,
            half_moves_since_pawn_or_capture: 0,
            full_moves_total: 0,
        }
    }
}
