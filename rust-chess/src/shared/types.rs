use enum_map::Enum;
use strum::EnumIter;

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Hash, EnumIter, Enum)]
pub enum Player {
    #[default] White,
    Black,
}

impl Player {
    pub fn to_usize(self) -> usize {
        self as usize
    }
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

    pub fn to_usize(self) -> usize {
        self.player as usize * 6 + self.piece as usize
    }

    pub fn from(c: char) -> Option<PlayerPiece> {
        let piece = Piece::from(c)?;

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

impl CastlingSide {
    pub fn to_usize(self) -> usize {
        self as usize
    }
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
    pub fn from(c: char) -> Option<Piece> {
        match c.to_ascii_lowercase() {
            'p' => Some(Piece::Pawn),
            'r' => Some(Piece::Rook),
            'n' => Some(Piece::Knight),
            'b' => Some(Piece::Bishop),
            'q' => Some(Piece::Queen),
            'k' => Some(Piece::King),
            _ => None,
        }
    }
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

    pub fn centipawns(&self) -> isize {
        match self {
            Piece::Pawn => 100,
            Piece::Rook => 500,
            Piece::Knight => 300,
            Piece::Bishop => 300,
            Piece::Queen => 900,
            Piece::King => 2000,
        }
    }
}

pub const PROMOTION_PIECES: [Piece; 4] = [Piece::Rook, Piece::Knight, Piece::Bishop, Piece::Queen];
