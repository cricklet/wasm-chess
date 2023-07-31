use std::{collections::HashMap, fs::File, io::Write};

use crate::{
    bitboard::{magic_constants, MAGIC_MOVE_TABLE},
    danger::Danger,
    game::Game,
    helpers::{indent, ErrorResult, Profiler},
    moves::{
        all_moves, index_in_danger, Capture, Move, MoveType, OnlyCaptures, OnlyQueenPromotion,
        Quiet,
    },
    types::{Piece, Player},
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

struct TraverseGameCallbackParams<'game> {
    moves_stack: &'game Vec<Move>,
    game: &'game Game,
    depth: usize,
    max_depth: usize,
}

fn print_game(fen: &str, moves: &Vec<Move>) -> String {
    let mut game = Game::from_fen(fen).unwrap();

    let uci_moves = moves
        .iter()
        .map(|m| m.to_uci())
        .collect::<Vec<_>>()
        .join(" ");

    let mut uci = "".to_string();
    uci.push_str(format!("position fen {} moves {}", fen, uci_moves).as_str());

    let mut s = "".to_string();
    s.push_str(format!("{}\n", game).as_str());

    for m in moves {
        s.push_str(format!("{}\n", m).as_str());
        let result = game.make_move(*m);
        match result {
            Ok(_) => {
                s.push_str(format!("{}\n", game).as_str());
            }
            Err(_) => {
                s.push_str("failed\n");
                break;
            }
        }
    }

    s.push_str(format!("{}", game.to_fen()).as_str());

    format!("'{}': {{\n{}\n}}", uci, indent(&s, 2))
}

fn traverse_game_callback(
    moves_stack: &mut Vec<Move>,
    game: &Game,
    depth: usize,
    max_depth: usize,
    callback: &mut dyn FnMut(&TraverseGameCallbackParams),
) -> ErrorResult<()> {
    callback(&TraverseGameCallbackParams {
        moves_stack,
        game,
        depth,
        max_depth,
    });

    if depth >= max_depth {
        return Ok(());
    }

    for result in game.for_each_legal_move() {
        let (next_game, m) = result?;
        moves_stack.push(m);
        traverse_game_callback(moves_stack, &next_game, depth + 1, max_depth, callback)?;
        moves_stack.pop();
    }

    Ok(())
}

pub fn run_perft(game: &Game, max_depth: usize) -> ErrorResult<usize> {
    let mut perft_overall = 0;

    let mut moves_stack = vec![];

    traverse_game_callback(&mut moves_stack, &game, 0, max_depth, &mut |params| {
        if params.depth == max_depth {
            perft_overall += 1;

            if params.depth == 0 {
                return;
            }
        }
    })?;

    Ok(perft_overall)
}

pub fn run_perft_counting_first_move(
    game: &Game,
    max_depth: usize,
) -> ErrorResult<(usize, HashMap<String, usize>)> {
    let mut total_count = 0;
    let mut count_per_move: HashMap<String, usize> = HashMap::new();

    for result in game.for_each_legal_move() {
        let (next_game, next_move) = result?;
        let move_str = next_move.to_uci();
        let count = count_per_move.entry(move_str).or_insert(0);

        *count = run_perft(&next_game, max_depth - 1)?;
        total_count += *count;
    }

    Ok((total_count, count_per_move))
}

fn assert_perft_matches_for_depth(fen: &str, max_depth: usize, expected_count: usize) {
    let game = Game::from_fen(fen).unwrap();
    assert_eq!(game.to_fen(), fen);

    let start_time = std::time::Instant::now();

    let mut perft_overall = 0;

    let mut moves_stack = vec![];

    let result = traverse_game_callback(&mut moves_stack, &game, 0, max_depth, &mut |params| {
        if params.depth == max_depth {
            perft_overall += 1;

            if params.depth == 0 {
                return;
            }
        }
    });

    result.expect(format!("{}", print_game(fen, &moves_stack)).as_str());

    let end_time = std::time::Instant::now();

    println!(
        "calculated perft for max_depth: {}, expected_count: {}, in {} ms",
        max_depth,
        expected_count,
        (end_time - start_time).as_millis()
    );

    assert_eq!(expected_count, perft_overall);
}

fn assert_perft_matches(fen: &str, expected_counts: &[usize]) {
    for (max_depth, &expected_count) in expected_counts.iter().enumerate() {
        assert_perft_matches_for_depth(fen, max_depth, expected_count);
    }
}

#[test]
fn test_perft_start_board() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    // Run once to warm up the magics cache
    let expected_count = [1, 20];
    assert_perft_matches(fen, &expected_count);

    {
        let p = Profiler::new("perft_start_board".to_string());
        let expected_count = [
            1, 20, 400, 8902, 197281, 4865609,
            // 119060324,
            // 3195901860,
        ];
        assert_perft_matches(fen, &expected_count);
        p.flush();
    }
}

// #[test]
// fn test_perft_start_board_depth_5() {
//     let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
//     let max_depth = 5;
//     let expected_count = 4865609;
//     let expected_branches = HashMap::from([
//         ("a2a3", 181046),
//         ("b2b3", 215255),
//         ("c2c3", 222861),
//         ("d2d3", 328511),
//         ("e2e3", 402988),
//         ("f2f3", 178889),
//         ("g2g3", 217210),
//         ("h2h3", 181044),
//         ("a2a4", 217832),
//         ("b2b4", 216145),
//         ("c2c4", 240082),
//         ("d2d4", 361790),
//         ("e2e4", 405385),
//         ("f2f4", 198473),
//         ("g2g4", 214048),
//         ("h2h4", 218829),
//         ("b1a3", 198572),
//         ("b1c3", 234656),
//         ("g1f3", 233491),
//         ("g1h3", 198502),
//     ]);
//     assert_perft_matches_for_depth(fen, max_depth, expected_count, Some(&expected_branches));
// }

#[test]
fn test_perft_position_2() {
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let expected_count = [1, 48, 2039, 97862];
    assert_perft_matches(fen, &expected_count);
}

#[test]
fn test_perft_position_3() {
    let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
    let expected_count = [1, 14, 191, 2812];
    assert_perft_matches(fen, &expected_count);
}

#[test]
fn test_perft_position_4() {
    let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
    let expected_count = [1, 6, 264, 9467];
    assert_perft_matches(fen, &expected_count);
}

#[test]
fn test_perft_position_5() {
    let fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
    let expected_count = [1, 44, 1486, 62379];
    assert_perft_matches(fen, &expected_count);
}

#[test]
fn test_perft_position_6() {
    let fen = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";
    let expected_count = [1, 46, 2079, 89890];
    assert_perft_matches(fen, &expected_count);
}

// fn measure_time(f: impl FnOnce()) -> std::time::Duration {
//     let start_time = std::time::Instant::now();
//     f();
//     let end_time = std::time::Instant::now();
//     end_time - start_time
// }

// #[test]
// fn test_copy_vs_ref() {
//     let n = 1000000;
//     let start = &mut Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w").unwrap();

//     let m = Move {
//         player: Player::White,
//         start_index: index_from_file_rank_str("e2").unwrap(),
//         end_index: index_from_file_rank_str("e3").unwrap(),
//         piece: Piece::Pawn,
//         move_type: MoveType::Quiet(Quiet::Move),
//     };

//     let copy_time = measure_time(|| {
//         for _ in 0..n {
//             let mut game = *start;
//         }
//     });

//     let ref_time = measure_time(|| {
//         for _ in 0..n {
//             let game = &mut *start;
//         }
//     });

//     println!("{:?} {:?}", copy_time, ref_time);
//     // assert_eq!(true, copy_time < ref_time.mul_f32(1.6));
//     // assert_eq!(true, copy_time > ref_time.mul_f32(1.3));
// }
