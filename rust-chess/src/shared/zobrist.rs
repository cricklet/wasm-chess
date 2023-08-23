use std::{
    collections::{HashMap, HashSet},
    fmt::{Display, Formatter},
    hash::Hash,
    sync::Mutex,
};

use lazy_static::lazy_static;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use strum::IntoEnumIterator;

use crate::{
    bitboard::{Bitboards, BoardIndex, ForPlayer},
    fen::FenDefinition,
    game::{CanCastleOnSide, Game},
    helpers::{err_result, ErrorResult, Joinable, OptionResult},
    moves::{all_moves, Move, MoveOptions},
    simple_move::SimpleMove,
    types::{CastlingSide, Piece, Player, PlayerPiece},
};

lazy_static! {
    static ref RANDOMIZER: Mutex<ChaCha8Rng> = {
        let r = ChaCha8Rng::seed_from_u64(32879419);
        Mutex::new(r)
    };
    static ref ZOBRIST_PIECE_AT_SQUARE: [[u64; 64]; 12] = {
        let mut arr = [[0; 64]; 12];
        let mut r = RANDOMIZER.lock().unwrap();
        for player in Player::iter() {
            for piece in Piece::iter() {
                let player_piece = PlayerPiece::new(player, piece);
                for board_index in 0..64 {
                    arr[player_piece.to_usize()][board_index] = r.gen();
                }
            }
        }
        for row in arr {
            for v in row {
                if v == 0 {
                    panic!("failed to initialize zobrist array");
                }
            }
        }
        arr
    };
    static ref ZOBRIST_SIDE_TO_MOVE: u64 = {
        let mut r = RANDOMIZER.lock().unwrap();
        r.gen()
    };
    static ref ZOBRIST_CASTLING_RIGHTS: [u64; 4] = {
        let mut arr = [0; 4];
        let mut r = RANDOMIZER.lock().unwrap();
        for i in 0..4 {
            arr[i] = r.gen();
        }
        arr
    };
    static ref ZOBRIST_EN_PASSANT: [u64; 8] = {
        let mut arr = [0; 8];
        let mut r = RANDOMIZER.lock().unwrap();
        for i in 0..8 {
            arr[i] = r.gen();
        }
        arr
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ZobristHash {
    value: u64,
}

impl Display for ZobristHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", self.value)
    }
}

impl ZobristHash {
    pub fn value(self) -> u64 {
        self.value
    }

    pub fn from(
        bitboards: &Bitboards,
        player: Player,
        can_castle: ForPlayer<CanCastleOnSide>,
        en_passant: Option<BoardIndex>,
    ) -> Self {
        let mut hash = 0;
        for board_index in 0..64 {
            let piece = bitboards.piece_at_index(BoardIndex::from(board_index));
            match piece {
                None => {}
                Some(player_piece) => {
                    hash ^= ZOBRIST_PIECE_AT_SQUARE[player_piece.to_usize()][board_index];
                }
            }
        }
        if player == Player::White {
            hash ^= *ZOBRIST_SIDE_TO_MOVE;
        }
        for (i, &player) in [Player::White, Player::Black].iter().enumerate() {
            for (j, &side) in [CastlingSide::Kingside, CastlingSide::Queenside]
                .iter()
                .enumerate()
            {
                if can_castle[player][side] {
                    hash ^= ZOBRIST_CASTLING_RIGHTS[2 * i + j];
                }
            }
        }
        if let Some(en_passant) = en_passant {
            hash ^= ZOBRIST_EN_PASSANT[en_passant.file()];
        }
        Self { value: hash }
    }

    pub fn on_castling_change(&mut self, player: Player, side: CastlingSide) {
        self.value ^= ZOBRIST_CASTLING_RIGHTS[2 * player.to_usize() + side.to_usize()];
    }
    pub fn on_player_change(&mut self) {
        self.value ^= *ZOBRIST_SIDE_TO_MOVE;
    }
    pub fn on_en_passant_change(&mut self, target: BoardIndex) {
        self.value ^= ZOBRIST_EN_PASSANT[target.file()];
    }
    pub fn on_update_square(&mut self, index: BoardIndex, piece: PlayerPiece) {
        self.value ^= ZOBRIST_PIECE_AT_SQUARE[piece.to_usize()][index.i];
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct ZobristHistory {
    seen: HashMap<ZobristHash, u8>,
    move_stack: Vec<ZobristHash>,
    draw_stack: Vec<IsDraw>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IsDraw {
    #[default]
    No,
    Almost,
    Yes,
}

impl ZobristHistory {
    pub fn new() -> Self {
        Self {
            seen: HashMap::new(),
            move_stack: vec![],
            draw_stack: vec![],
        }
    }

    pub fn seen(&self) -> &HashMap<ZobristHash, u8> {
        &self.seen
    }

    pub fn update(&mut self, startpos: String, moves: &[String]) -> IsDraw {
        self.seen.clear();
        self.move_stack.clear();
        self.draw_stack.clear();

        let mut game = Game::from_fen(&startpos).unwrap();
        self.add(game.zobrist());

        for (_i, mv) in moves.iter().enumerate() {
            let mv = game.move_from_str(mv).unwrap();
            game.make_move(mv).unwrap();
            self.add(game.zobrist());

            if self.is_draw() == IsDraw::Yes {
                return IsDraw::Yes;
            }
        }
        IsDraw::No
    }

    pub fn is_draw(&self) -> IsDraw {
        *self.draw_stack.last().unwrap_or(&IsDraw::No)
    }

    pub fn add(&mut self, zobrist: ZobristHash) {
        let entry = self.seen.entry(zobrist).or_insert(0);
        *entry += 1;
        let entry = *entry;

        self.move_stack.push(zobrist);

        let is_draw = if self.is_draw() == IsDraw::Yes || entry >= 3 {
            IsDraw::Yes
        } else if entry == 2 {
            IsDraw::Almost
        } else {
            IsDraw::No
        };

        self.draw_stack.push(is_draw);
    }

    pub fn pop(&mut self) -> ErrorResult<()> {
        let zobrist = self.move_stack.pop();
        let _ = self.draw_stack.pop();

        if let Some(zobrist) = zobrist {
            if let Some(entry) = self.seen.get_mut(&zobrist) {
                *entry -= 1;
                return Ok(());
            }
        }
        err_result(&format!("zobrist hash not found in history"))
    }
}

#[test]
fn test_draw_detection() {
    let uci = "position startpos moves d2d4 d7d5 b1c3 b8c6 g1f3 g8f6 c1g5 f6e4 e2e3 e4g5 f3g5 e7e5 f2f4 f7f6 g5f3 e5e4 f3d2 c8e6 d2e4 d5e4 d4d5 e6d5 c3d5 f8d6 g2g3 d8d7 f1g2 f6f5 d1d2 e8c8 e1c1 c6e7 d2a5 e7c6 a5d2 a7a5 h1e1 c6b4 d5b4 a5b4 c2c3 d7e6 d2d5 e6d5 d1d5 g7g6 c3b4 d6b4 d5d8 h8d8 e1d1 d8d6 d1d6 c7d6 b2b3 d6d5 a2a4 h7h5 c1d1 b4c3 g2f1 c8d8 h2h4 b7b6 f1b5 d8e7 b5c6 d5d4 e3d4 c3d4 d1e2 e7f6 b3b4 f6e7 e2f1 e7e6 b4b5 e6e7 c6d5 e4e3 d5b3 e7f6 f1e1 f6e7 e1e2 e7f6 e2d3 f6e7 d3c2 e3e2 c2d2 d4f2 d2e2 f2g3 e2f1 e7f6 f1g1 g3f4 g1f2 g6g5 h4g5 f6g5 f2f1 h5h4 f1g2 f4g3 g2f3 g5f6 f3g2 f6g5 g2f3 g5f6 f3g2 f6g5";
    let (position, moves) = FenDefinition::split_uci(uci).unwrap();
    let mut history = ZobristHistory::new();
    history.update(position, &moves);
    assert_eq!(history.is_draw(), IsDraw::Yes);
}

#[test]
fn test_draw_simple() {
    let position = "8/8/3k4/8/8/8/3K4/8 w".to_string();
    let moves: Vec<String> = vec![
        "d2c1", "d6c5", "c1d2", "c5d6", "d2c1", "d6c5", "c1d2", "c5d6",
    ]
    .iter()
    .map(|v| v.to_string())
    .collect();

    let mut history = ZobristHistory::new();
    history.update(position, &moves);
    assert_eq!(history.is_draw(), IsDraw::Yes);
}
