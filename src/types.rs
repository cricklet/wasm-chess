pub use strum::EnumIter;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, strum::EnumIter)]
pub enum Player {
    White,
    Black,
}

pub fn other_player(player: Player) -> Player {
    match player {
        Player::White => Player::Black,
        Player::Black => Player::White,
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, strum::EnumIter)]
pub enum CastlingSide {
    KingSide,
    QueenSide,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, EnumIter)]
pub enum Piece {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

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
