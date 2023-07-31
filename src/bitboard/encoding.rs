use super::*;

pub fn bitboard_string(bb: Bitboard) -> String {
    let mut result: [String; 8] = Default::default();

    for rank in 0..=7 {
        let bits_before = rank * 8;
        let bits_after = 64 - bits_before - 8;

        let mut sub = bb;

        // clip everything above this rank
        sub = rotate_toward_index_63(sub, bits_after);
        // clip everything before this rank
        sub = rotate_toward_index_0(sub, bits_before + bits_after);

        result[7 - rank as usize] = format!("{:08b}", reverse_bits(sub as u8)).replace("0", ".");
    }

    result.join("\n")
}

pub fn pretty_bitboard(bb: Bitboard) -> String {
    bitboard_string(bb).replace(".", ". ").replace("1", "1 ")
}

pub fn binary_string(bb: Bitboard) -> String {
    format!("{:064b}", bb)
}

pub fn bitboard_from_string(str: &str) -> Bitboard {
    let mut bb: Bitboard = 0;
    for (inverse_rank, line) in str.split("\n").enumerate() {
        let rank = 7 - inverse_rank;

        for (file, c) in line.chars().enumerate() {
            if c == '1' {
                bb |= single_bitboard(BoardIndex::from_file_rank(file as usize, rank as usize))
            }
        }
    }
    bb
}

pub fn bitboard_from_bytes(bytes: [u8; 8]) -> Bitboard {
    let mut bb: Bitboard = 0;
    for byte in bytes {
        bb <<= 8;
        bb |= byte.reverse_bits() as Bitboard;
    }
    bb
}

#[test]
fn test_bitboard_from_bytes() {
    let expected = ".1......\n\
            ........\n\
            ........\n\
            ........\n\
            ........\n\
            ........\n\
            ........\n\
            ........";

    let bb = bitboard_from_bytes([
        0b01000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
        0b00000000,
    ]);

    assert_eq!(bitboard_string(bb), expected);
}

#[test]
fn test_bitboard_string_zero_roundtrip() {
    let start = "........\n\
            ........\n\
            ........\n\
            ........\n\
            ........\n\
            ........\n\
            ........\n\
            ........";

    let bb = bitboard_from_string(start);
    let end = bitboard_string(bb);

    assert_eq!(start, end);
    assert_eq!(bb, 0);
}
#[test]
fn test_bitboard_string_simple_roundtrip() {
    let start = ".1......\n\
            ........\n\
            ........\n\
            ........\n\
            ........\n\
            ........\n\
            ........\n\
            ........";

    let bb = bitboard_from_string(start);
    let end = bitboard_string(bb);

    assert_eq!(start, end);
    assert_ne!(bb, 0);
}
#[test]
fn test_bitboard_string_many_roundtrip() {
    let start = "........\n\
            .1......\n\
            ....1...\n\
            ........\n\
            ........\n\
            ........\n\
            ....111.\n\
            .......1";

    let bb = bitboard_from_string(start);
    let end = bitboard_string(bb);

    assert_eq!(start, end);
    assert_ne!(bb, 0);
}
