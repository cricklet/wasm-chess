use lazy_static::lazy_static;
use memoize::memoize;
use strum::EnumIter;

use crate::{
    helpers::{err_result, ErrorResult},
    types::Player,
};

use super::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, EnumIter)]
pub enum Direction {
    N,
    S,
    E,
    W,
    NE,
    NW,
    SE,
    SW,
    NNE,
    NNW,
    SSE,
    SSW,
    EEN,
    EES,
    WWN,
    WWS,
}

impl Direction {
    pub fn offset(self) -> isize {
        match self {
            Direction::N => OFFSET_N,
            Direction::S => OFFSET_S,
            Direction::E => OFFSET_E,
            Direction::W => OFFSET_W,
            Direction::NE => OFFSET_NE,
            Direction::NW => OFFSET_NW,
            Direction::SE => OFFSET_SE,
            Direction::SW => OFFSET_SW,
            Direction::NNE => OFFSET_NNE,
            Direction::NNW => OFFSET_NNW,
            Direction::SSE => OFFSET_SSE,
            Direction::SSW => OFFSET_SSW,
            Direction::EEN => OFFSET_EEN,
            Direction::EES => OFFSET_EES,
            Direction::WWN => OFFSET_WWN,
            Direction::WWS => OFFSET_WWS,
        }
    }
}

const OFFSET_N: isize = 8;
const OFFSET_S: isize = -8;
const OFFSET_E: isize = 1;
const OFFSET_W: isize = -1;

const OFFSET_NE: isize = OFFSET_N + OFFSET_E;
const OFFSET_NW: isize = OFFSET_N + OFFSET_W;
const OFFSET_SE: isize = OFFSET_S + OFFSET_E;
const OFFSET_SW: isize = OFFSET_S + OFFSET_W;

const OFFSET_NNE: isize = OFFSET_N + OFFSET_N + OFFSET_E;
const OFFSET_NNW: isize = OFFSET_N + OFFSET_N + OFFSET_W;
const OFFSET_SSE: isize = OFFSET_S + OFFSET_S + OFFSET_E;
const OFFSET_SSW: isize = OFFSET_S + OFFSET_S + OFFSET_W;
const OFFSET_EEN: isize = OFFSET_E + OFFSET_E + OFFSET_N;
const OFFSET_EES: isize = OFFSET_E + OFFSET_E + OFFSET_S;
const OFFSET_WWN: isize = OFFSET_W + OFFSET_W + OFFSET_N;
const OFFSET_WWS: isize = OFFSET_W + OFFSET_W + OFFSET_S;

pub const KNIGHT_DIRS: [Direction; 8] = [
    Direction::NNE,
    Direction::NNW,
    Direction::SSE,
    Direction::SSW,
    Direction::EEN,
    Direction::EES,
    Direction::WWN,
    Direction::WWS,
];

pub const KING_DIRS: [Direction; 8] = [
    Direction::N,
    Direction::S,
    Direction::E,
    Direction::W,
    Direction::NE,
    Direction::NW,
    Direction::SE,
    Direction::SW,
];

pub const ROOK_DIRS: [Direction; 4] = [Direction::N, Direction::S, Direction::E, Direction::W];

pub const BISHOP_DIRS: [Direction; 4] =
    [Direction::NE, Direction::NW, Direction::SE, Direction::SW];

pub fn pawn_push_direction_for_player(player: Player) -> Direction {
    match player {
        Player::White => Direction::N,
        Player::Black => Direction::S,
    }
}

pub fn pawn_capture_directions_for_player(player: Player) -> &'static [Direction; 2] {
    match player {
        Player::White => &[Direction::NE, Direction::NW],
        Player::Black => &[Direction::SE, Direction::SW],
    }
}

pub fn en_passant_move_and_target_offsets(player: Player) -> &'static [(Direction, Direction); 2] {
    match player {
        Player::White => &[(Direction::NE, Direction::E), (Direction::NW, Direction::W)],
        Player::Black => &[(Direction::SE, Direction::S), (Direction::SW, Direction::W)],
    }
}

lazy_static! {
    pub static ref PAWN_PROMOTION_BITBOARD: Bitboard = bitboard_from_string(
        "11111111\n\
         ........\n\
         ........\n\
         ........\n\
         ........\n\
         ........\n\
         ........\n\
         11111111",
    );
}

pub const ALL_ZEROS: Bitboard = 0;
pub const ALL_ONES: Bitboard = 0xffffffffffffffff;

#[memoize]
pub fn zeros_for_file(file: char) -> Bitboard {
    let mut bb = ALL_ONES;
    let f = file_from_char(file).unwrap();

    for r in 0..8 {
        let index = index_from_file_rank(f, r);
        bb &= !single_bitboard(index);
    }

    bb
}

#[memoize]
pub fn zeros_for_rank(rank: char) -> Bitboard {
    let mut bb = ALL_ONES;
    let r = rank_from_char(rank).unwrap();

    for f in 0..8 {
        let index = index_from_file_rank(f, r);
        bb &= !single_bitboard(index);
    }

    bb
}

pub fn zeros_for(cs: &[char]) -> ErrorResult<Bitboard> {
    let mut bb = ALL_ONES;
    for c in cs {
        if is_file(*c) {
            bb &= zeros_for_file(*c);
        } else if is_rank(*c) {
            bb &= zeros_for_rank(*c);
        } else {
            return err_result(&format!("Invalid char: {}", c));
        }
    }
    Ok(bb)
}

pub fn pre_move_mask(direction: Direction) -> Bitboard {
    match direction {
        Direction::N => zeros_for(&['8']).unwrap(),
        Direction::S => zeros_for(&['1']).unwrap(),
        Direction::E => zeros_for(&['h']).unwrap(),
        Direction::W => zeros_for(&['a']).unwrap(),

        Direction::NE => zeros_for(&['8', 'h']).unwrap(),
        Direction::NW => zeros_for(&['8', 'a']).unwrap(),
        Direction::SE => zeros_for(&['1', 'h']).unwrap(),
        Direction::SW => zeros_for(&['1', 'a']).unwrap(),

        Direction::NNE => zeros_for(&['8', '7', 'h']).unwrap(),
        Direction::NNW => zeros_for(&['8', '7', 'a']).unwrap(),
        Direction::SSE => zeros_for(&['1', '2', 'h']).unwrap(),
        Direction::SSW => zeros_for(&['1', '2', 'a']).unwrap(),
        Direction::EEN => zeros_for(&['h', 'g', '8']).unwrap(),
        Direction::EES => zeros_for(&['h', 'g', '1']).unwrap(),
        Direction::WWN => zeros_for(&['a', 'b', '8']).unwrap(),
        Direction::WWS => zeros_for(&['a', 'b', '1']).unwrap(),
    }
}

#[test]
fn test_pre_move_mask_unwrap() {
    for dir in <Direction as strum::IntoEnumIterator>::iter() {
        pre_move_mask(dir);
    }
}

lazy_static! {
    static ref STARTING_PAWN_MASK_WHITE: Bitboard = !zeros_for(&['2']).unwrap();
    static ref STARTING_PAWN_MASK_BLACK: Bitboard = !zeros_for(&['7']).unwrap();
}

pub fn starting_pawns_mask(player: Player) -> &'static Bitboard {
    match player {
        Player::White => &STARTING_PAWN_MASK_WHITE,
        Player::Black => &STARTING_PAWN_MASK_BLACK,
    }
}

#[test]
fn test_bitwise_not() {
    let bb = single_bitboard(BoardIndex::from(0));
    assert_eq!(
        bitboard_string(bb),
        "\
        ........\n\
        ........\n\
        ........\n\
        ........\n\
        ........\n\
        ........\n\
        ........\n\
        1......."
            .to_string()
    );
    assert_eq!(
        bitboard_string(!bb),
        "\
        11111111\n\
        11111111\n\
        11111111\n\
        11111111\n\
        11111111\n\
        11111111\n\
        11111111\n\
        .1111111"
            .to_string()
    );
}

#[test]
fn test_zero_bitboards() {
    assert_eq!(
        bitboard_string(zeros_for_file('a')),
        "\
        .1111111\n\
        .1111111\n\
        .1111111\n\
        .1111111\n\
        .1111111\n\
        .1111111\n\
        .1111111\n\
        .1111111"
            .to_string()
    );
    assert_eq!(
        bitboard_string(zeros_for_rank('4')),
        "\
        11111111\n\
        11111111\n\
        11111111\n\
        11111111\n\
        ........\n\
        11111111\n\
        11111111\n\
        11111111"
            .to_string()
    );
}

#[test]
fn test_pre_move_mask() {
    assert_eq!(
        bitboard_string(pre_move_mask(Direction::SSE)),
        "\
        1111111.\n\
        1111111.\n\
        1111111.\n\
        1111111.\n\
        1111111.\n\
        1111111.\n\
        ........\n\
        ........"
            .to_string()
    );
}

#[test]
fn test_starting_pawns_mask() {
    assert_eq!(
        bitboard_string(*starting_pawns_mask(Player::White)),
        "\
        ........\n\
        ........\n\
        ........\n\
        ........\n\
        ........\n\
        ........\n\
        11111111\n\
        ........"
            .to_string()
    );
    assert_eq!(
        bitboard_string(*starting_pawns_mask(Player::Black)),
        "\
        ........\n\
        11111111\n\
        ........\n\
        ........\n\
        ........\n\
        ........\n\
        ........\n\
        ........"
            .to_string()
    );
}

lazy_static! {
    pub static ref KNIGHT_MOVE_BITBOARD: [Bitboard; 64] = {
        let mut result = [ALL_ZEROS; 64];
        for index in 0..64 {
            let mut mask = ALL_ZEROS;

            let bb = single_bitboard(BoardIndex::from(index));

            for dir in KNIGHT_DIRS.iter() {
                let filtered_bb = bb & pre_move_mask(*dir);
                let offset_bb = rotate_toward_index_63(filtered_bb, dir.offset());
                mask |= offset_bb;
            }
            result[index] = mask;
        }
        result
    };
    pub static ref KING_MOVE_BITBOARD: [Bitboard; 64] = {
        let mut result = [ALL_ZEROS; 64];
        for index in 0..64 {
            let mut mask = ALL_ZEROS;

            let bb = single_bitboard(BoardIndex::from(index));

            for dir in KING_DIRS.iter() {
                let filtered_bb = bb & pre_move_mask(*dir);
                let offset_bb = rotate_toward_index_63(filtered_bb, dir.offset());
                mask |= offset_bb;
            }
            result[index] = mask;
        }
        result
    };
}

#[test]
pub fn test_knight_move_bitboard() {
    assert_eq!(
        bitboard_string(single_bitboard(BoardIndex::from(9))),
        "\
        ........\n\
        ........\n\
        ........\n\
        ........\n\
        ........\n\
        ........\n\
        .1......\n\
        ........"
            .to_string()
    );
    assert_eq!(
        bitboard_string(KNIGHT_MOVE_BITBOARD[9]),
        "\
        ........\n\
        ........\n\
        ........\n\
        ........\n\
        1.1.....\n\
        ...1....\n\
        ........\n\
        ...1...."
            .to_string()
    );
}

#[test]
pub fn test_king_move_bitboard() {
    assert_eq!(
        bitboard_string(single_bitboard(BoardIndex::from(9))),
        "\
        ........\n\
        ........\n\
        ........\n\
        ........\n\
        ........\n\
        ........\n\
        .1......\n\
        ........"
            .to_string()
    );
    assert_eq!(
        bitboard_string(KING_MOVE_BITBOARD[9]),
        "\
        ........\n\
        ........\n\
        ........\n\
        ........\n\
        ........\n\
        111.....\n\
        1.1.....\n\
        111....."
            .to_string()
    );
    assert_eq!(
        bitboard_string(single_bitboard(BoardIndex::from(47))),
        "\
        ........\n\
        ........\n\
        .......1\n\
        ........\n\
        ........\n\
        ........\n\
        ........\n\
        ........"
            .to_string()
    );
    assert_eq!(
        bitboard_string(KING_MOVE_BITBOARD[47]),
        "\
        ........\n\
        ......11\n\
        ......1.\n\
        ......11\n\
        ........\n\
        ........\n\
        ........\n\
        ........"
            .to_string()
    );
}
