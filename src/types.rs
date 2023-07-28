use enum_map::Enum;
use strum::EnumIter;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, EnumIter, Enum)]
pub enum Player {
    White,
    Black,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct PlayerPiece {
    pub player: Player,
    pub piece: Piece,
}

impl PlayerPiece {
    pub fn new(player: Player, piece: Piece) -> Self {
        Self { player, piece }
    }

    pub fn from(c: char) -> Option<PlayerPiece> {
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

        Some(PlayerPiece::new(player, piece))
    }

    pub fn to_fen_char(self) -> char {
        let PlayerPiece { player, piece } = self;
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
}

impl std::fmt::Display for PlayerPiece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_fen_char())
    }
}

impl std::fmt::Debug for PlayerPiece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_fen_char())
    }
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Enum)]
pub enum CastlingSide {
    Kingside,
    Queenside,
}

pub static CASTLING_SIDES: [CastlingSide; 2] = [CastlingSide::Kingside, CastlingSide::Queenside];

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, EnumIter, Enum)]
pub enum Piece {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

impl Piece {
    pub fn to_uci(&self) -> &'static str {
        match self {
            Piece::Pawn => "p",
            Piece::Rook => "r",
            Piece::Knight => "n",
            Piece::Bishop => "b",
            Piece::Queen => "q",
            Piece::King => "k",
        }
    }
}

pub const PROMOTION_PIECES: [Piece; 4] = [Piece::Rook, Piece::Knight, Piece::Bishop, Piece::Queen];
