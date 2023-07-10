use super::*;

#[derive(Debug, Clone, Copy)]
pub struct MagicValue {
    magic: u64,
    bits_in_magic_index: isize,
}

pub struct MagicMoveTable {
    // Each of the 64 indices on a board have a magic-lookup precomputed,
    // allowing us to look up a bitboard of possible moves given the
    // current occupancy of the board.
    //
    // eg
    // let blocker_bb = magic_table.blocker_masks[piece_index] & all_occupied_bb
    // let magic_values = magic_table.magics[piece_index]
    // let magic_index = compute_magic_index(magic_values.magic, blocker_bb, magic_values.bits_in_magic_index)
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
    let magic_index = (blocker_bb * magic.magic) >> (64 - magic.bits_in_magic_index);
    magic_index as usize
}

pub struct MoveAndBlocker {
    move_bb: Bitboard,
    blocker_bb: Bitboard,
}

pub fn magic_works(magic_value: MagicValue, expected_moves: &[MoveAndBlocker]) -> bool {
    let size = 1 << magic_value.bits_in_magic_index;
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

pub fn generate_walk_bb(piece_index: isize, blocker_bb: Bitboard, offset: isize) -> Bitboard {
    let premove_mask = pre_move_mask(offset).unwrap();
    let mut last_location_bb = single_bitboard(piece_index);

    let mut walk_bb = Bitboard::default();

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

pub fn generate_overall_blocker_bb(start_index: isize, offsets: &[isize]) -> Bitboard {
    let mut bb = Bitboard::default();
    for offset in offsets {
        let walk_bb = generate_walk_bb(start_index, 0, *offset);
        bb |= walk_bb;
    }

    bb
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
        let overall_blocker_bb = generate_overall_blocker_bb(start_index, &BISHOP_DIRS);

        assert_eq!(
            bitboard_string(overall_blocker_bb),
            "\
        ........\n\
        .......1\n\
        ......1.\n\
        .....1..\n\
        1...1...\n\
        .1.1....\n\
        ........\n\
        .1.1...."
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
    let overall_blocker_bb = generate_overall_blocker_bb(start_index, &BISHOP_DIRS);

    assert_eq!(
        bitboard_string(overall_blocker_bb),
        "\
        ........\n\
        .......1\n\
        ......1.\n\
        .....1..\n\
        1...1...\n\
        .1.1....\n\
        ........\n\
        .1.1...."
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
        ........\n\
        .1......"
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
            ......1.\n\
            ........\n\
            ....1...\n\
            ...1....\n\
            ........\n\
            ...1...."
                .to_string()
        );
    }
}

// pub fn generate_moves_for_piece_and_blockers(
//     piece_index: isize,
//     blocker_bb: Bitboard,
//     offsets: &[isize],
// ) -> [MoveAndBlocker] {
// }
