use std::collections::{HashMap, HashSet};

use crate::{
    bitboard::{Bitboards, BoardIndex, ForPlayer},
    game::{CanCastleOnSide, Game},
    helpers::{err_result, ErrorResult},
    perft::traverse_game_callback,
    types::{CastlingSide, Player, PlayerPiece},
    zobrist::ZobristHash,
};
use derive_getters::Getters;

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

    pub fn to_fen(&self) -> String {
        format!(
            "{} {} {} {}",
            self.bitboards().to_fen(),
            self.player().to_fen(),
            self.can_castle().to_fen(),
            self.en_passant()
                .map(|i| i.to_string())
                .unwrap_or("-".to_string()),
        )
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
            return err_result(&format!(
                "can only set cleared squares: {:#?}",
                self.bitboards
            ));
        }

        self.bitboards.set_square(index, piece);
        self.zobrist.on_update_square(index, piece);
        Ok(())
    }
}

#[test]
fn test_no_early_zobrist_repeats() {
    let game = Game::from_fen("startpos").unwrap();

    let mut hash_to_fen: HashMap<ZobristHash, String> = HashMap::new();
    let mut fen_to_hash: HashMap<String, ZobristHash> = HashMap::new();
    let mut moves_stack = vec![];

    traverse_game_callback(&mut moves_stack, &game, 0, 4, &mut |params| {
        let hash = params.game.zobrist();
        let fen = params.game.board().to_fen();

        if let Some(previous) = hash_to_fen.get(&hash) {
            if previous != &fen {
                panic!(
                    "found duplicate zobrist hash for board: {} and {}",
                    previous, fen
                );
            }
        } else {
            hash_to_fen.insert(hash, fen.clone());
        }

        if let Some(previous) = fen_to_hash.get(&fen) {
            if previous != &hash {
                panic!(
                    "found differing zobrist hashes {} and {} for {}",
                    previous, hash, fen
                );
            }
        } else {
            fen_to_hash.insert(fen, hash);
        }
    })
    .unwrap();
}

#[test]
fn test_zobrist_transposition_depth_4() {
    let mut game1 = Game::from_fen("startpos").unwrap();
    game1
        .make_move(game1.move_from_str("a2a3").unwrap())
        .unwrap();
    game1
        .make_move(game1.move_from_str("a7a6").unwrap())
        .unwrap();
    game1
        .make_move(game1.move_from_str("b2b3").unwrap())
        .unwrap();
    game1
        .make_move(game1.move_from_str("a6a5").unwrap())
        .unwrap();

    let mut game2 = Game::from_fen("startpos").unwrap();
    game2
        .make_move(game2.move_from_str("b2b3").unwrap())
        .unwrap();
    game2
        .make_move(game2.move_from_str("a7a6").unwrap())
        .unwrap();
    game2
        .make_move(game2.move_from_str("a2a3").unwrap())
        .unwrap();
    game2
        .make_move(game2.move_from_str("a6a5").unwrap())
        .unwrap();

    assert_eq!(game1.bitboards().to_fen(), game2.bitboards().to_fen());
    assert_eq!(game1.zobrist(), game2.zobrist());

    // if the player is different, so should the zobrist hash
    let mut game3 = Game::from_fen("startpos").unwrap();
    game3
        .make_move(game3.move_from_str("b2b3").unwrap())
        .unwrap();
    game3
        .make_move(game3.move_from_str("a7a5").unwrap())
        .unwrap();
    game3
        .make_move(game3.move_from_str("a2a3").unwrap())
        .unwrap();

    assert_ne!(game1.zobrist(), game3.zobrist());
}
