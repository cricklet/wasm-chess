
use memoize::memoize;

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

pub fn shift_toward_index_0(bb: Bitboard, n: i32) -> Bitboard {
    bb >> n
}

pub fn shift_toward_index_63(bb: Bitboard, n: i32) -> Bitboard {
    bb << n
}

#[memoize]
fn reverse_bits_cache() -> [u8;256] {
    let mut result: [u8;256] = [0;256];
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
    let board_expected =
            "........\n\
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

pub fn index_from_rank_file(rank: i32, file: i32) -> i32 {
    rank * 8 + file
}
