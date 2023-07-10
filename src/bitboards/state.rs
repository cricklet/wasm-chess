use std::collections::HashSet;

use strum::IntoEnumIterator;

use crate::{helpers::*, types::*};

use super::arithmetic::*;

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Bitboards {
    pub pieces: ForPlayer<PieceBitboards>,
    pub occupied: ForPlayer<Bitboard>,
}

impl Bitboards {
    pub fn new() -> Bitboards {
        Bitboards {
            pieces: ForPlayer::new(PieceBitboards::new(), PieceBitboards::new()),
            occupied: ForPlayer::new(0, 0),
        }
    }

    pub fn from_fen(fen: String) -> ErrorResult<Bitboards> {
        let mut bb = Bitboards::new();

        let mut rank = 7;
        let mut file = 0;
        for c in fen.chars() {
            if c == '/' {
                if file != 8 {
                    return Err(format!(
                        "not enough squares in rank ({:}, fen {:}",
                        file_rank_to_str(file, rank),
                        fen
                    ));
                }
                rank -= 1;
                file = 0;
            } else if let Some(d) = c.to_digit(10) {
                file += d as isize;
                if file > 8 {
                    return Err(format!(
                        "too many squares in rank ({:}, fen {:})",
                        file_rank_to_str(file, rank),
                        fen
                    ));
                }
            } else if let Some((player, piece_type)) = player_and_piece_from_fen_char(c) {
                let index = index_from_file_rank(file, rank);
                bb.set_square(index, player, piece_type);
                file += 1;
            } else {
                return Err(format!(
                    "unknown character {:} ({:}, fen {:})",
                    c,
                    file_rank_to_str(file, rank),
                    fen
                ));
            }
        }

        Ok(bb)
    }

    pub fn piece_at_index(&self, index: isize) -> Option<(Player, Piece)> {
        for piece in Piece::iter() {
            for player in Player::iter() {
                if single_bitboard(index) & self.pieces[player][piece] != 0 {
                    return Some((player, piece));
                }
            }
        }
        None
    }

    pub fn pretty(&self) -> String {
        let mut s = String::new();

        for rank in (0..8).rev() {
            for file in 0..8 {
                let index = index_from_file_rank(file, rank);

                let piece: Option<(Player, Piece)> = self.piece_at_index(index);
                if let Some((player, piece)) = piece {
                    match player {
                        Player::White => match piece {
                            Piece::Pawn => s.push_str("♙"),
                            Piece::Rook => s.push_str("♖"),
                            Piece::Knight => s.push_str("♘"),
                            Piece::Bishop => s.push_str("♗"),
                            Piece::Queen => s.push_str("♕"),
                            Piece::King => s.push_str("♔"),
                        },
                        Player::Black => match piece {
                            Piece::Pawn => s.push_str("♟"),
                            Piece::Rook => s.push_str("♜"),
                            Piece::Knight => s.push_str("♞"),
                            Piece::Bishop => s.push_str("♝"),
                            Piece::Queen => s.push_str("♛"),
                            Piece::King => s.push_str("♚"),
                        },
                    }
                } else {
                    s.push_str("·");
                }
            }
            s.push_str("\n");
        }

        s
    }

    pub fn is_occupied_by_player(&self, index: isize, player: Player) -> bool {
        self.occupied[player] & single_bitboard(index) != 0
    }

    pub fn is_occupied(&self, index: isize) -> bool {
        self.occupied[Player::White] & single_bitboard(index) != 0
            || self.occupied[Player::Black] & single_bitboard(index) != 0
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
                            found.insert((player, piece));
                        }
                    }
                }

                if found.len() > 1 {
                    return Err(format!(
                        "more than one piece at {:} -- {:?}",
                        file_rank_to_str(file, rank),
                        found,
                    ));
                }

                if found.len() == 0 {
                    if self.is_occupied(index) {
                        return Err(format!(
                            "no piece at {:} but occupied -- {:?}",
                            file_rank_to_str(file, rank),
                            found,
                        ));
                    }
                    continue;
                }

                let (player, _) = found.iter().next().unwrap();
                if self.is_occupied_by_player(index, other_player(*player)) {
                    return Err(format!(
                        "piece at {:} but occupied by other player -- {:?}",
                        file_rank_to_str(file, rank),
                        found,
                    ));
                } else if !self.is_occupied_by_player(index, *player) {
                    return Err(format!(
                        "piece at {:} but not occupied by player -- {:?}",
                        file_rank_to_str(file, rank),
                        found,
                    ));
                }
            }
        }

        Ok(())
    }

    pub fn clear_square(&mut self, index: isize, player: Player, piece: Piece) {
        let bb = single_bitboard(index);

        self.pieces[player][piece] &= !bb;
        self.occupied[player] &= !bb;
    }

    pub fn set_square(&mut self, index: isize, player: Player, piece: Piece) {
        let bb = single_bitboard(index);

        self.pieces[player][piece] |= bb;
        self.occupied[player] |= bb;
    }
}

#[test]
pub fn test_starting_board() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR".to_string();
    let mut bb = Bitboards::from_fen(fen).unwrap();
    assert_eq!(
        bb.pretty(),
        "♜♞♝♛♚♝♞♜\n\
						 ♟♟♟♟♟♟♟♟\n\
						 ········\n\
						 ········\n\
						 ········\n\
						 ········\n\
						 ♙♙♙♙♙♙♙♙\n\
						 ♖♘♗♕♔♗♘♖\n"
    );

    bb.verify().unwrap();

    let i = index_from_file_rank_str("e2").unwrap();
    bb.clear_square(i, Player::White, Piece::Pawn);

    assert_eq!(
        bb.pretty(),
        "♜♞♝♛♚♝♞♜\n\
						 ♟♟♟♟♟♟♟♟\n\
						 ········\n\
						 ········\n\
						 ········\n\
						 ········\n\
						 ♙♙♙♙·♙♙♙\n\
						 ♖♘♗♕♔♗♘♖\n"
    );

    bb.verify().unwrap();
}
