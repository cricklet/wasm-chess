use crate::helpers::*;

use super::*;

fn each_index_of_one_callback<F: FnMut(i32) -> Loop>(bb: Bitboard, mut callback: F) {
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

fn each_index_of_one_closure(bb: Bitboard) -> impl FnMut() -> Option<i32> {
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

fn each_index_of_one_iteratorfn(bb: Bitboard) -> FnIterator<i32, impl FnMut() -> Option<i32>> {
    let mut temp = bb;

    FnIterator::new(move || {
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

pub fn each_index_of_one(bb: Bitboard) -> impl Iterator<Item = u32> {
    let mut temp = bb;

    FnIterator::new(move || {
        if temp != 0 {
            let ls1 = least_significant_one(temp);
            let index = (ls1 - 1).count_ones();

            temp = temp ^ ls1;

            return Some(index as u32);
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
fn test_each_index_of_one_iteratorfn() {
    let binary = 0b0000000000000000000000000001000000000000000000000000000010000010;

    let mut expected = vec![36, 7, 1];

    for index in each_index_of_one_iteratorfn(binary) {
        assert_eq!(expected.pop().unwrap(), index);
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
