use super::arithmetic::*;


#[derive(Debug)]
pub struct BitboardsForPlayer {
	pub pawns: Bitboard,
	pub rooks: Bitboard,
	pub knights: Bitboard,
	pub bishops: Bitboard,
	pub queens: Bitboard,
	pub king: Bitboard,
	pub occupied: Bitboard,
}

impl BitboardsForPlayer {
	pub fn new() -> BitboardsForPlayer {
		BitboardsForPlayer {
			pawns: 0,
			rooks: 0,
			knights: 0,
			bishops: 0,
			queens: 0,
			king: 0,
			occupied: 0,
		}
	}
}

#[derive(Debug)]
pub struct Bitboards {
	pub white: BitboardsForPlayer,
	pub black: BitboardsForPlayer,
}

impl Bitboards {
	pub fn new() -> Bitboards {
		Bitboards {
			white: BitboardsForPlayer::new(),
			black: BitboardsForPlayer::new(),
		}
	}
}

