use lazy_static::*;

pub type Bitboard = u64;

pub fn least_significant_one(bb: Bitboard) -> Bitboard {
    bb & bb.wrapping_neg()
}

pub fn first_index_of_one(bb: Bitboard) -> usize {
    let ls1 = least_significant_one(bb);
    (ls1 - 1).count_ones() as usize
}

#[test]
fn test_least_significant_one() {
    let binary = 0b0000000010000000000000000000000000000000000000000000000000010000;

    assert_eq!(first_index_of_one(binary), 4);
    assert_eq!(least_significant_one(binary), single_bitboard(4));
}

pub fn shift_toward_index_0(bb: Bitboard, n: isize) -> Bitboard {
    bb >> n
}

pub fn shift_toward_index_63(bb: Bitboard, n: isize) -> Bitboard {
    bb << n
}

pub fn rotate_toward_index_0(bb: Bitboard, n: isize) -> Bitboard {
    bb.rotate_right(n as u32)
}

pub fn rotate_toward_index_63(bb: Bitboard, n: isize) -> Bitboard {
    bb.rotate_left(n as u32)
}

lazy_static! {
    static ref REVERSE_BITS_CACHE: [u8; 256] = {
        let mut result: [u8; 256] = [0; 256];
        for i in 0..=255 {
            let mut reversed: u8 = 0;
            for bit in 0..8 {
                reversed |= ((i >> bit) & 1) << (7 - bit);
            }
            result[i as usize] = reversed;
        }
        result
    };
}
pub fn reverse_bits(v: u8) -> u8 {
    return REVERSE_BITS_CACHE[v as usize];
}

pub fn single_bitboard(index: usize) -> Bitboard {
    shift_toward_index_63(1, index as isize)
}

#[test]
fn test_single_bitboard() {
    use super::encoding::*;

    let bb = single_bitboard(1);
    let board_expected = "\
            ........\n\
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

pub fn index_from_file_rank(file: usize, rank: usize) -> usize {
    rank * 8 + file
}

pub fn file_rank_from_index(index: usize) -> (usize, usize) {
    (index % 8, index / 8)
}

pub fn file_from_char(file: char) -> Option<usize> {
    match file {
        'a' => Some(0),
        'b' => Some(1),
        'c' => Some(2),
        'd' => Some(3),
        'e' => Some(4),
        'f' => Some(5),
        'g' => Some(6),
        'h' => Some(7),
        _ => None,
    }
}

pub fn rank_from_char(rank: char) -> Option<usize> {
    match rank {
        '1' => Some(0),
        '2' => Some(1),
        '3' => Some(2),
        '4' => Some(3),
        '5' => Some(4),
        '6' => Some(5),
        '7' => Some(6),
        '8' => Some(7),
        _ => None,
    }
}

pub fn is_rank(c: char) -> bool {
    match rank_from_char(c) {
        Some(_) => true,
        None => false,
    }
}

pub fn is_file(c: char) -> bool {
    match file_from_char(c) {
        Some(_) => true,
        None => false,
    }
}

pub fn index_from_file_rank_str(file_rank_str: &str) -> Option<usize> {
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

    let file = match file_from_char(file_char) {
        Some(f) => f,
        None => return None,
    };

    let rank = match rank_from_char(rank_char) {
        Some(r) => r,
        None => return None,
    };

    Some(index_from_file_rank(file, rank))
}

pub fn file_rank_to_str(file: usize, rank: usize) -> String {
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

pub fn index_to_file_rank_str(i: usize) -> String {
    let (file, rank) = file_rank_from_index(i);
    file_rank_to_str(file, rank)
}

pub fn bitboard_with_indices_set(indices: &[usize]) -> Bitboard {
    let mut bb: Bitboard = 0;
    for index in indices {
        bb |= single_bitboard(*index);
    }
    bb
}

pub fn bitboard_with_file_rank_strs_set(locations: &[&str]) -> Bitboard {
    let indices = locations
        .iter()
        .map(|s| index_from_file_rank_str(s).unwrap())
        .collect::<Vec<usize>>();
    bitboard_with_indices_set(&indices)
}
