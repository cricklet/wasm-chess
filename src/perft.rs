use std::{collections::HashMap, fs::File, io::Write};

use pprof::protos::Message;

use crate::{
    bitboards::{magic_constants, MAGIC_MOVE_TABLE},
    game::Game,
    helpers::{indent, ErrorResult},
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

struct TraverseGameCallbackParams<'game> {
    moves_stack: &'game Vec<Move>,
    game: &'game Game,
    depth: usize,
    max_depth: usize,
}

fn print_game(fen: &str, moves: &Vec<Move>) -> String {
    let mut game = Game::from_fen(fen).unwrap();

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

    format!("history: {{\n{}\n}}", indent(&s, 2))
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

    let player = game.player;

    let moves = all_moves(player, game, OnlyCaptures::NO, OnlyQueenPromotion::NO);
    for m in moves {
        let m = m?;
        moves_stack.push(m);

        let mut next_game = *game;
        next_game.make_move(m)?;

        let king_index = next_game.board.index_of_piece(player, Piece::King);
        let illegal_move = index_in_danger(player, king_index, &next_game).unwrap();

        if !illegal_move {
            traverse_game_callback(moves_stack, &next_game, depth + 1, max_depth, callback)?;
        }

        moves_stack.pop();
    }

    Ok(())
}

pub fn run_perft(game: &Game, max_depth: usize) -> ErrorResult<(usize, HashMap<Move, usize>)> {
    let mut perft_per_move: HashMap<Move, usize> = HashMap::new();
    let mut perft_overall = 0;

    let mut moves_stack = vec![];

    traverse_game_callback(&mut moves_stack, &game, 0, max_depth, &mut |params| {
        if params.depth == max_depth {
            perft_overall += 1;

            if params.depth == 0 {
                return;
            }

            let count = perft_per_move.entry(params.moves_stack[0]).or_insert(0);
            *count += 1;
        }
    })?;

    Ok((perft_overall, perft_per_move))
}

fn assert_perft_matches_for_depth(
    fen: &str,
    max_depth: usize,
    expected_count: usize,
    expected_branches: Option<&HashMap<&str, usize>>,
) {
    let game = Game::from_fen(fen).unwrap();
    assert_eq!(game.to_fen(), fen);

    let start_time = std::time::Instant::now();

    let mut perft_per_move: HashMap<Move, usize> = HashMap::new();
    let mut perft_overall = 0;

    let mut moves_stack = vec![];

    let result = traverse_game_callback(&mut moves_stack, &game, 0, max_depth, &mut |params| {
        if params.depth == max_depth {
            perft_overall += 1;

            if params.depth == 0 {
                return;
            }

            let count = perft_per_move.entry(params.moves_stack[0]).or_insert(0);
            *count += 1;
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

    for (m, count) in perft_per_move.iter() {
        let m = format!("{}{}", m.start_index, m.end_index);

        if let Some(expected_branches) = expected_branches {
            let expected_count = expected_branches.get(m.as_str()).unwrap();
            assert_eq!(expected_count, count, "incorrect perft for: {}", m);
        }
    }

    assert_eq!(expected_count, perft_overall);
}

fn assert_perft_matches(fen: &str, expected_counts: &[usize]) {
    for (max_depth, &expected_count) in expected_counts.iter().enumerate() {
        assert_perft_matches_for_depth(fen, max_depth, expected_count, None);
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
    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(1000)
        .blocklist(&["libc", "libgcc", "pthread", "vdso", "backtrace"])
        .build()
        .unwrap();

    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let expected_count = [
        1, 20, 400, 8902, 197281, 4865609,
        // 119060324,
        // 3195901860,
    ];
    assert_perft_matches(fen, &expected_count);

    match guard.report().build() {
        Ok(report) => {
            let mut file = File::create("profile.pb").unwrap();
            let profile = report.pprof().unwrap();

            let mut content = Vec::new();
            profile.write_to_vec(&mut content).unwrap();

            file.write_all(&content).unwrap();
        }
        Err(_) => {}
    };
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
fn test_perft_start_board_a2a4_depth_4() {
    let fen = "rnbqkbnr/pppppppp/8/8/P7/8/1PPPPPPP/RNBQKBNR b KQkq - 0 1";
    let max_depth = 4;
    let expected_count = 4865609;
    let expected_branches = HashMap::from([
        ("a7a6", 9312),
        ("b7b6", 10348),
        ("c7c6", 10217),
        ("d7d6", 13203),
        ("e7e6", 14534),
        ("f7f6", 9328),
        ("g7g6", 10310),
        ("h7h6", 9328),
        ("a7a5", 9062),
        ("b7b5", 11606),
        ("c7c5", 10737),
        ("d7d5", 13725),
        ("e7e5", 14560),
        ("f7f5", 9847),
        ("g7g5", 10293),
        ("h7h5", 10293),
        ("b8a6", 9827),
        ("b8c6", 10746),
        ("g8f6", 10758),
        ("g8h6", 9798),
    ]);
    assert_perft_matches_for_depth(fen, max_depth, expected_count, Some(&expected_branches));
}

fn measure_time(f: impl FnOnce()) -> std::time::Duration {
    let start_time = std::time::Instant::now();
    f();
    let end_time = std::time::Instant::now();
    end_time - start_time
}

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
