
use std::collections::{HashSet, HashMap};

use derive_getters::Getters;
use crate::{bitboard::{Bitboards, ForPlayer, BoardIndex}, types::{Player, CastlingSide, PlayerPiece}, game::{CanCastleOnSide, Game}, zobrist::ZobristHash, helpers::{ErrorResult, err_result}, perft::traverse_game_callback};


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

    pub fn update_player(&mut self, player: Player) -> ErrorResult<()> {
        if self.player == player {
            return err_result("cannot update player to the same player");
        }
        self.player = player;
        self.zobrist.on_player_change();
        Ok(())
    }

    pub fn update_en_passant(&mut self, target: Option<BoardIndex>) {
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

#[test]
fn test_no_early_zobrist_repeats() {
    let game = Game::from_fen("startpos").unwrap();

    let mut hash_for_board_fen: HashMap<ZobristHash, String> = HashMap::new();
    let mut moves_stack = vec![];

    traverse_game_callback(&mut moves_stack, &game, 0, 4, &mut |params| {
        let hash = params.game.zobrist();
        let board_fen = params.game.bitboards().to_fen();

        if let Some(previous) = hash_for_board_fen.get(&hash) {
            if previous != &board_fen {
                panic!(
                    "found duplicate zobrist hash for board: {} and {}",
                    previous, board_fen
                );
            }
        } else {
            hash_for_board_fen.insert(hash, board_fen);
        }
    }).unwrap();
    
}