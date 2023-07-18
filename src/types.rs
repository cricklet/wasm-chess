use enum_map::Enum;
use strum::EnumIter;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, EnumIter, Enum)]
pub enum Player {
    White,
    Black,
}

impl Player {
    pub fn other(self) -> Self {
        other_player(self)
    }

    pub fn to_fen(self) -> &'static str {
        match self {
            Player::White => "w",
            Player::Black => "b",
        }
    }
}

pub fn other_player(player: Player) -> Player {
    match player {
        Player::White => Player::Black,
        Player::Black => Player::White,
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, EnumIter, Enum)]
pub enum CastlingSide {
    Kingside,
    Queenside,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, EnumIter, Enum)]
pub enum Piece {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

pub const PROMOTION_PIECES: [Piece; 4] = [Piece::Rook, Piece::Knight, Piece::Bishop, Piece::Queen];

pub type PlayerPiece = (Player, Piece);

pub fn player_and_piece_from_fen_char(c: char) -> Option<PlayerPiece> {
    let piece = match c.to_ascii_uppercase() {
        'P' => Piece::Pawn,
        'R' => Piece::Rook,
        'N' => Piece::Knight,
        'B' => Piece::Bishop,
        'Q' => Piece::Queen,
        'K' => Piece::King,
        _ => return None,
    };

    let player = match c.is_uppercase() {
        true => Player::White,
        false => Player::Black,
    };

    Some((player, piece))
}

pub fn player_and_piece_to_fen_char(player_piece: PlayerPiece) -> char {
    let (player, piece) = player_piece;
    let fen_char = match piece {
        Piece::Pawn => 'P',
        Piece::Rook => 'R',
        Piece::Knight => 'N',
        Piece::Bishop => 'B',
        Piece::Queen => 'Q',
        Piece::King => 'K',
    };

    match player {
        Player::White => fen_char,
        Player::Black => fen_char.to_ascii_lowercase(),
    }
}
