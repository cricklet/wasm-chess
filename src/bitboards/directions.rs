use memoize::memoize;

use crate::{helpers::ErrorResult, types::Player};

use super::{arithmetic::*, *};

const N: isize = 8;
const S: isize = -8;
const E: isize = 1;
const W: isize = -1;

const NE: isize = N + E;
const NW: isize = N + W;
const SE: isize = S + E;
const SW: isize = S + W;

const NNE: isize = N + N + E;
const NNW: isize = N + N + W;
const SSE: isize = S + S + E;
const SSW: isize = S + S + W;
const EEN: isize = E + E + N;
const EES: isize = E + E + S;
const WWN: isize = W + W + N;
const WWS: isize = W + W + S;

const KNIGHT_DIRS: [isize; 8] = [NNE, NNW, SSE, SSW, EEN, EES, WWN, WWS];

const KING_DIRS: [isize; 8] = [N, S, E, W, NE, NW, SE, SW];

const ROOK_DIRS: [isize; 4] = [N, S, E, W];

const BISHOP_DIRS: [isize; 4] = [NE, NW, SE, SW];

pub fn pawn_dir_for_player(player: Player) -> isize {
    match player {
        Player::White => N,
        Player::Black => S,
    }
}

pub fn pawn_capture_dir_for_player(player: Player) -> [isize; 2] {
    match player {
        Player::White => [NE, NW],
        Player::Black => [SE, SW],
    }
}

#[memoize]
pub fn pawn_promotion_bitboard() -> Bitboard {
    bitboard_from_string(
        "11111111\n\
         ........\n\
         ........\n\
         ........\n\
         ........\n\
         ........\n\
         ........\n\
         11111111"
            .to_string(),
    )
}

const ALL_ZEROS: Bitboard = 0;
const ALL_ONES: Bitboard = 0xffffffffffffffff;

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
            return Err(format!("Invalid char: {}", c));
        }
    }
    Ok(bb)
}

#[memoize]
pub fn pre_move_mask(offset: isize) -> ErrorResult<Bitboard> {
    match offset {
        N => zeros_for(&['8']),
        S => zeros_for(&['1']),
        E => zeros_for(&['h']),
        W => zeros_for(&['a']),

        NE => zeros_for(&['8', 'h']),
        NW => zeros_for(&['8', 'a']),
        SE => zeros_for(&['1', 'h']),
        SW => zeros_for(&['1', 'a']),

        NNE => zeros_for(&['8', '7', 'h']),
        NNW => zeros_for(&['8', '7', 'a']),
        SSE => zeros_for(&['1', '2', 'h']),
        SSW => zeros_for(&['1', '2', 'a']),
        EEN => zeros_for(&['h', 'g', '8']),
        EES => zeros_for(&['h', 'g', '1']),
        WWN => zeros_for(&['a', 'b', '8']),
        WWS => zeros_for(&['a', 'b', '1']),

        _ => Err(format!("Invalid offset: {}", offset)),
    }
}

#[test]
fn test_bitwise_not() {
    let bb = single_bitboard(0);
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
        bitboard_string(pre_move_mask(SSE).unwrap()),
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
