use std::{cmp::Ordering, collections::HashSet};

use lazy_static::lazy_static;

use crate::{
    game::Game,
    helpers::ErrorResult,
    moves::{all_moves, Capture, Move, MoveOptions, MoveType},
    types::Piece,
};

pub fn capture_sort(moves: &mut [Move]) -> ErrorResult<()> {
    let mut swap_i = 0;

    for piece in [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight] {
        let start_i = swap_i;
        for i in start_i..moves.len() {
            if moves[i].target_piece() == Some(piece) {
                moves.swap(swap_i, i);
                swap_i += 1;
            }
        }

        moves[start_i..swap_i]
            .sort_unstable_by(|a, b| a.piece.piece.centipawns().cmp(&b.piece.piece.centipawns()))
    }

    Ok(())
}

#[test]
fn test_capture_sort() {
    let fen = "8/2p1k3/3R4/3r4/4nq2/2p5/3Q4/2K5 b";

    let game = Game::from_fen(fen).unwrap();
    let mut moves: Vec<Move> = vec![];
    all_moves(&mut moves, game.player(), &game, MoveOptions::default()).unwrap();

    capture_sort(&mut moves).unwrap();

    println!("{:?}", moves);
}

#[test]
fn test_capture_sort_2() {
    let fen = "r3k2r/1bq1bppp/pp2p3/2p1n3/P3PP2/2PBN3/1P1BQ1PP/R4RK1 b kq - 0 16";

    let game = Game::from_fen(fen).unwrap();
    let mut moves: Vec<Move> = vec![];
    all_moves(&mut moves, game.player(), &game, MoveOptions::default()).unwrap();

    let moves_set = moves.iter().cloned().collect::<HashSet<_>>();

    capture_sort(&mut moves).unwrap();
    println!("{:?}", moves);

    for m in moves.iter() {
        assert!(moves_set.contains(m));
    }
    assert_eq!(moves_set.len(), moves.len());
}
