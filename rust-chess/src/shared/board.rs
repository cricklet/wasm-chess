use derive_getters::Getters;

use crate::{bitboard::{Bitboards, ForPlayer, BoardIndex}, types::{Player, CastlingSide, PlayerPiece}, game::CanCastleOnSide, zobrist::ZobristHash, helpers::{ErrorResult, err_result}};


#[derive(Getters, Debug, Clone, Copy)]
pub struct Board {
    bitboards: Bitboards,
    player: Player,
    can_castle: ForPlayer<CanCastleOnSide>,
    en_passant: Option<BoardIndex>,

    zobrist: ZobristHash,
}

impl Board {
    pub fn new(
        bitboards: Bitboards,
        player: Player,
        can_castle: ForPlayer<CanCastleOnSide>,
        en_passant: Option<BoardIndex>,
    ) -> Self {
        let zobrist = ZobristHash::from(&bitboards, player, can_castle, en_passant);
        Self {
            bitboards,
            player,
            can_castle,
            en_passant,
            zobrist,
        }
    }

    pub fn update_castling(&mut self, player: Player, side: CastlingSide, can_castle: bool) {
        let c = &mut self.can_castle[player][side];
        if *c == can_castle {
            return;
        }

        *c = can_castle;
        self.zobrist.on_castling_change(player, side);
    }

    pub fn update_player(&mut self) {
        self.player = self.player.other();
        self.zobrist.on_player_change();
    }

    pub fn set_en_passant(&mut self, target: Option<BoardIndex>) {
        if self.en_passant == target {
            return;
        }

        if let Some(previous) = self.en_passant {
            self.zobrist.on_en_passant_change(previous);
        }

        self.en_passant = target;
        if let Some(previous) = self.en_passant {
            self.zobrist.on_en_passant_change(previous);
        }
    }

    pub fn clear_square(&mut self, index: BoardIndex, piece: PlayerPiece) -> ErrorResult<()> {
        if !self.bitboards.is_occupied(index) {
            return err_result(&format!(
                "can only clear occupied squares: {:#?}",
                self.bitboards
            ));
        }

        self.bitboards.clear_square(index, piece);
        self.zobrist.on_update_square(index, piece);
        Ok(())
    }

    pub fn set_square(&mut self, index: BoardIndex, piece: PlayerPiece) -> ErrorResult<()> {
        if self.bitboards.is_occupied(index) {
            return err_result(&format!("can only set cleared squares: {:#?}", self.bitboards));
        }

        self.bitboards.set_square(index, piece);
        self.zobrist.on_update_square(index, piece);
        Ok(())
    }
}