use std::collections::HashSet;

use strum::IntoEnumIterator;

use crate::{helpers::*, moves::walk_potential_bb, types::*};

use super::*;

#[derive(Debug, Copy, Clone)]
pub struct PieceBitboards {
    pub pawns: Bitboard,
    pub rooks: Bitboard,
    pub knights: Bitboard,
    pub bishops: Bitboard,
    pub queens: Bitboard,
    pub kings: Bitboard,
}

impl PieceBitboards {
    pub fn new() -> PieceBitboards {
        PieceBitboards {
            pawns: 0,
            rooks: 0,
            knights: 0,
            bishops: 0,
            queens: 0,
            kings: 0,
        }
    }
}

impl std::ops::Index<Piece> for PieceBitboards {
    type Output = Bitboard;

    fn index(&self, index: Piece) -> &Self::Output {
        match index {
            Piece::Pawn => &self.pawns,
            Piece::Rook => &self.rooks,
            Piece::Knight => &self.knights,
            Piece::Bishop => &self.bishops,
            Piece::Queen => &self.queens,
            Piece::King => &self.kings,
        }
    }
}

impl std::ops::IndexMut<Piece> for PieceBitboards {
    fn index_mut(&mut self, index: Piece) -> &mut Self::Output {
        match index {
            Piece::Pawn => &mut self.pawns,
            Piece::Rook => &mut self.rooks,
            Piece::Knight => &mut self.knights,
            Piece::Bishop => &mut self.bishops,
            Piece::Queen => &mut self.queens,
            Piece::King => &mut self.kings,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ForPlayer<T> {
    pub white: T,
    pub black: T,
}

impl<T> ForPlayer<T> {
    pub fn new(white: T, black: T) -> ForPlayer<T> {
        ForPlayer { white, black }
    }
}

impl<T> std::ops::Index<Player> for ForPlayer<T> {
    type Output = T;

    fn index(&self, index: Player) -> &Self::Output {
        match index {
            Player::White => &self.white,
            Player::Black => &self.black,
        }
    }
}

impl<T> std::ops::IndexMut<Player> for ForPlayer<T> {
    fn index_mut(&mut self, index: Player) -> &mut Self::Output {
        match index {
            Player::White => &mut self.white,
            Player::Black => &mut self.black,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Bitboards {
    pub pieces: ForPlayer<PieceBitboards>,
    pub occupied: ForPlayer<Bitboard>,
    pub piece_at_index: [Option<PlayerPiece>; 64],

    pub in_check: ForPlayer<Option<bool>>,
    pub pinned: ForPlayer<Option<Bitboard>>,
}

impl std::fmt::Display for Bitboards {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        for rank in (0..8).rev() {
            s.push_str(format!("{} ", rank_to_char(rank)).as_str());
            for file in 0..8 {
                let index = index_from_file_rank(file, rank);

                let piece: Option<PlayerPiece> = self.piece_at_index(index);
                if let Some(PlayerPiece { player, piece }) = piece {
                    match player {
                        Player::White => match piece {
                            Piece::Pawn => s.push_str("♙ "),
                            Piece::Rook => s.push_str("♖ "),
                            Piece::Knight => s.push_str("♘ "),
                            Piece::Bishop => s.push_str("♗ "),
                            Piece::Queen => s.push_str("♕ "),
                            Piece::King => s.push_str("♔ "),
                        },
                        Player::Black => match piece {
                            Piece::Pawn => s.push_str("♟ "),
                            Piece::Rook => s.push_str("♜ "),
                            Piece::Knight => s.push_str("♞ "),
                            Piece::Bishop => s.push_str("♝ "),
                            Piece::Queen => s.push_str("♛ "),
                            Piece::King => s.push_str("♚ "),
                        },
                    }
                } else {
                    s.push_str("· ");
                }
            }
            s.push_str("\n");
        }

        s.push_str("  ");
        for file in 0..8 {
            s.push_str(format!("{} ", file_to_char(file)).as_str());
        }
        s.push_str("\n");

        write!(f, "{}", s)
    }
}

impl std::fmt::Debug for Bitboards {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\nwhite pinned:\n{}\nblack pinned:\n{}",
            self,
            self.pinned
                .white
                .map_or_else(|| "?".to_string(), bitboard_string),
            self.pinned
                .black
                .map_or_else(|| "?".to_string(), bitboard_string),
        )
    }
}

impl Bitboards {
    pub fn new() -> Bitboards {
        Bitboards {
            pieces: ForPlayer::new(PieceBitboards::new(), PieceBitboards::new()),
            occupied: ForPlayer::new(0, 0),
            piece_at_index: [None; 64],
            pinned: ForPlayer::new(None, None),
            in_check: ForPlayer::new(None, None),
        }
    }

    pub fn to_fen(&self) -> String {
        let mut fen = String::new();
        for rank in (0..8).rev() {
            let mut empty_count = 0;
            for file in 0..8 {
                let index = index_from_file_rank(file, rank);
                if let Some(piece) = self.piece_at_index(index) {
                    if empty_count > 0 {
                        fen.push_str(&empty_count.to_string());
                        empty_count = 0;
                    }
                    fen.push_str(piece.to_fen_char().to_string().as_str());
                } else {
                    empty_count += 1;
                }
            }
            if empty_count > 0 {
                fen.push_str(&empty_count.to_string());
            }
            if rank > 0 {
                fen.push('/');
            }
        }

        fen
    }

    pub fn from_fen(fen: &str) -> ErrorResult<Bitboards> {
        let mut bb = Bitboards::new();

        let mut rank = 7;
        let mut file = 0;
        for c in fen.chars() {
            if c == '/' {
                if file != 8 {
                    return err_result(&format!(
                        "not enough squares in rank ({:}, fen {:}",
                        FileRank { file, rank },
                        fen
                    ));
                }
                rank -= 1;
                file = 0;
            } else if let Some(d) = c.to_digit(10) {
                file += d as usize;
                if file > 8 {
                    return err_result(&format!(
                        "too many squares in rank ({:}, fen {:})",
                        FileRank { file, rank },
                        fen
                    ));
                }
            } else if let Some(piece) = PlayerPiece::from(c) {
                let index = index_from_file_rank(file, rank);
                bb.set_square(index, piece);
                file += 1;
            } else {
                return err_result(&format!(
                    "unknown character {:} ({:}, fen {:})",
                    c,
                    FileRank { file, rank },
                    fen
                ));
            }
        }

        Ok(bb)
    }

    pub fn piece_at_index(&self, index: BoardIndex) -> Option<PlayerPiece> {
        self.piece_at_index[index.i]
    }

    pub fn index_of_piece(&self, player: Player, piece: Piece) -> BoardIndex {
        let bb = self.pieces[player][piece];
        first_index_of_one(bb)
    }

    pub fn is_occupied_by_player(&self, index: BoardIndex, player: Player) -> bool {
        self.occupied[player] & single_bitboard(index) != 0
    }

    pub fn is_occupied(&self, index: BoardIndex) -> bool {
        self.occupied[Player::White] & single_bitboard(index) != 0
            || self.occupied[Player::Black] & single_bitboard(index) != 0
    }

    pub fn all_occupied(&self) -> Bitboard {
        self.occupied[Player::White] | self.occupied[Player::Black]
    }

    pub fn verify(&mut self) -> ErrorResult<()> {
        for file in 0..8 {
            for rank in 0..8 {
                let index = index_from_file_rank(file, rank);
                let single = single_bitboard(index);

                let mut found: HashSet<PlayerPiece> = HashSet::new();
                for piece in Piece::iter() {
                    for player in Player::iter() {
                        if single & self.pieces[player][piece] != 0 {
                            found.insert(PlayerPiece::new(player, piece));
                        }
                    }
                }

                if found.len() > 1 {
                    return err_result(&format!(
                        "more than one piece at {:} -- {:?}",
                        FileRank { file, rank },
                        found,
                    ));
                }

                if found.len() == 0 {
                    if self.is_occupied(index) {
                        return err_result(&format!(
                            "no piece at {:} but occupied -- {:?}",
                            FileRank { file, rank },
                            found,
                        ));
                    }
                    continue;
                }

                let piece = found.iter().next().unwrap();
                if self.is_occupied_by_player(index, other_player(piece.player)) {
                    return err_result(&format!(
                        "piece at {:} but occupied by other player -- {:?}",
                        FileRank { file, rank },
                        found,
                    ));
                } else if !self.is_occupied_by_player(index, piece.player) {
                    return err_result(&format!(
                        "piece at {:} but not occupied by player -- {:?}",
                        FileRank { file, rank },
                        found,
                    ));
                }

                if self.piece_at_index(index) != Some(*piece) {
                    return err_result(&format!(
                        "piece at {:} but not found in piece_at_index -- {:?}",
                        FileRank { file, rank },
                        found,
                    ));
                }
            }
        }

        Ok(())
    }

    fn in_danger_and_pinned_pieces(
        target: BoardIndex,
        occupied: Bitboard,
        player_occupied: Bitboard,
        enemy_occupied: Bitboard,
        walk_type: WalkType,
    ) -> (bool, Bitboard) {
        let dangers = moves_bb_for_piece_and_blockers(target, walk_type, occupied);
        let in_danger = dangers & enemy_occupied != 0;

        // Ignore direct threats
        let enemy_occupied = enemy_occupied & !dangers;

        let mut pinned = Bitboard::default();

        let player_potential_blockers = dangers & player_occupied;
        for potential_blocker_index in each_index_of_one(player_potential_blockers) {
            // If we remove this blocker, is the king in danger?
            let occupied = occupied & !single_bitboard(potential_blocker_index);
            let dangers = moves_bb_for_piece_and_blockers(target, walk_type, occupied);
            let in_danger = dangers & enemy_occupied != 0;

            if in_danger {
                pinned |= single_bitboard(potential_blocker_index);
            }
        }

        (in_danger, pinned)
    }

    pub fn update_pin_and_danger_maps(&mut self, player: Player) -> ErrorResult<()> {
        let enemy = other_player(player);

        let target = self.index_of_piece(player, Piece::King);

        let (in_danger_from_bishop, pinned_by_bishop) = Bitboards::in_danger_and_pinned_pieces(
            target,
            self.all_occupied(),
            self.occupied[player],
            self.pieces[enemy].bishops | self.pieces[enemy].queens,
            WalkType::Bishop,
        );

        let (in_danger_from_rook, pinned_by_rook) = Bitboards::in_danger_and_pinned_pieces(
            target,
            self.all_occupied(),
            self.occupied[player],
            self.pieces[enemy].rooks | self.pieces[enemy].queens,
            WalkType::Rook,
        );

        self.pinned[player] = Some(pinned_by_bishop | pinned_by_rook);
        self.in_check[player] = Some(in_danger_from_bishop || in_danger_from_rook);

        Ok(())
    }

    pub fn piece_is_pinned(&mut self, index: BoardIndex) -> ErrorResult<bool> {
        let PlayerPiece { player, piece: _ } =
            self.piece_at_index(index).ok_or(err("no piece at index"))?;
        if self.pinned[player] == None {
            self.update_pin_and_danger_maps(player)?;
        }

        let ref pinned = self.pinned[player].ok_or(err("failed to update pins"))?;
        Ok(pinned & single_bitboard(index) != 0)
    }

    pub fn in_check(&mut self, player: Player) -> ErrorResult<bool> {
        if self.in_check[player] == None {
            self.update_pin_and_danger_maps(player)?;
        }

        let ref in_check = self.in_check[player].ok_or(err("failed to update pins"))?;
        Ok(*in_check)
    }

    pub fn clear_square(&mut self, index: BoardIndex, piece: PlayerPiece) {
        self.pinned.white = None;
        self.pinned.black = None;
        self.in_check.white = None;
        self.in_check.black = None;

        let bb = single_bitboard(index);

        self.pieces[piece.player][piece.piece] &= !bb;
        self.occupied[piece.player] &= !bb;
        self.piece_at_index[index.i] = None;
    }

    pub fn set_square(&mut self, index: BoardIndex, piece: PlayerPiece) {
        self.pinned.white = None;
        self.pinned.black = None;
        self.in_check.white = None;
        self.in_check.black = None;

        let bb = single_bitboard(index);

        self.pieces[piece.player][piece.piece] |= bb;
        self.occupied[piece.player] |= bb;
        self.piece_at_index[index.i] = Some(piece);
    }
}

#[test]
pub fn test_starting_board() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
    let mut bb = Bitboards::from_fen(fen).unwrap();
    assert_eq!(
        format!("{}", bb).trim(),
        "\
        8 ♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜ \n\
        7 ♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟ \n\
        6 · · · · · · · · \n\
        5 · · · · · · · · \n\
        4 · · · · · · · · \n\
        3 · · · · · · · · \n\
        2 ♙ ♙ ♙ ♙ ♙ ♙ ♙ ♙ \n\
        1 ♖ ♘ ♗ ♕ ♔ ♗ ♘ ♖ \n\
        \u{20}\u{20}a b c d e f g h"
    );

    bb.verify().unwrap();

    let i = index_from_file_rank_str("e2").unwrap();
    bb.clear_square(i, PlayerPiece::new(Player::White, Piece::Pawn));

    assert_eq!(
        format!("{}", bb).trim(),
        "\
        8 ♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜ \n\
        7 ♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟ \n\
        6 · · · · · · · · \n\
        5 · · · · · · · · \n\
        4 · · · · · · · · \n\
        3 · · · · · · · · \n\
        2 ♙ ♙ ♙ ♙ · ♙ ♙ ♙ \n\
        1 ♖ ♘ ♗ ♕ ♔ ♗ ♘ ♖ \n\
        \u{20}\u{20}a b c d e f g h"
    );

    bb.verify().unwrap();
}

#[test]
pub fn test_pins() {
    let fen = "2k5/3r4/6b1/1N1N4/4N3/3K4/8/8";
    let mut bb = Bitboards::from_fen(fen).unwrap();

    assert_eq!(false, bb.in_check(Player::White).unwrap());

    assert_eq!(
        true,
        bb.piece_is_pinned(BoardIndex::from_str("d5").unwrap())
            .unwrap()
    );
    assert_eq!(
        true,
        bb.piece_is_pinned(BoardIndex::from_str("e4").unwrap())
            .unwrap()
    );
    assert_eq!(
        false,
        bb.piece_is_pinned(BoardIndex::from_str("b5").unwrap())
            .unwrap()
    );
}

#[test]
pub fn test_double_pin() {
    let fen = "2k5/3r4/q5b1/1N1N1B2/2N1N3/3K1rPr/2Pnn3/3q1b2";
    let mut bb = Bitboards::from_fen(fen).unwrap();

    assert_eq!(true, bb.in_check(Player::White).unwrap());

    assert_eq!(
        true,
        bb.piece_is_pinned(BoardIndex::from_str("d5").unwrap())
            .unwrap()
    );
    assert_eq!(
        false,
        bb.piece_is_pinned(BoardIndex::from_str("b5").unwrap())
            .unwrap()
    );
    assert_eq!(
        false,
        bb.piece_is_pinned(BoardIndex::from_str("c4").unwrap())
            .unwrap()
    );
    assert_eq!(
        false,
        bb.piece_is_pinned(BoardIndex::from_str("c2").unwrap())
            .unwrap()
    );
    assert_eq!(
        false,
        bb.piece_is_pinned(BoardIndex::from_str("e4").unwrap())
            .unwrap()
    );
    assert_eq!(
        false,
        bb.piece_is_pinned(BoardIndex::from_str("f5").unwrap())
            .unwrap()
    );
}

#[test]
pub fn test_check() {
    let fen = "2k5/3r4/6b1/1N1N4/4N3/q2K4/8/8";
    let mut bb = Bitboards::from_fen(fen).unwrap();

    assert_eq!(true, bb.in_check(Player::White).unwrap());

    assert_eq!(
        true,
        bb.piece_is_pinned(BoardIndex::from_str("d5").unwrap())
            .unwrap()
    );
    assert_eq!(
        true,
        bb.piece_is_pinned(BoardIndex::from_str("e4").unwrap())
            .unwrap()
    );
    assert_eq!(
        false,
        bb.piece_is_pinned(BoardIndex::from_str("b5").unwrap())
            .unwrap()
    );
}
