use std::collections::HashMap;


use crate::bitboard::warm_magic_cache;

use super::{
    danger::Danger,
    game::{Game, Legal},
    helpers::{err_result, indent, ErrorResult},
    iterative_traversal::TraversalStack,
    moves::{Move, MoveBuffer, MoveOptions},
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

    let danger = Danger::from(game.player, &game.board)?;

    let mut moves = MoveBuffer::default();
    game.fill_pseudo_move_buffer(&mut moves, MoveOptions::default())?;

    for &m in moves.iter() {
        let mut next_game = game.clone();
        next_game.make_move(m)?;

        if next_game.move_legality(&m, &danger) == Legal::No {
            continue;
        }

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

    let danger = Danger::from(game.player, &game.board)?;

    let mut moves = MoveBuffer::default();
    game.fill_pseudo_move_buffer(&mut moves, MoveOptions::default())?;

    for &next_move in moves.iter() {
        let mut next_game = game.clone();
        next_game.make_move(next_move)?;

        if next_game.move_legality(&next_move, &danger) == Legal::No {
            continue;
        }

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

pub fn run_perft_recursively(game: Game, max_depth: usize) -> ErrorResult<usize> {
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

pub fn run_perft_iteratively<const N: usize>(game: Game) -> ErrorResult<usize> {
    let mut data = TraversalStack::<(), N>::new(game)?;
    let mut overall_count = 0;

    if N <= 1 {
        return Ok(1);
    }

    loop {
        // Leaf node case:
        if data.depth + 1 >= N {
            overall_count += 1;
            data.depth -= 1;
        }

        // We have moves to traverse, dig deeper
        let next_move = data.get_and_increment_move()?;
        if let Some(next_move) = next_move {
            let (current, next) = data.current_and_next_mut()?;

            let result = next.setup_from_move(current, &next_move)?;
            if result == Legal::No {
                continue;
            } else {
                data.depth += 1;
                continue;
            }
        }

        // We're out of moves to traverse, pop back up.
        if data.depth == 0 {
            break;
        } else {
            data.depth -= 1;
            continue;
        }
    }

    Ok(overall_count)
}

pub fn run_perft_iteratively_to_depth(game: Game, max_depth: usize) -> ErrorResult<usize> {
    match max_depth {
        0 => Ok(1),
        1 => run_perft_iteratively::<1>(game),
        2 => run_perft_iteratively::<2>(game),
        3 => run_perft_iteratively::<3>(game),
        4 => run_perft_iteratively::<4>(game),
        5 => run_perft_iteratively::<5>(game),
        6 => run_perft_iteratively::<6>(game),
        7 => run_perft_iteratively::<7>(game),
        8 => run_perft_iteratively::<8>(game),
        9 => run_perft_iteratively::<9>(game),
        10 => run_perft_iteratively::<10>(game),
        _ => err_result("unsupported depth"),
    }
}

#[test]
fn test_perft_start_board() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    let expected_count = [
        1, 20, 400, 8902, 197281, // 4865609, // 119060324,
               // 3195901860,
    ];

    run_perft_recursively(Game::from_fen(fen).unwrap(), 2).unwrap();

    for (i, expected_count) in expected_count.into_iter().enumerate().collect::<Vec<_>>() {
        let start_time = std::time::Instant::now();

        let max_depth = i;

        let count = run_perft_recursively(Game::from_fen(fen).unwrap(), max_depth).unwrap();
        assert_eq!(count, expected_count);

        let end_time = std::time::Instant::now();

        println!(
            "calculated perft for max_depth: {}, expected_count: {}, in {} ms",
            max_depth,
            expected_count,
            (end_time - start_time).as_millis()
        );
    }
}

#[test]
fn test_perft_start_board_iteratively() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    let expected_count = [
        1, 20, 400, 8902, 197281, // 4865609, // 119060324,
               // 3195901860,
    ];

    run_perft_iteratively_to_depth(Game::from_fen(fen).unwrap(), 2).unwrap();

    for (i, expected_count) in expected_count.into_iter().enumerate().collect::<Vec<_>>() {
        let start_time = std::time::Instant::now();

        let max_depth = i + 1;

        let count =
            run_perft_iteratively_to_depth(Game::from_fen(fen).unwrap(), max_depth).unwrap();
        assert_eq!(count, expected_count);

        let end_time = std::time::Instant::now();

        println!(
            "calculated perft for max_depth: {}, expected_count: {}, in {} ms",
            max_depth,
            expected_count,
            (end_time - start_time).as_millis()
        );
    }
}

const MAX_PERFT_DEPTH: usize = 10;


#[derive(Debug, PartialEq, Eq)]
pub enum PerftLoopResult {
    Continue,
    Done,
    Interrupted,
}

#[derive(Debug)]
pub struct PerftLoop {
    pub stack: TraversalStack<(), MAX_PERFT_DEPTH>,

    pub count: usize,
    pub max_depth: usize,
    pub start_fen: String,

    loop_count: usize,
}
const LOOP_COUNT: usize = 1_000_000;

impl PerftLoop {
    pub fn new(fen: &str, max_depth: usize) -> Self {
        if max_depth > MAX_PERFT_DEPTH {
            panic!("max_depth must be <= {}", MAX_PERFT_DEPTH);
        }

        let game = Game::from_fen(fen).unwrap();
        let stack = TraversalStack::<(), MAX_PERFT_DEPTH>::new(game).unwrap();

        Self {
            stack,
            count: 0,
            max_depth,
            loop_count: LOOP_COUNT,
            start_fen: fen.to_string(),
        }
    }

    fn iterate(&mut self) -> PerftLoopResult {
        let ref mut traversal = self.stack;

        // Leaf node case:
        if traversal.depth + 1 >= self.max_depth {
            self.count += 1;
            traversal.depth -= 1;

            return PerftLoopResult::Continue;
        }

        // We have moves to traverse, dig deeper
        let next_move = traversal.get_and_increment_move().unwrap();
        if let Some(next_move) = next_move {
            let (current, next) = traversal.current_and_next_mut().unwrap();

            let result = next.setup_from_move(current, &next_move).unwrap();
            if result == Legal::No {
                return PerftLoopResult::Continue;
            } else {
                traversal.depth += 1;
                return PerftLoopResult::Continue;
            }
        }

        // We're out of moves to traverse, pop back up.
        if traversal.depth == 0 {
            return PerftLoopResult::Done;
        } else {
            traversal.depth -= 1;
            return PerftLoopResult::Continue;
        }
    }

    pub fn iterate_loop(&mut self) -> PerftLoopResult {
        for _ in 0..self.loop_count {
            let result = self.iterate();
            if result != PerftLoopResult::Continue {
                return result;
            }
        }

        PerftLoopResult::Continue
    }


}
