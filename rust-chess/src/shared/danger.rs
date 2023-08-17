use crate::{
    helpers::{OptionResult, StableOption},
    lazy::Lazy,
};
use lazy_static::lazy_static;

use super::{
    bitboard::{
        each_index_of_one, moves_bb_for_piece_and_blockers, single_bitboard, Bitboard, Bitboards,
        BoardIndex, WalkType,
    },
    helpers::ErrorResult,
    moves::index_in_danger,
    types::{Piece, Player},
};

#[derive(Debug, Clone, Copy, Default)]
pub struct Danger {
    pub check: bool,
    pub pinned: Bitboard,
}

fn compute_pins(
    target: BoardIndex,
    occupied: Bitboard,
    player_occupied: Bitboard,
    enemy_occupied: Bitboard,
    walk_type: WalkType,
) -> Bitboard {
    let dangers = moves_bb_for_piece_and_blockers(target, walk_type, occupied);

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

    pinned
}

impl Danger {
    pub fn from(player: Player, bitboards: &Bitboards) -> ErrorResult<Danger> {
        let enemy = player.other();

        let target = bitboards.index_of_piece(player, Piece::King);

        let pinned_by_bishop = compute_pins(
            target,
            bitboards.all_occupied(),
            bitboards.occupied[player],
            bitboards.pieces[enemy].bishops | bitboards.pieces[enemy].queens,
            WalkType::Bishop,
        );

        let pinned_by_rook = compute_pins(
            target,
            bitboards.all_occupied(),
            bitboards.occupied[player],
            bitboards.pieces[enemy].rooks | bitboards.pieces[enemy].queens,
            WalkType::Rook,
        );

        let check = index_in_danger(player, target, bitboards)?;

        Ok(Danger {
            check,
            pinned: pinned_by_bishop | pinned_by_rook,
        })
    }

    pub fn piece_is_pinned(&self, index: BoardIndex) -> bool {
        self.pinned & single_bitboard(index) != 0
    }
}

#[derive(Debug)]
pub struct LazyDanger {
    value: Option<Danger>,
}

lazy_static! {
    static ref DEFAULT_BB: Bitboards = Bitboards::new();
}

impl Default for LazyDanger {
    fn default() -> Self {
        LazyDanger { value: None }
    }
}

impl LazyDanger {
    pub fn reset(&mut self) {
        *self = LazyDanger { value: None }
    }

    pub fn get(&mut self, player: Player, bitboards: &Bitboards) -> ErrorResult<&Danger> {
        if self.value.is_none() {
            self.value = Some(Danger::from(player, bitboards)?);
        }
        self.value.as_ref().as_result()
    }
}

#[test]
fn test_pins() {
    let fen = "2k5/3r4/6b1/1N1N4/4N3/3K4/8/8";
    let bb = Bitboards::from_fen(fen).unwrap();
    let d = Danger::from(Player::White, &bb).unwrap();

    assert_eq!(false, d.check);

    assert_eq!(true, d.piece_is_pinned(BoardIndex::from_str("d5").unwrap()));
    assert_eq!(true, d.piece_is_pinned(BoardIndex::from_str("e4").unwrap()));
    assert_eq!(
        false,
        d.piece_is_pinned(BoardIndex::from_str("b5").unwrap())
    );
}

#[test]
fn test_double_pin() {
    let fen = "2k5/3r4/q5b1/1N1N1B2/2N1N3/3K1rPr/2Pnn3/3q1b2";
    let bb = Bitboards::from_fen(fen).unwrap();
    let d = Danger::from(Player::White, &bb).unwrap();

    assert_eq!(true, d.check);

    assert_eq!(true, d.piece_is_pinned(BoardIndex::from_str("d5").unwrap()));
    assert_eq!(
        false,
        d.piece_is_pinned(BoardIndex::from_str("b5").unwrap())
    );
    assert_eq!(
        false,
        d.piece_is_pinned(BoardIndex::from_str("c4").unwrap())
    );
    assert_eq!(
        false,
        d.piece_is_pinned(BoardIndex::from_str("c2").unwrap())
    );
    assert_eq!(
        false,
        d.piece_is_pinned(BoardIndex::from_str("e4").unwrap())
    );
    assert_eq!(
        false,
        d.piece_is_pinned(BoardIndex::from_str("f5").unwrap())
    );
}

#[test]
fn test_check() {
    let fen = "2k5/3r4/6b1/1N1N4/4N3/q2K4/8/8";
    let bb = Bitboards::from_fen(fen).unwrap();
    let d = Danger::from(Player::White, &bb).unwrap();

    assert_eq!(true, d.check);

    assert_eq!(true, d.piece_is_pinned(BoardIndex::from_str("d5").unwrap()));
    assert_eq!(true, d.piece_is_pinned(BoardIndex::from_str("e4").unwrap()));
    assert_eq!(
        false,
        d.piece_is_pinned(BoardIndex::from_str("b5").unwrap())
    );
}
