use std::rc::Rc;

use super::super::{
    helpers::{err_result, ErrorResult},
    types::Piece,
};

use super::*;
use memoize::memoize;
use rand::*;
use strum::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum WalkType {
    Rook,
    Bishop,
}

pub fn walk_type_for_piece(piece: Piece) -> ErrorResult<&'static [WalkType]> {
    match piece {
        Piece::Rook => Ok(&[WalkType::Rook]),
        Piece::Bishop => Ok(&[WalkType::Bishop]),
        Piece::Queen => Ok(&[WalkType::Rook, WalkType::Bishop]),
        _ => err_result(&format!("piece {:?} does not walk", piece)),
    }
}

#[test]
fn test_piece_enum_iter() {
    {
        let mut iter = WalkType::iter();
        assert_eq!(iter.next(), Some(WalkType::Rook));
        assert_eq!(iter.next(), Some(WalkType::Bishop));
        assert_eq!(iter.next(), None);
    }
    {
        let mut iter = WalkType::iter().rev();
        assert_eq!(iter.next(), Some(WalkType::Bishop));
        assert_eq!(iter.next(), Some(WalkType::Rook));
        assert_eq!(iter.next(), None);
    }
}

const DIRECTIONS_FOR_MAGIC: [&[Direction]; 2] = [&ROOK_DIRS, &BISHOP_DIRS];

#[derive(Debug, Clone, Copy, Default)]
pub struct MagicValue {
    pub magic: u64,
    pub bits_required: usize,
}

pub fn magic_index_for_specific_blocker_bb(magic: &MagicValue, blocker_bb: Bitboard) -> usize {
    let magic_index = (blocker_bb.wrapping_mul(magic.magic)) >> (64 - magic.bits_required);
    magic_index as usize
}

#[derive(Debug, Clone, Copy)]
pub struct PotentialMoves {
    potential_moves_bb: Bitboard,
    specific_blocker_bb: Bitboard,
}

pub type ForPieceIndex<T> = [T; 64];

#[derive(Default)]
pub struct ForWalkType<T: Clone> {
    pub rook: T,
    pub bishop: T,
}

impl<T: Clone> ForWalkType<T> {
    pub fn new(t: T) -> ForWalkType<T> {
        ForWalkType {
            rook: t.clone(),
            bishop: t.clone(),
        }
    }
}

impl<T: Clone> std::ops::Index<WalkType> for ForWalkType<T> {
    type Output = T;

    fn index(&self, index: WalkType) -> &Self::Output {
        match index {
            WalkType::Rook => &self.rook,
            WalkType::Bishop => &self.bishop,
        }
    }
}

impl<T: Clone> std::ops::IndexMut<WalkType> for ForWalkType<T> {
    fn index_mut(&mut self, index: WalkType) -> &mut Self::Output {
        match index {
            WalkType::Rook => &mut self.rook,
            WalkType::Bishop => &mut self.bishop,
        }
    }
}

pub fn generate_magic_moves(
    piece_index: BoardIndex,
    piece: WalkType,
    magic_value: &MagicValue,
) -> Option<Vec<Bitboard>> {
    let potential_moves = potential_moves_for_piece(piece_index, piece);

    let size = 1 << magic_value.bits_required;
    let mut magic_move_table: Vec<Bitboard> = vec![0; size];
    let mut populated: Vec<bool> = vec![false; size];

    for potential in potential_moves.as_ref() {
        let magic_index =
            magic_index_for_specific_blocker_bb(magic_value, potential.specific_blocker_bb);

        if populated[magic_index] {
            if magic_move_table[magic_index] != potential.potential_moves_bb {
                return None;
            }
        } else {
            magic_move_table[magic_index] = potential.potential_moves_bb;
            populated[magic_index] = true;
        }
    }

    Some(magic_move_table)
}

#[memoize]
pub fn generate_walk_bb(piece_index: BoardIndex, blocker_bb: Bitboard, dir: Direction) -> Bitboard {
    let mut walk_bb = Bitboard::default();

    let premove_mask = pre_move_mask(dir);
    let mut last_location_bb = single_bitboard(piece_index);

    while last_location_bb != 0 {
        let next_location_bb =
            rotate_toward_index_63(last_location_bb & premove_mask, dir.offset());

        let quiet_bb = next_location_bb & !blocker_bb;
        let capture_bb = next_location_bb & blocker_bb;

        walk_bb |= quiet_bb | capture_bb;

        last_location_bb = quiet_bb
    }

    walk_bb
}

#[test]
fn test_generate_walk_bb() {
    let start_index = 10;
    let start_bb = single_bitboard(BoardIndex::from(start_index));

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
        let dir = Direction::NE;
        let walk_bb = generate_walk_bb(BoardIndex::from(start_index), blocker_bb, dir);

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
            ........",
        );
        let dir = Direction::NE;
        let walk_bb = generate_walk_bb(BoardIndex::from(start_index), blocker_bb, dir);

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

pub fn generate_mask_blockers_bb(start_index: BoardIndex, piece: WalkType) -> Bitboard {
    let mut mask_blockers_bb = Bitboard::default();

    for &dir in DIRECTIONS_FOR_MAGIC[piece as usize] {
        let walk_bb = generate_walk_bb(start_index, mask_blockers_bb, dir);
        let walk_bb_filtered = walk_bb & pre_move_mask(dir);

        mask_blockers_bb |= walk_bb_filtered;
    }

    mask_blockers_bb
}

#[test]
fn test_generate_overall_blocker_bb() {
    let start_index = 10 as usize;
    let start_bb = single_bitboard(BoardIndex::from(start_index));

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
        let mask_blockers_bb =
            generate_mask_blockers_bb(BoardIndex::from(start_index), WalkType::Bishop);

        assert_eq!(
            bitboard_string(mask_blockers_bb),
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

pub fn generate_specific_blocker_bb(mask_blockers_bb: Bitboard, seed: usize) -> Bitboard {
    let mut specific_blocker_bb = Bitboard::default();

    let num_bits = mask_blockers_bb.count_ones() as usize;
    for i in 0..num_bits {
        // If the bit at i is 1 in the seed
        if seed & (1 << i) != 0 {
            // Find the ith one bit in blockerMask and set the corresponding bit to one in result.
            for (j, bit_index) in each_index_of_one(mask_blockers_bb).enumerate() {
                if i == j {
                    specific_blocker_bb |= single_bitboard(bit_index);
                }
            }
        }
    }

    specific_blocker_bb
}

#[test]
fn test_generate_specific_blocker_bb() {
    let start_index = 15;
    let start_bb = single_bitboard(BoardIndex::from(start_index));

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
    let mask_blockers_bb = generate_mask_blockers_bb(BoardIndex::from(start_index), WalkType::Rook);

    assert_eq!(
        bitboard_string(mask_blockers_bb),
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
        let specific_blocker_bb = generate_specific_blocker_bb(mask_blockers_bb, 1);

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
        let specific_blocker_bb = generate_specific_blocker_bb(mask_blockers_bb, 0b10101010);

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

#[memoize]
pub fn potential_moves_for_piece(
    piece_index: BoardIndex,
    piece: WalkType,
) -> Rc<Vec<PotentialMoves>> {
    let mask_blockers_bb = generate_mask_blockers_bb(piece_index, piece);
    let num_seeds = 1 << mask_blockers_bb.count_ones();

    let mut moves = vec![];

    for seed in 0..num_seeds {
        let specific_blocker_bb = generate_specific_blocker_bb(mask_blockers_bb, seed);

        let mut potential_moves_bb = Bitboard::default();
        for &offset in DIRECTIONS_FOR_MAGIC[piece as usize] {
            potential_moves_bb |= generate_walk_bb(piece_index, specific_blocker_bb, offset);
        }

        moves.push(PotentialMoves {
            potential_moves_bb,
            specific_blocker_bb,
        });
    }

    moves.into()
}

fn rand64() -> u64 {
    rand::thread_rng().gen()
}

fn mostly_zero_rand_64() -> u64 {
    rand64() & rand64() & rand64()
}

// Good enough magic bit counts from https://www.chessprogramming.org/Looking_for_Magics

const ROOK_BITS: [[usize; 8]; 8] = [
    [12, 11, 11, 11, 11, 11, 11, 12],
    [11, 10, 10, 10, 10, 10, 10, 11],
    [11, 10, 10, 10, 10, 10, 10, 11],
    [11, 10, 10, 10, 10, 10, 10, 11],
    [11, 10, 10, 10, 10, 10, 10, 11],
    [11, 10, 10, 10, 10, 10, 10, 11],
    [11, 10, 10, 10, 10, 10, 10, 11],
    [12, 11, 11, 11, 11, 11, 11, 12],
];

const BISHOP_BITS: [[usize; 8]; 8] = [
    [6, 5, 5, 5, 5, 5, 5, 6],
    [5, 5, 5, 5, 5, 5, 5, 5],
    [5, 5, 7, 7, 7, 7, 5, 5],
    [5, 5, 7, 9, 9, 7, 5, 5],
    [5, 5, 7, 9, 9, 7, 5, 5],
    [5, 5, 7, 7, 7, 7, 5, 5],
    [5, 5, 5, 5, 5, 5, 5, 5],
    [6, 5, 5, 5, 5, 5, 5, 6],
];

pub fn bits_required(piece_index: BoardIndex, piece: WalkType) -> usize {
    match piece {
        WalkType::Rook => ROOK_BITS[piece_index.i as usize / 8][piece_index.i as usize % 8],
        WalkType::Bishop => BISHOP_BITS[piece_index.i as usize / 8][piece_index.i as usize % 8],
    }
}

pub fn warm_magic_cache() {
    let piece_index = BoardIndex::from(0);
    for piece in WalkType::iter() {
        find_magic_value(piece_index, piece);
    }
}

pub fn find_magic_value(piece_index: BoardIndex, piece: WalkType) -> Option<MagicValue> {
    let bits_required = bits_required(piece_index, piece);

    for _ in 0..100000000 {
        let magic = mostly_zero_rand_64();

        let magic_value = MagicValue {
            magic,
            bits_required,
        };

        let magic_move_table = generate_magic_moves(piece_index, piece, &magic_value);

        if magic_move_table.is_some() {
            return Some(magic_value);
        }
    }

    None
}

pub fn moves_bb_for_piece_and_blockers(
    piece_index: BoardIndex,
    piece: WalkType,
    occupancy_bb: Bitboard,
) -> Bitboard {
    let ref magic_value = MAGIC_MOVE_TABLE.magics[piece][piece_index.i];
    let ref magic_moves = MAGIC_MOVE_TABLE.moves_table[piece][piece_index.i];
    let ref mask_blockers_bb = MAGIC_MOVE_TABLE.mask_blocker_bbs[piece][piece_index.i];

    let specific_blocker_bb = mask_blockers_bb & occupancy_bb;
    let magic_index = magic_index_for_specific_blocker_bb(magic_value, specific_blocker_bb);

    let moves_bb = magic_moves[magic_index];

    moves_bb
}

#[test]
fn test_find_best_magic() {
    let mut magics_for_piece: [[u64; 64]; 2] = [[0; 64]; 2];
    let mut bits_required_for_piece: [[usize; 64]; 2] = [[0; 64]; 2];
    for piece in WalkType::iter() {
        for piece_index in 0..64 as usize {
            let magic = find_magic_value(BoardIndex::from(piece_index), piece);
            assert!(magic.is_some());

            magics_for_piece[piece as usize][piece_index as usize] = magic.unwrap().magic;
            bits_required_for_piece[piece as usize][piece_index as usize] =
                magic.unwrap().bits_required;

            // Remove this `break;` to recompute values for `magic_constants.rs`
            break;
        }

        // for piece_index in 0..64 as usize {
        //     let magic_value = MagicValue {
        //         magic: magics_for_piece[piece as usize][piece_index as usize],
        //         bits_required: bits_required_for_piece[piece as usize]
        //             [piece_index as usize],
        //     };
        //     magic_move_table(piece_index, piece, magic_value).unwrap();
        // }
    }

    // println!("{:?}", magics_for_piece);
    // println!("{:?}", bits_required_for_piece);
}

#[test]
fn test_moves_for_piece_and_blockers() {
    let occupancy_bb = bitboard_from_string(
        "\
    .......1\n\
    ....1...\n\
    ..1...1.\n\
    ........\n\
    .....1..\n\
    ........\n\
    .1...1..\n\
    ........",
    );

    let piece_bb = bitboard_from_string(
        "\
    ........\n\
    ........\n\
    .....1..\n\
    ........\n\
    ........\n\
    ........\n\
    ........\n\
    ........",
    );

    {
        let moves_bb = moves_bb_for_piece_and_blockers(
            first_index_of_one(piece_bb),
            WalkType::Bishop,
            occupancy_bb,
        );

        assert_eq!(
            bitboard_string(moves_bb),
            "\
    .......1\n\
    ....1.1.\n\
    ........\n\
    ....1.1.\n\
    ...1...1\n\
    ..1.....\n\
    .1......\n\
    ........"
        );
    }

    {
        let moves_bb = moves_bb_for_piece_and_blockers(
            first_index_of_one(piece_bb),
            WalkType::Rook,
            occupancy_bb,
        );

        assert_eq!(
            bitboard_string(moves_bb),
            "\
            .....1..\n\
            .....1..\n\
            ..111.1.\n\
            .....1..\n\
            .....1..\n\
            ........\n\
            ........\n\
            ........"
        );
    }
}
