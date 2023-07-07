use memoize::memoize;

use crate::helpers::*;

pub type Bitboard = u64;

pub fn least_significant_one(bb: Bitboard) -> Bitboard {
    bb & bb.wrapping_neg()
}

pub fn first_index_of_one(bb: Bitboard) -> u32 {
    let ls1 = least_significant_one(bb);
    (ls1 - 1).count_ones()
}

#[test]
fn test_least_significant_one() {
    let binary = 0b0000000010000000000000000000000000000000000000000000000000010000;

    assert_eq!(first_index_of_one(binary), 4);
    assert_eq!(least_significant_one(binary), single_bitboard(4));
}

pub fn each_index_of_one_callback<F: FnMut(i32) -> Loop>(bb: Bitboard, mut callback: F) {
    let mut temp = bb;
    while temp != 0 {
        let ls1 = least_significant_one(temp);
        let index = (ls1 - 1).count_ones();

        let result = callback(index as i32);
        match result {
            Loop::Continue => {}
            Loop::Break => break,
        }

        temp = temp ^ ls1
    }
}

pub fn each_index_of_one_closure(bb: Bitboard) -> impl FnMut() -> Option<i32> {
    let mut temp = bb;
    move || {
        if temp != 0 {
            let ls1 = least_significant_one(temp);
            let index = (ls1 - 1).count_ones();

            temp = temp ^ ls1;

            return Some(index as i32);
        } else {
            return None;
        }
    }
}

pub fn each_index_of_one(bb: Bitboard) -> IteratorFn<i32, impl FnMut() -> Option<i32>> {
    let mut temp = bb;

    IteratorFn::new(move || {
        if temp != 0 {
            let ls1 = least_significant_one(temp);
            let index = (ls1 - 1).count_ones();

            temp = temp ^ ls1;

            return Some(index as i32);
        } else {
            return None;
        }
    })
}

#[test]
fn test_each_index_of_one_callback() {
    let binary = 0b0000000000000000000000000001000000000000000000000000000010000010;

    let mut expected = vec![36, 7, 1];

    each_index_of_one_callback(binary, |index| {
        assert_eq!(expected.pop().unwrap(), index);
        Loop::Continue
    });
}

#[test]
fn test_each_index_of_one_closure() {
    let binary = 0b0000000000000000000000000001000000000000000000000000000010000010;

    let mut expected = vec![36, 7, 1];

    let mut generator = each_index_of_one_closure(binary);
    loop {
        match generator() {
            Some(index) => {
                assert_eq!(expected.pop().unwrap(), index);
            }
            None => break,
        }
    }
}

#[test]
fn test_each_index_of_one() {
    let binary = 0b0000000000000000000000000001000000000000000000000000000010000010;

    let mut expected = vec![36, 7, 1];

    for index in each_index_of_one(binary) {
        assert_eq!(expected.pop().unwrap(), index);
    }
}

pub fn shift_toward_index_0(bb: Bitboard, n: i32) -> Bitboard {
    bb >> n
}

pub fn shift_toward_index_63(bb: Bitboard, n: i32) -> Bitboard {
    bb << n
}

#[memoize]
fn reverse_bits_cache() -> [u8; 256] {
    let mut result: [u8; 256] = [0; 256];
    for i in 0..=255 {
        let mut reversed: u8 = 0;
        for bit in 0..8 {
            reversed |= ((i >> bit) & 1) << (7 - bit);
        }
        result[i as usize] = reversed;
    }
    result
}

pub fn reverse_bits(v: u8) -> u8 {
    return reverse_bits_cache()[v as usize];
}

pub fn single_bitboard(index: i32) -> Bitboard {
    shift_toward_index_63(1, index)
}

#[test]
fn test_single_bitboard() {
    use super::encoding::*;

    let bb = single_bitboard(1);
    let board_expected = "........\n\
            ........\n\
            ........\n\
            ........\n\
            ........\n\
            ........\n\
            ........\n\
            .1......";

    let binary_expected = "0000000000000000000000000000000000000000000000000000000000000010";

    assert_eq!(bitboard_string(bb), board_expected);
    assert_eq!(binary_string(bb), binary_expected);
}

pub fn index_from_file_rank(file: i32, rank: i32) -> i32 {
    rank * 8 + file
}

pub fn file_rank_from_index(index: i32) -> (i32, i32) {
    (index % 8, index / 8)
}

pub fn index_from_file_rank_str(file_rank_str: &str) -> Option<i32> {
    let mut chars = file_rank_str.chars();

    if file_rank_str.len() != 2 {
        return None;
    }

    let file_char = match chars.next() {
        Some(c) => c,
        None => return None,
    };

    let rank_char = match chars.next() {
        Some(c) => c,
        None => return None,
    };

    let file = match file_char {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        'h' => 7,
        _ => return None,
    };

    let rank = match rank_char {
        '1' => 0,
        '2' => 1,
        '3' => 2,
        '4' => 3,
        '5' => 4,
        '6' => 5,
        '7' => 6,
        '8' => 7,
        _ => return None,
    };

    return Some(index_from_file_rank(file, rank));
}

pub fn file_rank_to_str(file: i32, rank: i32) -> String {
    let file_char = match file {
        0 => 'a',
        1 => 'b',
        2 => 'c',
        3 => 'd',
        4 => 'e',
        5 => 'f',
        6 => 'g',
        7 => 'h',
        _ => return "??".to_string(),
    };

    let rank_char = match rank {
        0 => '1',
        1 => '2',
        2 => '3',
        3 => '4',
        4 => '5',
        5 => '6',
        6 => '7',
        7 => '8',
        _ => return "??".to_string(),
    };

    format!("{}{}", file_char, rank_char)
}

pub fn index_to_file_rank_str(i: i32) -> String {
    let (file, rank) = file_rank_from_index(i);
    file_rank_to_str(file, rank)
}
