use super::*;
use memoize::memoize;
use rand::*;

#[derive(Debug, Clone, Copy)]
pub enum PieceForMagic {
    Rook,
    Bishop,
}

const OFFSETS_FOR_MAGIC: [&[isize]; 2] = [&ROOK_DIRS, &BISHOP_DIRS];

#[derive(Debug, Clone, Copy)]
pub struct MagicValue {
    magic: u64,
    bits_required: isize,
}

pub struct MagicMoveTable {
    // Each of the 64 indices on a board have a magic-lookup precomputed,
    // allowing us to look up a bitboard of possible moves given the
    // current occupancy of the board.
    //
    // eg
    // let blocker_bb = magic_table.blocker_masks[piece_index] & all_occupied_bb
    // let magic_values = magic_table.magics[piece_index]
    // let magic_index = compute_magic_index(magic_values.magic, blocker_bb, magic_values.bits_required)
    //
    // let potential_bb = magic_table.moves[magic_index][piece_index]
    // let move_bb = potential & ^self_occupied_bb
    //
    // let quiet_bb = potential & ^all_occupied_bb
    // let capture_bb = potential & ^quiet_bb
    magics: [MagicValue; 64],
    blocker_masks: [Bitboard; 64],
    moves: [[Bitboard; 64]],
}

pub fn compute_magic_index(magic: MagicValue, blocker_bb: Bitboard) -> usize {
    let magic_index = (blocker_bb.wrapping_mul(magic.magic)) >> (64 - magic.bits_required);
    magic_index as usize
}

#[derive(Debug, Clone, Copy)]
pub struct MoveAndBlocker {
    move_bb: Bitboard,
    blocker_bb: Bitboard,
}

pub fn magic_works(magic_value: MagicValue, expected_moves: &[MoveAndBlocker]) -> bool {
    let size = 1 << magic_value.bits_required;
    let mut magic_move_table: Vec<Option<Bitboard>> = vec![None; size];

    for expected in expected_moves {
        let magic_index = compute_magic_index(magic_value, expected.blocker_bb);
        match magic_move_table[magic_index] {
            None => {
                magic_move_table[magic_index] = Some(expected.move_bb);
            }
            Some(move_bb) => {
                if move_bb != expected.move_bb {
                    return false;
                }
            }
        }
    }

    true
}

#[memoize]
pub fn generate_walk_bb(piece_index: isize, blocker_bb: Bitboard, offset: isize) -> Bitboard {
    let mut walk_bb = Bitboard::default();

    let premove_mask = pre_move_mask(offset).unwrap();
    let mut last_location_bb = single_bitboard(piece_index);

    while last_location_bb != 0 {
        let next_location_bb = rotate_toward_index_63(last_location_bb & premove_mask, offset);

        let quiet_bb = next_location_bb & !blocker_bb;
        let capture_bb = next_location_bb & blocker_bb;

        walk_bb |= quiet_bb | capture_bb;

        last_location_bb = quiet_bb
    }

    walk_bb
}

#[test]
pub fn test_generate_walk_bb() {
    let start_index = 10;
    let start_bb = single_bitboard(start_index);

    assert_eq!(
        bitboard_string(start_bb),
        "\
        ........\n\
        ........\n\
        ........\n\
        ........\n\
        ........\n\
        ........\n\
        ..1.....\n\
        ........"
            .to_string()
    );
    {
        let blocker_bb = Bitboard::default();
        let offset = NE;
        let walk_bb = generate_walk_bb(start_index, blocker_bb, offset);

        assert_eq!(
            bitboard_string(walk_bb),
            "\
        ........\n\
        .......1\n\
        ......1.\n\
        .....1..\n\
        ....1...\n\
        ...1....\n\
        ........\n\
        ........"
                .to_string()
        );
    }
    {
        let blocker_bb = bitboard_from_string(
            "\
            ........\n\
            ........\n\
            ........\n\
            .....1..\n\
            ........\n\
            ........\n\
            ........\n\
            ........"
                .to_string(),
        );
        let offset = NE;
        let walk_bb = generate_walk_bb(start_index, blocker_bb, offset);

        assert_eq!(
            bitboard_string(walk_bb),
            "\
        ........\n\
        ........\n\
        ........\n\
        .....1..\n\
        ....1...\n\
        ...1....\n\
        ........\n\
        ........"
                .to_string()
        );
    }
}

pub fn generate_overall_blocker_bb(start_index: isize, piece_for_magic: PieceForMagic) -> Bitboard {
    let mut overall_blocker_bb = Bitboard::default();

    for &offset in OFFSETS_FOR_MAGIC[piece_for_magic as usize] {
        let walk_bb = generate_walk_bb(start_index, overall_blocker_bb, offset);
        let walk_bb_filtered = walk_bb & pre_move_mask(offset).unwrap();

        overall_blocker_bb |= walk_bb_filtered;
    }

    overall_blocker_bb
}

#[test]
pub fn test_generate_overall_blocker_bb() {
    let start_index = 10;
    let start_bb = single_bitboard(start_index);

    assert_eq!(
        bitboard_string(start_bb),
        "\
        ........\n\
        ........\n\
        ........\n\
        ........\n\
        ........\n\
        ........\n\
        ..1.....\n\
        ........"
            .to_string()
    );
    {
        let overall_blocker_bb = generate_overall_blocker_bb(start_index, PieceForMagic::Bishop);

        assert_eq!(
            bitboard_string(overall_blocker_bb),
            "\
        ........\n\
        ........\n\
        ......1.\n\
        .....1..\n\
        ....1...\n\
        .1.1....\n\
        ........\n\
        ........"
                .to_string()
        );
    }
}

pub fn generate_specific_blocker_bb(overall_blocker_bb: Bitboard, seed: usize) -> Bitboard {
    let mut specific_blocker_bb = Bitboard::default();

    let num_bits = overall_blocker_bb.count_ones() as usize;
    for i in 0..num_bits {
        // If the bit at i is 1 in the seed
        if seed & (1 << i) != 0 {
            // Find the ith one bit in blockerMask and set the corresponding bit to one in result.
            for (j, bit_index) in each_index_of_one(overall_blocker_bb).enumerate() {
                if i == j {
                    specific_blocker_bb |= single_bitboard(bit_index as isize);
                }
            }
        }
    }

    specific_blocker_bb
}

#[test]
pub fn test_generate_specific_blocker_bb() {
    let start_index = 15;
    let start_bb = single_bitboard(start_index);

    assert_eq!(
        bitboard_string(start_bb),
        "\
        ........\n\
        ........\n\
        ........\n\
        ........\n\
        ........\n\
        ........\n\
        .......1\n\
        ........"
            .to_string()
    );
    let overall_blocker_bb = generate_overall_blocker_bb(start_index, PieceForMagic::Rook);

    assert_eq!(
        bitboard_string(overall_blocker_bb),
        "\
        ........\n\
        .......1\n\
        .......1\n\
        .......1\n\
        .......1\n\
        .......1\n\
        .111111.\n\
        ........"
            .to_string()
    );
    {
        let specific_blocker_bb = generate_specific_blocker_bb(overall_blocker_bb, 1);

        assert_eq!(
            bitboard_string(specific_blocker_bb),
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
    }
    {
        let specific_blocker_bb = generate_specific_blocker_bb(overall_blocker_bb, 0b10101010);

        assert_eq!(
            bitboard_string(specific_blocker_bb),
            "\
            ........\n\
            ........\n\
            ........\n\
            ........\n\
            .......1\n\
            ........\n\
            ..1.1.1.\n\
            ........"
                .to_string()
        );
    }
}

pub fn generate_moves_for_piece(
    piece_index: isize,
    piece_for_magic: PieceForMagic,
) -> Vec<MoveAndBlocker> {
    let overall_blocker_bb = generate_overall_blocker_bb(piece_index, piece_for_magic);
    let num_seeds = 1 << overall_blocker_bb.count_ones();

    let mut moves: Vec<MoveAndBlocker> = vec![];

    for seed in 0..num_seeds {
        let specific_blocker_bb = generate_specific_blocker_bb(overall_blocker_bb, seed);

        let mut move_bb = Bitboard::default();
        for &offset in OFFSETS_FOR_MAGIC[piece_for_magic as usize] {
            move_bb |= generate_walk_bb(piece_index, specific_blocker_bb, offset);
        }

        moves.push(MoveAndBlocker {
            move_bb,
            blocker_bb: specific_blocker_bb,
        });
    }

    moves
}

pub fn magic_value_works(
    magic: u64,
    moves: &[MoveAndBlocker],
    bits_required: isize,
) -> Option<MagicValue> {
    if magic.count_ones() < 6 {
        return None;
    }

    let magic_value = MagicValue {
        magic,
        bits_required,
    };
    if magic_works(magic_value, moves) {
        Some(magic_value)
    } else {
        None
    }
}

fn rand64() -> u64 {
    rand::thread_rng().gen()
}

fn mostly_zero_rand_64() -> u64 {
    rand64() & rand64() & rand64()
}

// Good enough magic bit counts from https://www.chessprogramming.org/Looking_for_Magics

const ROOK_BITS: [[isize; 8]; 8] = [
    [12, 11, 11, 11, 11, 11, 11, 12],
    [11, 10, 10, 10, 10, 10, 10, 11],
    [11, 10, 10, 10, 10, 10, 10, 11],
    [11, 10, 10, 10, 10, 10, 10, 11],
    [11, 10, 10, 10, 10, 10, 10, 11],
    [11, 10, 10, 10, 10, 10, 10, 11],
    [11, 10, 10, 10, 10, 10, 10, 11],
    [12, 11, 11, 11, 11, 11, 11, 12],
];

const BISHOP_BITS: [[isize; 8]; 8] = [
    [6, 5, 5, 5, 5, 5, 5, 6],
    [5, 5, 5, 5, 5, 5, 5, 5],
    [5, 5, 7, 7, 7, 7, 5, 5],
    [5, 5, 7, 9, 9, 7, 5, 5],
    [5, 5, 7, 9, 9, 7, 5, 5],
    [5, 5, 7, 7, 7, 7, 5, 5],
    [5, 5, 5, 5, 5, 5, 5, 5],
    [6, 5, 5, 5, 5, 5, 5, 6],
];

pub fn bits_required(piece_index: isize, piece_for_magic: PieceForMagic) -> isize {
    match piece_for_magic {
        PieceForMagic::Rook => ROOK_BITS[piece_index as usize / 8][piece_index as usize % 8],
        PieceForMagic::Bishop => BISHOP_BITS[piece_index as usize / 8][piece_index as usize % 8],
    }
}

pub fn find_best_magic(piece_index: isize, piece_for_magic: PieceForMagic) -> Option<MagicValue> {
    let moves = generate_moves_for_piece(piece_index, piece_for_magic);

    let bits_required = bits_required(piece_index, piece_for_magic);

    for _ in 0..100000000 {
        let magic = mostly_zero_rand_64();

        let magic_value = magic_value_works(magic, &moves, bits_required);
        if let Some(magic_value) = magic_value {
            return Some(magic_value);
        }
    }

    None
}

#[test]
pub fn test_find_best_magic() {
    let rook_magic = find_best_magic(28, PieceForMagic::Rook);
    assert!(rook_magic.is_some());
    let bishop_magic = find_best_magic(28, PieceForMagic::Bishop);
    assert!(bishop_magic.is_some());
}
