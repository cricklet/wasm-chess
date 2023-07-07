use crate::bitboards;
use crate::types;

#[derive(Debug)]
pub struct Game {
    pub board: bitboards::Bitboards,
    pub player: types::Player,
    pub can_castle_on_side_for_player: [[bool; 2]; 2],
    pub en_passant: Option<i32>,
    pub half_moves_since_pawn_or_capture: i32,
    pub full_moves_total: i32,
}

impl Game {
    pub fn new() -> Game {
        Game {
            board: bitboards::state::Bitboards::new(),
            player: types::Player::White,
            can_castle_on_side_for_player: [[true, true], [true, true]],
            en_passant: None,
            half_moves_since_pawn_or_capture: 0,
            full_moves_total: 0,
        }
    }
}
