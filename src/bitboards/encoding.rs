use super::arithmetic::*;


pub fn bitboard_string(bb: Bitboard) -> String {
    let mut result: [String;8] = Default::default();

    for rank in 0..=7 {
		let bits_before = rank * 8;
		let bits_after = 64 - bits_before - 8;

        let mut sub = bb;

		// clip everything above this rank
		sub = shift_toward_index_63(sub, bits_after);
		// clip everything before this rank
		sub = shift_toward_index_0(sub, bits_before+bits_after);

        result[7 - rank as usize] = format!("{:08b}", reverse_bits(sub as u8)).replace("0", ".");
    }

    result.join("\n")
}

pub fn binary_string(bb: Bitboard) -> String {
    format!("{:064b}", bb)
}

pub fn bitboard_from_string(str: String) -> Bitboard {
    let mut bb: Bitboard = 0;
    for (inverse_rank, line) in str.split("\n").enumerate() {
        let rank = 7 - inverse_rank;

        for (file, c) in line.chars().enumerate() {
            if c == '1' {
                bb |= single_bitboard(index_from_rank_file(rank as i32, file as i32))
            }
        }
    }
    bb
}

#[test]
fn test_bitboard_string_zero_roundtrip() {
    let start =
            "........\n\
            ........\n\
            ........\n\
            ........\n\
            ........\n\
            ........\n\
            ........\n\
            ........";

    let bb = bitboard_from_string(start.to_string());
    let end = bitboard_string(bb);

    assert_eq!(start, end);
    assert_eq!(bb, 0);
}
#[test]
fn test_bitboard_string_simple_roundtrip() {
    let start =
            ".1......\n\
            ........\n\
            ........\n\
            ........\n\
            ........\n\
            ........\n\
            ........\n\
            ........";

    let bb = bitboard_from_string(start.to_string());
    let end = bitboard_string(bb);

    assert_eq!(start, end);
    assert_ne!(bb, 0);
}
#[test]
fn test_bitboard_string_many_roundtrip() {
    let start =
            "........\n\
            .1......\n\
            ....1...\n\
            ........\n\
            ........\n\
            ........\n\
            ....111.\n\
            .......1";

    let bb = bitboard_from_string(start.to_string());
    let end = bitboard_string(bb);

    assert_eq!(start, end);
    assert_ne!(bb, 0);
}