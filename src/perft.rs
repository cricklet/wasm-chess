use std::{collections::HashMap, iter, rc::Rc};

use fallible_streaming_iterator::FallibleStreamingIterator;
use scopeguard::defer;

use crate::{
    game::Game,
    helpers::{Error, ErrorResult},
    moves::{all_moves, index_in_danger, Move, OnlyCaptures, OnlyQueenPromotion},
    types::Piece,
};

fn assert_fen_matches(expected_fen: &str) {
    let game = Game::from_fen(expected_fen).unwrap();
    let game_fen = game.to_fen();

    let expected_fen: Vec<&str> = expected_fen.split(" ").collect();
    let game_fen: Vec<&str> = game_fen.split(" ").collect();

    for (expected, actual) in expected_fen.iter().zip(game_fen.iter()) {
        assert_eq!(expected, actual);
    }
}

#[test]
fn test_fen_start_board() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    assert_fen_matches(fen);

    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0";
    assert_fen_matches(fen);

    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -";
    assert_fen_matches(fen);

    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq";
    assert_fen_matches(fen);

    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w";
    assert_fen_matches(fen);

    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
    assert_fen_matches(fen);
}

fn traverse_game(game: &Game, depth: u8, max_depth: u8) -> usize {
    if depth >= max_depth {
        return 1;
    }

    let mut result = 0;

    let player = game.player;

    let moves = all_moves(player, game, OnlyCaptures::NO, OnlyQueenPromotion::NO);
    for m in moves {
        let m = m.unwrap();
        let mut next_game = *game;
        next_game.make_move(m).unwrap();

        let king_index = next_game.board.index_of_piece(player, Piece::King);
        let illegal_move = index_in_danger(player, king_index, &next_game).unwrap();

        if illegal_move {
            continue;
        }

        result += traverse_game(&next_game, depth + 1, max_depth);
    }

    result
}

struct TraverseGameCallbackParams<'game> {
    moves_stack: &'game Vec<Move>,
    game: &'game Game,
    depth: u8,
    max_depth: u8,
}

fn traverse_game_callback(
    moves_stack: &mut Vec<Move>,
    game: &Game,
    depth: u8,
    max_depth: u8,
    callback: &mut dyn FnMut(&TraverseGameCallbackParams),
) {
    callback(&TraverseGameCallbackParams {
        moves_stack,
        game,
        depth,
        max_depth,
    });

    if depth >= max_depth {
        return;
    }

    let player = game.player;

    let moves = all_moves(player, game, OnlyCaptures::NO, OnlyQueenPromotion::NO);
    for m in moves {
        let m = m.unwrap();

        moves_stack.push(m);

        let mut next_game = *game;
        next_game.make_move(m).unwrap();

        let king_index = next_game.board.index_of_piece(player, Piece::King);
        let illegal_move = index_in_danger(player, king_index, &next_game).unwrap();

        if !illegal_move {
            traverse_game_callback(moves_stack, &next_game, depth + 1, max_depth, callback);
        }

        moves_stack.pop();
    }
}

fn assert_perft_matches(fen: &str, expected_counts: &[u64]) {
    let game = Game::from_fen(fen).unwrap();

    assert_eq!(game.to_fen(), fen);

    for max_depth in 1..3 {
        let mut perft_per_move: HashMap<Move, u64> = HashMap::new();
        let mut perft_overall = 0;

        traverse_game_callback(&mut vec![], &game, 0, max_depth, &mut |params| {
            if params.depth == max_depth {
                perft_overall += 1;

                let count = perft_per_move.entry(params.moves_stack[0]).or_insert(0);
                *count += 1;
            }
        });

        assert_eq!(expected_counts[max_depth as usize], perft_overall);
    }

    // traverse

    // let mut perft = perft::Perft::new(game);
    // for (depth, expected_count) in expected_counts.iter().enumerate() {
    //     let count = perft.perft(depth as u8);
    //     assert_eq!(count, *expected_count);
    // }
}

#[test]
fn test_perft_start_board() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let expected_count = [
        1, 20, 400, 8902, 197281,
        // 4865609,
        // 119060324,
        // 3195901860,
    ];
    assert_perft_matches(fen, &expected_count);
}
